use crate::{Diagnostic, PublicMessage};
use std::collections::BTreeSet;

/// Conservative application-side references to generated `l.*` message paths.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApplicationUsage {
    static_paths: BTreeSet<UsagePath>,
    dynamic_prefixes: BTreeSet<UsagePath>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UsagePath {
    display: String,
    segments: Vec<UsageSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UsageSegment {
    raw: String,
    decoded: Option<Vec<String>>,
}

impl ApplicationUsage {
    pub fn from_source(source: &str) -> Self {
        let mut usage = Self::default();
        usage.extend_source(source);
        usage
    }

    pub fn extend_source(&mut self, source: &str) {
        let LexedSource {
            tokens,
            uncertain_syntax,
        } = lex(source);
        if uncertain_syntax {
            self.dynamic_prefixes.insert(UsagePath::root());
        }
        let (imports, mut roots, factories) = imported_symbols(&tokens, source);
        for index in 0..tokens.len() {
            let Some((name, after_symbol, _)) = symbol_reference(&tokens, source, index) else {
                continue;
            };
            if imports.contains(&index) || !factories.contains(name) {
                continue;
            }
            let Some(after_call) = call_end(&tokens, source, after_symbol) else {
                continue;
            };
            if member_path(&tokens, after_call).0.is_empty() {
                if let Some((binding, _)) = declaration_binding(&tokens, source, index) {
                    roots.insert(binding);
                }
            }
        }

        let mut index = 0;
        while index < tokens.len() {
            if imports.contains(&index) {
                index += 1;
                continue;
            }
            let Some((root, after_symbol, bracket_property)) =
                symbol_reference(&tokens, source, index)
            else {
                index += 1;
                continue;
            };
            if factories.contains(root) {
                if let Some(after_call) = call_end(&tokens, source, after_symbol) {
                    let (segments, dynamic, next) = member_path(&tokens, after_call);
                    if segments.is_empty() {
                        if declaration_binding(&tokens, source, index)
                            .map_or(true, |(_, exported)| exported)
                        {
                            self.dynamic_prefixes.insert(UsagePath::root());
                        }
                    } else {
                        self.record_path(&tokens, source, segments, dynamic, next);
                    }
                    index += 1;
                    continue;
                }
                self.dynamic_prefixes.insert(UsagePath::root());
                index += 1;
                continue;
            }
            if !bracket_property && is_binding_declaration(&tokens, source, index) {
                index += 1;
                continue;
            }
            if !roots.contains(root)
                || (!bracket_property
                    && is_member_property(&tokens, index)
                    && !matches!(root, "l" | "lgl" | "messages"))
            {
                index += 1;
                continue;
            }

            let (segments, dynamic, next) = member_path(&tokens, after_symbol);
            self.record_path(&tokens, source, segments, dynamic, next);
            index = next.max(index + 1);
        }
    }

    fn record_path(
        &mut self,
        tokens: &[Token],
        source: &str,
        segments: Vec<UsageSegment>,
        dynamic: bool,
        next: usize,
    ) {
        if segments.is_empty() {
            self.dynamic_prefixes.insert(UsagePath::root());
            return;
        }
        let path = UsagePath::new(segments);
        if dynamic || !is_invocation(tokens, source, next) {
            self.dynamic_prefixes.insert(path);
        } else {
            self.static_paths.insert(path);
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.static_paths.extend(other.static_paths);
        self.dynamic_prefixes.extend(other.dynamic_prefixes);
    }

    pub fn static_paths(&self) -> impl Iterator<Item = &str> {
        self.static_paths.iter().map(|path| path.display.as_str())
    }

    pub fn dynamic_prefixes(&self) -> impl Iterator<Item = &str> {
        self.dynamic_prefixes
            .iter()
            .map(|path| path.display.as_str())
    }
}

pub fn analyze_unused_messages(
    schema_messages: &[PublicMessage],
    usage: &ApplicationUsage,
    ignored_prefixes: &[String],
) -> Vec<Diagnostic> {
    schema_messages
        .iter()
        .filter(|message| {
            !usage
                .static_paths
                .iter()
                .any(|path| path.overlaps(&message.name))
                && !usage
                    .dynamic_prefixes
                    .iter()
                    .any(|prefix| prefix.overlaps(&message.name))
                && !ignored_prefixes
                    .iter()
                    .any(|prefix| path_is_within(&message.name, prefix))
        })
        .map(|message| {
            Diagnostic::warning(
                format!(
                    "schema message `{}` is not referenced by configured application sources",
                    message.name
                ),
                message.span,
            )
            .as_lint("unused_message")
            .with_note(
                "remove the message or add its canonical path to \
                 `analysis.unused_messages.ignore` when it is selected dynamically",
            )
        })
        .collect()
}

fn path_is_within(path: &str, prefix: &str) -> bool {
    prefix.is_empty()
        || path == prefix
        || path
            .strip_prefix(prefix)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

impl UsagePath {
    fn root() -> Self {
        Self {
            display: String::new(),
            segments: Vec::new(),
        }
    }

    fn new(segments: Vec<UsageSegment>) -> Self {
        let display = segments
            .iter()
            .flat_map(|segment| {
                segment.decoded.as_ref().map_or_else(
                    || vec![segment.raw.as_str()],
                    |decoded| decoded.iter().map(String::as_str).collect(),
                )
            })
            .collect::<Vec<_>>()
            .join(".");
        Self { display, segments }
    }

    fn overlaps(&self, canonical: &str) -> bool {
        if self.segments.is_empty() || canonical.is_empty() {
            return true;
        }
        let canonical = canonical.split('.').collect::<Vec<_>>();
        let mut reachable = BTreeSet::from([0_usize]);
        for segment in &self.segments {
            let mut next = BTreeSet::new();
            for position in reachable {
                if position == canonical.len() {
                    return true;
                }
                let raw = std::slice::from_ref(&segment.raw);
                let alternatives =
                    std::iter::once(raw).chain(segment.decoded.iter().map(Vec::as_slice));
                for alternative in alternatives {
                    let remaining = &canonical[position..];
                    let shared = remaining.len().min(alternative.len());
                    if remaining[..shared]
                        .iter()
                        .zip(&alternative[..shared])
                        .any(|(canonical, candidate)| *canonical != candidate.as_str())
                    {
                        continue;
                    }
                    if remaining.len() <= alternative.len() {
                        return true;
                    }
                    next.insert(position + alternative.len());
                }
            }
            if next.is_empty() {
                return false;
            }
            reachable = next;
        }
        !reachable.is_empty()
    }
}

impl UsageSegment {
    fn new(raw: String) -> Self {
        let decoded = decode_generated_identifier(&raw)
            .filter(|decoded| decoded != &raw)
            .map(|decoded| decoded.split('.').map(str::to_owned).collect());
        Self { raw, decoded }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Identifier(String),
    StringLiteral(Option<String>),
    OpaqueLiteral,
    Dot,
    Question,
    OpenBracket,
    CloseBracket,
    Semicolon,
    Other,
}

fn symbol_reference<'a>(
    tokens: &'a [Token],
    source: &str,
    index: usize,
) -> Option<(&'a str, usize, bool)> {
    match &tokens.get(index)?.kind {
        TokenKind::Identifier(name) => Some((name, index + 1, false)),
        TokenKind::StringLiteral(Some(name)) if is_bracket_member(tokens, source, index) => {
            Some((name, index + 2, true))
        }
        _ => None,
    }
}

fn is_bracket_member(tokens: &[Token], source: &str, index: usize) -> bool {
    if !matches!(
        index.checked_sub(1).and_then(|index| tokens.get(index)),
        Some(Token {
            kind: TokenKind::OpenBracket,
            ..
        })
    ) || !matches!(
        tokens.get(index + 1),
        Some(Token {
            kind: TokenKind::CloseBracket,
            ..
        })
    ) {
        return false;
    }
    let Some(receiver) = index.checked_sub(2) else {
        return false;
    };
    if can_end_member_receiver(&tokens[receiver], source) {
        return true;
    }
    matches!(tokens[receiver].kind, TokenKind::Dot)
        && matches!(
            receiver
                .checked_sub(1)
                .and_then(|index| tokens.get(index))
                .map(|token| &token.kind),
            Some(TokenKind::Question)
        )
        && receiver
            .checked_sub(2)
            .and_then(|index| tokens.get(index))
            .is_some_and(|token| can_end_member_receiver(token, source))
}

fn can_end_member_receiver(token: &Token, source: &str) -> bool {
    matches!(
        token.kind,
        TokenKind::Identifier(_) | TokenKind::StringLiteral(_) | TokenKind::CloseBracket
    ) || (matches!(token.kind, TokenKind::Other) && &source[token.start..token.end] == ")")
}

struct LexedSource {
    tokens: Vec<Token>,
    uncertain_syntax: bool,
}

fn lex(source: &str) -> LexedSource {
    let mut tokens = Vec::new();
    let mut position = 0;
    let mut uncertain_syntax = false;
    lex_code(
        source,
        &mut position,
        false,
        &mut tokens,
        &mut uncertain_syntax,
    );
    LexedSource {
        tokens,
        uncertain_syntax,
    }
}

fn lex_code(
    source: &str,
    position: &mut usize,
    stop_at_template_brace: bool,
    tokens: &mut Vec<Token>,
    uncertain_syntax: &mut bool,
) {
    let bytes = source.as_bytes();
    let mut brace_depth = 0_usize;
    let context_start = tokens.len();
    while *position < bytes.len() {
        let byte = bytes[*position];
        if byte.is_ascii_whitespace() {
            *position += 1;
            continue;
        }
        if byte == b'/' && bytes.get(*position + 1) == Some(&b'/') {
            *position += 2;
            while *position < bytes.len() && bytes[*position] != b'\n' {
                *position += 1;
            }
            continue;
        }
        if byte == b'/' && bytes.get(*position + 1) == Some(&b'*') {
            *position += 2;
            while *position + 1 < bytes.len()
                && !(bytes[*position] == b'*' && bytes[*position + 1] == b'/')
            {
                *position += 1;
            }
            if *position + 1 == bytes.len() {
                *uncertain_syntax = true;
                *position = bytes.len();
            } else {
                *position += 2;
            }
            continue;
        }
        if byte == b'/' && can_start_regex(&tokens[context_start..], source) {
            if !lex_regex(source, position, tokens) {
                *uncertain_syntax = true;
            }
            continue;
        }
        if byte == b'<'
            && can_start_markup(&tokens[context_start..], source, *position)
            && lex_markup(source, position, tokens, uncertain_syntax)
        {
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            if !lex_string(source, position, tokens) {
                *uncertain_syntax = true;
            }
            continue;
        }
        if byte == b'`' {
            if !lex_template(source, position, tokens, uncertain_syntax) {
                *uncertain_syntax = true;
            }
            continue;
        }
        if byte == b'{' && lex_svelte_block_close(source, position) {
            continue;
        }
        if byte == b'{' {
            brace_depth += 1;
            push_token(tokens, TokenKind::Other, *position, *position + 1);
            *position += 1;
            continue;
        }
        if byte == b'}' {
            if stop_at_template_brace && brace_depth == 0 {
                *position += 1;
                return;
            }
            brace_depth = brace_depth.saturating_sub(1);
            push_token(tokens, TokenKind::Other, *position, *position + 1);
            *position += 1;
            continue;
        }

        let Some(character) = source[*position..].chars().next() else {
            break;
        };
        if is_identifier_start(character) {
            let start = *position;
            *position += character.len_utf8();
            while *position < bytes.len() {
                let Some(character) = source[*position..].chars().next() else {
                    break;
                };
                if !is_identifier_continue(character) {
                    break;
                }
                *position += character.len_utf8();
            }
            push_token(
                tokens,
                TokenKind::Identifier(source[start..*position].to_owned()),
                start,
                *position,
            );
            continue;
        }

        let kind = match byte {
            b'.' => TokenKind::Dot,
            b'?' => TokenKind::Question,
            b'[' => TokenKind::OpenBracket,
            b']' => TokenKind::CloseBracket,
            b';' => TokenKind::Semicolon,
            _ => TokenKind::Other,
        };
        push_token(tokens, kind, *position, *position + 1);
        *position += 1;
    }
    if stop_at_template_brace {
        *uncertain_syntax = true;
    }
}

fn can_start_regex(tokens: &[Token], source: &str) -> bool {
    let Some(previous) = tokens.last() else {
        return true;
    };
    match &previous.kind {
        TokenKind::Identifier(name) => matches!(
            name.as_str(),
            "await"
                | "case"
                | "delete"
                | "do"
                | "else"
                | "in"
                | "instanceof"
                | "new"
                | "of"
                | "return"
                | "throw"
                | "typeof"
                | "void"
                | "yield"
        ),
        TokenKind::OpenBracket | TokenKind::Question | TokenKind::Semicolon => true,
        TokenKind::Other => {
            let spelling = &source[previous.start..previous.end];
            if matches!(spelling, "+" | "-")
                && tokens
                    .get(tokens.len().saturating_sub(2))
                    .is_some_and(|before| {
                        matches!(before.kind, TokenKind::Other)
                            && &source[before.start..before.end] == spelling
                    })
            {
                return false;
            }
            matches!(
                spelling,
                "(" | "{"
                    | ","
                    | ":"
                    | "="
                    | "!"
                    | "&"
                    | "|"
                    | "+"
                    | "-"
                    | "*"
                    | "%"
                    | "^"
                    | "~"
                    | "<"
                    | ">"
            )
        }
        TokenKind::StringLiteral(_)
        | TokenKind::OpaqueLiteral
        | TokenKind::Dot
        | TokenKind::CloseBracket => false,
    }
}

fn lex_regex(source: &str, position: &mut usize, tokens: &mut Vec<Token>) -> bool {
    let bytes = source.as_bytes();
    let start = *position;
    *position += 1;
    let mut in_character_class = false;
    while *position < bytes.len() {
        match bytes[*position] {
            b'\\' => *position = (*position + 2).min(bytes.len()),
            b'[' if !in_character_class => {
                in_character_class = true;
                *position += 1;
            }
            b']' if in_character_class => {
                in_character_class = false;
                *position += 1;
            }
            b'/' if !in_character_class => {
                *position += 1;
                while *position < bytes.len() {
                    let Some(flag) = source[*position..].chars().next() else {
                        break;
                    };
                    if !is_identifier_continue(flag) {
                        break;
                    }
                    *position += flag.len_utf8();
                }
                push_token(tokens, TokenKind::OpaqueLiteral, start, *position);
                return true;
            }
            b'\n' | b'\r' => {
                *position = start + 1;
                push_token(tokens, TokenKind::Other, start, *position);
                return false;
            }
            _ => *position += 1,
        }
    }
    *position = start + 1;
    push_token(tokens, TokenKind::Other, start, *position);
    false
}

fn can_start_markup(tokens: &[Token], source: &str, position: usize) -> bool {
    let line_start = source[..position].rfind('\n').map_or(0, |index| index + 1);
    if source[line_start..position].trim().is_empty() {
        return true;
    }
    let Some(previous) = tokens.last() else {
        return true;
    };
    match &previous.kind {
        TokenKind::Identifier(name) => matches!(name.as_str(), "return" | "yield"),
        TokenKind::OpenBracket | TokenKind::Question | TokenKind::Semicolon => true,
        TokenKind::Other => {
            let spelling = &source[previous.start..previous.end];
            matches!(spelling, "(" | "{" | "," | ":" | "=" | "!" | "&" | "|")
                || (spelling == ">"
                    && tokens
                        .get(tokens.len().saturating_sub(2))
                        .is_some_and(|before| {
                            matches!(before.kind, TokenKind::Other)
                                && &source[before.start..before.end] == "="
                        }))
        }
        TokenKind::StringLiteral(_)
        | TokenKind::OpaqueLiteral
        | TokenKind::Dot
        | TokenKind::CloseBracket => false,
    }
}

fn lex_markup(
    source: &str,
    position: &mut usize,
    tokens: &mut Vec<Token>,
    uncertain_syntax: &mut bool,
) -> bool {
    if source[*position..].starts_with("<!--") {
        if let Some(end) = source[*position + 4..].find("-->") {
            *position += 4 + end + 3;
        } else {
            *position = source.len();
            *uncertain_syntax = true;
        }
        return true;
    }
    if source[*position..].starts_with("<!") {
        if let Some(end) = source[*position + 2..].find('>') {
            *position += 2 + end + 1;
        } else {
            *position = source.len();
            *uncertain_syntax = true;
        }
        return true;
    }

    let Some((name, after_name)) = markup_tag_name(source, *position) else {
        return false;
    };
    let self_closing = markup_opening_is_self_closing(source, after_name);
    if !self_closing
        && !is_void_markup_element(&name)
        && find_closing_tag(source, after_name, &name).is_none()
    {
        return false;
    }

    *position = after_name;
    if !lex_markup_opening(source, position, tokens, uncertain_syntax) {
        return true;
    }
    if self_closing || is_void_markup_element(&name) {
        return true;
    }

    if name.eq_ignore_ascii_case("script") || name.eq_ignore_ascii_case("style") {
        let Some(close) = find_closing_tag(source, *position, &name) else {
            *position = source.len();
            *uncertain_syntax = true;
            return true;
        };
        if name.eq_ignore_ascii_case("script") {
            lex_code_fragment(source, *position, close, tokens, uncertain_syntax);
        }
        *position = close;
        consume_closing_tag(source, position);
        return true;
    }

    while *position < source.len() {
        if closing_tag_matches(source, *position, &name) {
            consume_closing_tag(source, position);
            return true;
        }
        if source[*position..].starts_with("<!--")
            && lex_markup(source, position, tokens, uncertain_syntax)
        {
            continue;
        }
        if source.as_bytes()[*position] == b'<'
            && lex_markup(source, position, tokens, uncertain_syntax)
        {
            continue;
        }
        if source.as_bytes()[*position] == b'{' {
            if lex_svelte_block_close(source, position) {
                continue;
            }
            *position += 1;
            lex_code(source, position, true, tokens, uncertain_syntax);
            continue;
        }
        let Some(character) = source[*position..].chars().next() else {
            break;
        };
        *position += character.len_utf8();
    }
    *uncertain_syntax = true;
    true
}

fn markup_tag_name(source: &str, start: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    if bytes.get(start) != Some(&b'<') || bytes.get(start + 1) == Some(&b'/') {
        return None;
    }
    if bytes.get(start + 1) == Some(&b'>') {
        return Some((String::new(), start + 1));
    }
    let first = source[start + 1..].chars().next()?;
    if !is_identifier_start(first) || first == '$' {
        return None;
    }
    let mut end = start + 1 + first.len_utf8();
    while end < source.len() {
        let character = source[end..].chars().next()?;
        if !(is_identifier_continue(character) || matches!(character, '-' | ':' | '.')) {
            break;
        }
        end += character.len_utf8();
    }
    let boundary = source.as_bytes().get(end).copied()?;
    if !boundary.is_ascii_whitespace() && !matches!(boundary, b'/' | b'>' | b'{') {
        return None;
    }
    Some((source[start + 1..end].to_owned(), end))
}

fn markup_opening_is_self_closing(source: &str, mut position: usize) -> bool {
    let bytes = source.as_bytes();
    let mut quote = None;
    let mut brace_depth = 0_usize;
    while position < bytes.len() {
        let byte = bytes[position];
        if let Some(active_quote) = quote {
            if byte == b'\\' {
                position = (position + 2).min(bytes.len());
            } else {
                position += 1;
                if byte == active_quote {
                    quote = None;
                }
            }
            continue;
        }
        match byte {
            b'\'' | b'"' => {
                quote = Some(byte);
                position += 1;
            }
            b'{' => {
                brace_depth += 1;
                position += 1;
            }
            b'}' => {
                brace_depth = brace_depth.saturating_sub(1);
                position += 1;
            }
            b'>' if brace_depth == 0 => {
                return source[..position].trim_end().ends_with('/');
            }
            _ => position += 1,
        }
    }
    false
}

fn lex_markup_opening(
    source: &str,
    position: &mut usize,
    tokens: &mut Vec<Token>,
    uncertain_syntax: &mut bool,
) -> bool {
    let bytes = source.as_bytes();
    while *position < bytes.len() {
        match bytes[*position] {
            b'\'' | b'"' => {
                let quote = bytes[*position];
                let expression = quoted_markup_attribute_is_expression(source, *position);
                *position += 1;
                let content_start = *position;
                while *position < bytes.len() && bytes[*position] != quote {
                    *position += 1;
                }
                if *position == bytes.len() {
                    *uncertain_syntax = true;
                    return false;
                }
                if expression {
                    lex_code_fragment(source, content_start, *position, tokens, uncertain_syntax);
                }
                *position += 1;
            }
            b'{' => {
                *position += 1;
                lex_code(source, position, true, tokens, uncertain_syntax);
            }
            b'>' => {
                *position += 1;
                return true;
            }
            _ => *position += 1,
        }
    }
    *uncertain_syntax = true;
    false
}

fn quoted_markup_attribute_is_expression(source: &str, quote: usize) -> bool {
    let bytes = source.as_bytes();
    let mut position = quote;
    while position > 0 && bytes[position - 1].is_ascii_whitespace() {
        position -= 1;
    }
    if position == 0 || bytes[position - 1] != b'=' {
        return false;
    }
    position -= 1;
    while position > 0 && bytes[position - 1].is_ascii_whitespace() {
        position -= 1;
    }
    let end = position;
    while position > 0
        && !bytes[position - 1].is_ascii_whitespace()
        && !matches!(bytes[position - 1], b'<' | b'>')
    {
        position -= 1;
    }
    let name = &source[position..end];
    name.starts_with(['@', ':', '#']) || name.starts_with("v-")
}

fn lex_svelte_block_close(source: &str, position: &mut usize) -> bool {
    let Some(rest) = source.get(*position..) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix("{/") else {
        return false;
    };
    let name_end = rest
        .find(|character: char| !is_identifier_continue(character))
        .unwrap_or(rest.len());
    let name = &rest[..name_end];
    if !matches!(name, "await" | "each" | "if" | "key" | "snippet") {
        return false;
    }
    let Some(close) = rest[name_end..].find('}') else {
        return false;
    };
    if !rest[name_end..name_end + close].trim().is_empty() {
        return false;
    }
    *position += 2 + name_end + close + 1;
    true
}

fn lex_code_fragment(
    source: &str,
    start: usize,
    end: usize,
    tokens: &mut Vec<Token>,
    uncertain_syntax: &mut bool,
) {
    let LexedSource {
        tokens: mut fragment,
        uncertain_syntax: fragment_uncertain,
    } = lex(&source[start..end]);
    for token in &mut fragment {
        token.start += start;
        token.end += start;
    }
    tokens.extend(fragment);
    *uncertain_syntax |= fragment_uncertain;
}

fn find_closing_tag(source: &str, mut position: usize, name: &str) -> Option<usize> {
    while let Some(relative) = source[position..].find("</") {
        let candidate = position + relative;
        if closing_tag_matches(source, candidate, name) {
            return Some(candidate);
        }
        position = candidate + 2;
    }
    None
}

fn closing_tag_matches(source: &str, position: usize, name: &str) -> bool {
    let Some(rest) = source.get(position + 2..) else {
        return false;
    };
    if name.is_empty() {
        return rest.starts_with('>');
    }
    let Some(candidate) = rest.get(..name.len()) else {
        return false;
    };
    candidate.eq_ignore_ascii_case(name)
        && rest
            .as_bytes()
            .get(name.len())
            .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b'>')
}

fn consume_closing_tag(source: &str, position: &mut usize) {
    if let Some(end) = source[*position..].find('>') {
        *position += end + 1;
    } else {
        *position = source.len();
    }
}

fn is_void_markup_element(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn lex_string(source: &str, position: &mut usize, tokens: &mut Vec<Token>) -> bool {
    let bytes = source.as_bytes();
    let start = *position;
    let quote = bytes[start];
    *position += 1;
    let content_start = *position;
    let mut simple = true;
    while *position < bytes.len() {
        match bytes[*position] {
            b'\\' => {
                simple = false;
                *position = (*position + 2).min(bytes.len());
            }
            byte if byte == quote => {
                let value = simple.then(|| source[content_start..*position].to_owned());
                *position += 1;
                push_token(tokens, TokenKind::StringLiteral(value), start, *position);
                return true;
            }
            b'\n' | b'\r' => {
                push_token(tokens, TokenKind::StringLiteral(None), start, *position);
                return false;
            }
            _ => *position += 1,
        }
    }
    push_token(tokens, TokenKind::StringLiteral(None), start, *position);
    false
}

fn lex_template(
    source: &str,
    position: &mut usize,
    tokens: &mut Vec<Token>,
    uncertain_syntax: &mut bool,
) -> bool {
    let bytes = source.as_bytes();
    let start = *position;
    *position += 1;
    while *position < bytes.len() {
        match bytes[*position] {
            b'\\' => *position = (*position + 2).min(bytes.len()),
            b'`' => {
                *position += 1;
                push_token(tokens, TokenKind::OpaqueLiteral, start, *position);
                return true;
            }
            b'$' if bytes.get(*position + 1) == Some(&b'{') => {
                *position += 2;
                lex_code(source, position, true, tokens, uncertain_syntax);
            }
            _ => *position += 1,
        }
    }
    push_token(tokens, TokenKind::OpaqueLiteral, start, *position);
    false
}

fn push_token(tokens: &mut Vec<Token>, kind: TokenKind, start: usize, end: usize) {
    tokens.push(Token { kind, start, end });
}

fn is_identifier_start(character: char) -> bool {
    matches!(character, '_' | '$') || character.is_alphabetic()
}

fn is_identifier_continue(character: char) -> bool {
    is_identifier_start(character) || character.is_alphanumeric()
}

fn imported_symbols(
    tokens: &[Token],
    source: &str,
) -> (BTreeSet<usize>, BTreeSet<String>, BTreeSet<String>) {
    let mut imported = BTreeSet::new();
    let mut roots = BTreeSet::from(["l".to_owned(), "lgl".to_owned(), "messages".to_owned()]);
    let mut factories = BTreeSet::from([
        "configureLinguini".to_owned(),
        "createLinguini".to_owned(),
        "createLinguiniProvider".to_owned(),
    ]);
    let mut index = 0;
    while index < tokens.len() {
        if !matches!(&tokens[index].kind, TokenKind::Identifier(name) if name == "import") {
            index += 1;
            continue;
        }
        if !is_top_level(tokens, source, index)
            || is_member_property(tokens, index)
            || tokens.get(index + 1).is_some_and(|token| {
                matches!(token.kind, TokenKind::Other) && &source[token.start..token.end] == "("
            })
            || matches!(
                tokens.get(index + 1).map(|token| &token.kind),
                Some(TokenKind::Dot)
            )
        {
            index += 1;
            continue;
        }

        let end = import_end(tokens, source, index);
        let side_effect = matches!(
            tokens.get(index + 1).map(|token| &token.kind),
            Some(TokenKind::StringLiteral(_))
        );
        let has_from_clause = tokens[index + 1..end]
            .iter()
            .enumerate()
            .any(|(offset, token)| {
                matches!(&token.kind, TokenKind::Identifier(name) if name == "from")
                    && tokens[index + offset + 2..end]
                        .iter()
                        .any(|token| matches!(token.kind, TokenKind::StringLiteral(_)))
            });
        if !side_effect && !has_from_clause {
            index += 1;
            continue;
        }
        for imported_index in index..end {
            imported.insert(imported_index);
        }
        for alias_index in index..end.saturating_sub(2) {
            let TokenKind::Identifier(imported_name) = &tokens[alias_index].kind else {
                continue;
            };
            if matches!(
                &tokens[alias_index + 1].kind,
                TokenKind::Identifier(name) if name == "as"
            ) {
                if let TokenKind::Identifier(alias) = &tokens[alias_index + 2].kind {
                    if roots.contains(imported_name) {
                        roots.insert(alias.clone());
                    }
                    if factories.contains(imported_name) {
                        factories.insert(alias.clone());
                    }
                }
            }
        }
        index = end.max(index + 1);
    }
    (imported, roots, factories)
}

fn is_top_level(tokens: &[Token], source: &str, end: usize) -> bool {
    let mut parentheses = 0_i32;
    let mut braces = 0_i32;
    let mut brackets = 0_i32;
    for token in &tokens[..end] {
        match &token.kind {
            TokenKind::OpenBracket => brackets += 1,
            TokenKind::CloseBracket => brackets -= 1,
            TokenKind::Other => match &source[token.start..token.end] {
                "(" => parentheses += 1,
                ")" => parentheses -= 1,
                "{" => braces += 1,
                "}" => braces -= 1,
                _ => {}
            },
            _ => {}
        }
    }
    parentheses == 0 && braces == 0 && brackets == 0
}

fn call_end(tokens: &[Token], source: &str, start: usize) -> Option<usize> {
    let first = tokens.get(start)?;
    if !matches!(first.kind, TokenKind::Other) || &source[first.start..first.end] != "(" {
        return None;
    }
    let mut depth = 0_usize;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        if !matches!(token.kind, TokenKind::Other) {
            continue;
        }
        match &source[token.start..token.end] {
            "(" => depth += 1,
            ")" => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
    }
    None
}

fn declaration_binding(tokens: &[Token], source: &str, factory: usize) -> Option<(String, bool)> {
    let equals = factory.checked_sub(1)?;
    let equals_token = tokens.get(equals)?;
    if !matches!(equals_token.kind, TokenKind::Other)
        || &source[equals_token.start..equals_token.end] != "="
    {
        return None;
    }

    let statement_start = tokens[..equals]
        .iter()
        .rposition(|token| matches!(token.kind, TokenKind::Semicolon))
        .map_or(0, |index| index + 1);
    for declaration in (statement_start..equals).rev() {
        if !matches!(
            &tokens[declaration].kind,
            TokenKind::Identifier(name) if matches!(name.as_str(), "const" | "let" | "var")
        ) {
            continue;
        }
        if tokens[declaration + 1..equals].iter().any(|token| {
            matches!(token.kind, TokenKind::Other) && &source[token.start..token.end] == "="
        }) {
            return None;
        }
        let TokenKind::Identifier(binding) = &tokens.get(declaration + 1)?.kind else {
            return None;
        };
        let exported = tokens[statement_start..declaration]
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Identifier(name) if name == "export"));
        return Some((binding.clone(), exported));
    }
    None
}

fn is_binding_declaration(tokens: &[Token], source: &str, index: usize) -> bool {
    if matches!(
        index.checked_sub(1).and_then(|index| tokens.get(index)),
        Some(Token {
            kind: TokenKind::Identifier(name),
            ..
        }) if matches!(name.as_str(), "const" | "let" | "var")
    ) {
        return true;
    }
    tokens.get(index + 1).is_some_and(|token| {
        matches!(token.kind, TokenKind::Other)
            && matches!(&source[token.start..token.end], "=" | ":")
    })
}

fn is_invocation(tokens: &[Token], source: &str, index: usize) -> bool {
    let Some(token) = tokens.get(index) else {
        return false;
    };
    if matches!(token.kind, TokenKind::Other) && &source[token.start..token.end] == "(" {
        return true;
    }
    matches!(
        tokens.get(index..index + 2),
        Some([
            Token {
                kind: TokenKind::Question,
                ..
            },
            Token {
                kind: TokenKind::Other,
                start,
                end,
            }
        ]) if &source[*start..*end] == "("
    )
}

fn import_end(tokens: &[Token], source: &str, start: usize) -> usize {
    let mut saw_module = false;
    for index in start + 1..tokens.len() {
        if matches!(tokens[index].kind, TokenKind::StringLiteral(_)) {
            saw_module = true;
        }
        if matches!(tokens[index].kind, TokenKind::Semicolon) {
            return index + 1;
        }
        if saw_module
            && tokens
                .get(index + 1)
                .is_some_and(|next| source[tokens[index].end..next.start].contains('\n'))
        {
            return index + 1;
        }
    }
    tokens.len()
}

fn is_member_property(tokens: &[Token], index: usize) -> bool {
    matches!(
        index.checked_sub(1).and_then(|index| tokens.get(index)),
        Some(Token {
            kind: TokenKind::Dot,
            ..
        })
    ) || matches!(
        index.checked_sub(2).map(|index| &tokens[index..index + 2]),
        Some([
            Token {
                kind: TokenKind::Question,
                ..
            },
            Token {
                kind: TokenKind::Dot,
                ..
            }
        ])
    )
}

fn member_path(tokens: &[Token], mut index: usize) -> (Vec<UsageSegment>, bool, usize) {
    let mut segments = Vec::new();
    loop {
        let member_start = if matches!(
            tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Dot)
        ) {
            Some(index + 1)
        } else if matches!(
            tokens.get(index..index + 2),
            Some([
                Token {
                    kind: TokenKind::Question,
                    ..
                },
                Token {
                    kind: TokenKind::Dot,
                    ..
                }
            ])
        ) {
            Some(index + 2)
        } else {
            None
        };
        if let Some(member) = member_start {
            if matches!(
                tokens.get(member).map(|token| &token.kind),
                Some(TokenKind::OpenBracket)
            ) {
                index = member;
            } else if let Some(Token {
                kind: TokenKind::Identifier(name),
                ..
            }) = tokens.get(member)
            {
                segments.push(UsageSegment::new(name.clone()));
                index = member + 1;
                continue;
            } else {
                return (segments, false, index);
            }
        }

        if !matches!(
            tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::OpenBracket)
        ) {
            return (segments, false, index);
        }
        match tokens.get(index + 1).map(|token| &token.kind) {
            Some(TokenKind::StringLiteral(Some(name)))
                if matches!(
                    tokens.get(index + 2).map(|token| &token.kind),
                    Some(TokenKind::CloseBracket)
                ) =>
            {
                segments.push(UsageSegment::new(name.clone()));
                index += 3;
            }
            _ => return (segments, true, index + 1),
        }
    }
}

fn decode_generated_identifier(name: &str) -> Option<String> {
    let encoded = name.strip_prefix("__lgl_name_")?;
    if encoded.len() % 2 != 0 {
        return None;
    }
    let bytes = encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(pair, 16).ok()
        })
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}
