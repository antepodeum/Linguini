use std::collections::BTreeSet;

use crate::{SourceId, Span, Token, TokenKind};

const MAX_NESTING_DEPTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub errors: Vec<LexError>,
}

impl LexError {
    fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Code,
    SingleLineText,
    Multiline(MultilineMode),
    Placeholder(ResumeMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MultilineMode {
    Dedented,
    Raw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResumeMode {
    SingleLineText,
    Multiline(MultilineMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignificantToken {
    Identifier,
    Other,
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    lex_in(source, SourceId::default())
}

pub fn lex_in(source: &str, source_id: SourceId) -> Result<Vec<Token>, LexError> {
    let output = lex_with_recovery_in(source, source_id);
    match output.errors.into_iter().next() {
        Some(error) => Err(error),
        None => Ok(output.tokens),
    }
}

pub fn lex_with_recovery(source: &str) -> LexOutput {
    lex_with_recovery_in(source, SourceId::default())
}

pub fn lex_with_recovery_in(source: &str, source_id: SourceId) -> LexOutput {
    Lexer::new(source, source_id).lex()
}

pub fn lex_schema(source: &str) -> Result<Vec<Token>, LexError> {
    lex_schema_in(source, SourceId::default())
}

pub fn lex_schema_in(source: &str, source_id: SourceId) -> Result<Vec<Token>, LexError> {
    let output = lex_schema_with_recovery_in(source, source_id);
    match output.errors.into_iter().next() {
        Some(error) => Err(error),
        None => Ok(output.tokens),
    }
}

pub fn lex_schema_with_recovery(source: &str) -> LexOutput {
    lex_schema_with_recovery_in(source, SourceId::default())
}

pub fn lex_schema_with_recovery_in(source: &str, source_id: SourceId) -> LexOutput {
    Lexer::new(source, source_id)
        .with_text_assignments(false)
        .lex()
}

struct Lexer<'src> {
    source: &'src str,
    source_id: SourceId,
    offset: usize,
    mode: Mode,
    text_assignments: bool,
    paren_depth: usize,
    brace_depth: usize,
    last_significant: Option<SignificantToken>,
    tokens: Vec<Token>,
    errors: Vec<LexError>,
}

impl<'src> Lexer<'src> {
    fn new(source: &'src str, source_id: SourceId) -> Self {
        Self {
            source,
            source_id,
            offset: 0,
            mode: Mode::Code,
            text_assignments: true,
            paren_depth: 0,
            brace_depth: 0,
            last_significant: None,
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn with_text_assignments(mut self, text_assignments: bool) -> Self {
        self.text_assignments = text_assignments;
        self
    }

    fn lex(mut self) -> LexOutput {
        while self.offset < self.source.len() {
            let result = match self.mode {
                Mode::Code => self.next_code(),
                Mode::SingleLineText => self.next_single_line_text(),
                Mode::Multiline(mode) => self.next_multiline_text(mode),
                Mode::Placeholder(resume) => self.next_placeholder(resume),
            };

            match result {
                Ok(token) => self.tokens.push(token),
                Err(error) => self.recover(error),
            }
        }

        let eof = self.span(self.offset, self.offset);
        match self.mode {
            Mode::Code | Mode::SingleLineText => {}
            Mode::Multiline(_) => self
                .errors
                .push(LexError::new("unterminated multiline text", eof)),
            Mode::Placeholder(_) => self
                .errors
                .push(LexError::new("unterminated placeholder", eof)),
        }

        LexOutput {
            tokens: self.tokens,
            errors: self.errors,
        }
    }

    fn next_code(&mut self) -> Result<Token, LexError> {
        let start = self.offset;

        if let Some(end) = self.scan_newline(start) {
            self.offset = end;
            self.last_significant = None;
            return Ok(self.token(TokenKind::Newline, start, end));
        }
        if let Some(end) = self.scan_horizontal_whitespace(start) {
            self.offset = end;
            return Ok(self.token(TokenKind::Whitespace, start, end));
        }
        if self.rest().starts_with("///") {
            return Ok(self.scan_comment(true));
        }
        if self.rest().starts_with("//") {
            return Ok(self.scan_comment(false));
        }
        if self.rest().starts_with("=>") {
            self.offset += 2;
            self.mode = Mode::SingleLineText;
            self.last_significant = Some(SignificantToken::Other);
            return Ok(self.token(TokenKind::Arrow, start, self.offset));
        }
        if self.rest().starts_with("\"\"\"") {
            self.offset += 3;
            self.mode = Mode::Multiline(MultilineMode::Dedented);
            self.last_significant = Some(SignificantToken::Other);
            return Ok(self.token(TokenKind::TripleQuote, start, self.offset));
        }
        if self.current_char() == Some('"') {
            let token = self.scan_string_literal()?;
            self.last_significant = Some(SignificantToken::Other);
            return Ok(token);
        }
        if self.current_char().is_some_and(is_identifier_start) {
            let token = self.scan_identifier();
            self.last_significant = Some(SignificantToken::Identifier);
            return Ok(token);
        }

        let Some(ch) = self.current_char() else {
            return Err(LexError::new(
                "unexpected end of input",
                self.span(start, start),
            ));
        };
        let end = start + ch.len_utf8();
        let kind = match ch {
            '{' => {
                self.open_brace(start, end)?;
                TokenKind::LBrace
            }
            '}' => {
                self.close_brace();
                TokenKind::RBrace
            }
            '(' => {
                self.paren_depth += 1;
                TokenKind::LParen
            }
            ')' => {
                self.paren_depth = self.paren_depth.saturating_sub(1);
                TokenKind::RParen
            }
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            '=' => {
                if self.text_assignments
                    && self.paren_depth == 0
                    && self.last_significant == Some(SignificantToken::Identifier)
                {
                    self.mode = Mode::SingleLineText;
                }
                TokenKind::Equals
            }
            '.' => TokenKind::Dot,
            '@' => TokenKind::At,
            _ => {
                return Err(LexError::new(
                    format!("invalid token `{ch}`"),
                    self.span(start, end),
                ));
            }
        };

        self.offset = end;
        self.last_significant = Some(SignificantToken::Other);
        Ok(self.token(kind, start, end))
    }

    fn next_single_line_text(&mut self) -> Result<Token, LexError> {
        let start = self.offset;

        if let Some((end, kind, mode)) = self.scan_text_block_start(start) {
            self.offset = end;
            self.mode = Mode::Multiline(mode);
            return Ok(self.token(kind, start, end));
        }
        if let Some(end) = self.scan_newline(start) {
            self.offset = end;
            self.mode = Mode::Code;
            self.last_significant = None;
            return Ok(self.token(TokenKind::Newline, start, end));
        }
        if self.rest().starts_with("{{") {
            self.offset += 2;
            return Ok(self.token(TokenKind::RawText("{".to_owned()), start, self.offset));
        }
        if self.rest().starts_with("}}") {
            self.offset += 2;
            return Ok(self.token(TokenKind::RawText("}".to_owned()), start, self.offset));
        }
        if self.current_char() == Some('{') {
            let end = start + 1;
            self.open_brace(start, end)?;
            self.offset = end;
            self.mode = Mode::Placeholder(ResumeMode::SingleLineText);
            return Ok(self.token(TokenKind::LBrace, start, end));
        }
        if self.current_char() == Some('}') {
            let end = start + 1;
            self.close_brace();
            self.offset = end;
            self.mode = Mode::Code;
            self.last_significant = Some(SignificantToken::Other);
            return Ok(self.token(TokenKind::RBrace, start, end));
        }
        if self.current_char() == Some('"') {
            return self.scan_string_literal();
        }

        let end = self.scan_until(start, |rest, ch| {
            ch == '{'
                || ch == '}'
                || ch == '"'
                || ch == '\n'
                || ch == '\r'
                || rest.starts_with("raw\"\"\"")
        });
        if end == start {
            return self.invalid_current("invalid text token");
        }
        self.offset = end;
        Ok(self.token(
            TokenKind::RawText(self.source[start..end].to_owned()),
            start,
            end,
        ))
    }

    fn next_multiline_text(&mut self, mode: MultilineMode) -> Result<Token, LexError> {
        let start = self.offset;

        if self.rest().starts_with("\"\"\"") {
            self.offset += 3;
            self.mode = Mode::Code;
            self.last_significant = Some(SignificantToken::Other);
            return Ok(self.token(
                match mode {
                    MultilineMode::Dedented => TokenKind::TripleQuote,
                    MultilineMode::Raw => TokenKind::RawTripleQuote,
                },
                start,
                self.offset,
            ));
        }
        if self.rest().starts_with("{{") {
            self.offset += 2;
            return Ok(self.token(TokenKind::RawText("{".to_owned()), start, self.offset));
        }
        if self.rest().starts_with("}}") {
            self.offset += 2;
            return Ok(self.token(TokenKind::RawText("}".to_owned()), start, self.offset));
        }
        if self.current_char() == Some('{') {
            let end = start + 1;
            self.open_brace(start, end)?;
            self.offset = end;
            self.mode = Mode::Placeholder(ResumeMode::Multiline(mode));
            return Ok(self.token(TokenKind::LBrace, start, end));
        }
        if self.current_char() == Some('}') {
            return Err(LexError::new(
                "unescaped `}` in message text; write `}}` for a literal brace",
                self.span(start, start + 1),
            ));
        }
        if let Some(end) = self.scan_newline(start) {
            self.offset = end;
            return Ok(self.token(
                TokenKind::RawText(self.source[start..end].to_owned()),
                start,
                end,
            ));
        }

        let end = self.scan_until(start, |rest, ch| {
            ch == '{' || ch == '}' || ch == '\n' || ch == '\r' || rest.starts_with("\"\"\"")
        });
        if end == start {
            return self.invalid_current("invalid multiline text token");
        }
        self.offset = end;
        Ok(self.token(
            TokenKind::RawText(self.source[start..end].to_owned()),
            start,
            end,
        ))
    }

    fn next_placeholder(&mut self, resume: ResumeMode) -> Result<Token, LexError> {
        let start = self.offset;

        if let Some(end) = self.scan_newline(start) {
            self.offset = end;
            return Ok(self.token(TokenKind::Newline, start, end));
        }
        if let Some(end) = self.scan_horizontal_whitespace(start) {
            self.offset = end;
            return Ok(self.token(TokenKind::Whitespace, start, end));
        }
        if self.rest().starts_with("///") {
            return Ok(self.scan_comment(true));
        }
        if self.rest().starts_with("//") {
            return Ok(self.scan_comment(false));
        }
        if self.current_char() == Some('}') {
            let end = start + 1;
            self.close_brace();
            self.offset = end;
            self.mode = match resume {
                ResumeMode::SingleLineText => Mode::SingleLineText,
                ResumeMode::Multiline(mode) => Mode::Multiline(mode),
            };
            return Ok(self.token(TokenKind::RBrace, start, end));
        }
        if self.current_char() == Some('"') {
            return self.scan_string_literal();
        }
        if self.current_char().is_some_and(is_identifier_start) {
            return Ok(self.scan_identifier());
        }

        let Some(ch) = self.current_char() else {
            return Err(LexError::new(
                "unexpected end of input",
                self.span(start, start),
            ));
        };
        let end = start + ch.len_utf8();
        let kind = match ch {
            '{' => {
                return Err(LexError::new(
                    "nested `{` is not valid inside an expression",
                    self.span(start, end),
                ));
            }
            '(' => {
                self.paren_depth += 1;
                TokenKind::LParen
            }
            ')' => {
                self.paren_depth = self.paren_depth.saturating_sub(1);
                TokenKind::RParen
            }
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            '=' => TokenKind::Equals,
            '.' => TokenKind::Dot,
            '@' => TokenKind::At,
            _ => {
                return Err(LexError::new(
                    format!("invalid token `{ch}`"),
                    self.span(start, end),
                ));
            }
        };
        self.offset = end;
        Ok(self.token(kind, start, end))
    }

    fn scan_text_block_start(&self, start: usize) -> Option<(usize, TokenKind, MultilineMode)> {
        let prefix_end = self.scan_horizontal_whitespace(start).unwrap_or(start);
        let rest = &self.source[prefix_end..];
        if rest.starts_with("raw\"\"\"") {
            Some((
                prefix_end + 6,
                TokenKind::RawTripleQuote,
                MultilineMode::Raw,
            ))
        } else if rest.starts_with("\"\"\"") {
            Some((
                prefix_end + 3,
                TokenKind::TripleQuote,
                MultilineMode::Dedented,
            ))
        } else {
            None
        }
    }

    fn scan_comment(&mut self, docs: bool) -> Token {
        let start = self.offset;
        let prefix_len = if docs { 3 } else { 2 };
        let content_start = start + prefix_len;
        let end = self.scan_until(content_start, |_, ch| ch == '\n' || ch == '\r');
        self.offset = end;
        self.token(
            if docs {
                TokenKind::DocComment(self.source[content_start..end].to_owned())
            } else {
                TokenKind::Comment(self.source[content_start..end].to_owned())
            },
            start,
            end,
        )
    }

    fn scan_string_literal(&mut self) -> Result<Token, LexError> {
        let start = self.offset;
        let mut cursor = start + 1;
        let mut decoded = String::new();

        while cursor < self.source.len() {
            let Some(ch) = self.source[cursor..].chars().next() else {
                break;
            };
            if ch == '"' {
                let end = cursor + 1;
                self.offset = end;
                return Ok(self.token(TokenKind::String(decoded), start, end));
            }
            if ch == '\n' || ch == '\r' {
                return Err(LexError::new(
                    "unterminated string literal",
                    self.span(start, cursor),
                ));
            }
            if ch != '\\' {
                decoded.push(ch);
                cursor += ch.len_utf8();
                continue;
            }

            let escape_start = cursor;
            cursor += 1;
            let Some(escaped) = self.source[cursor..].chars().next() else {
                return Err(LexError::new(
                    "unterminated string escape",
                    self.span(escape_start, cursor),
                ));
            };
            cursor += escaped.len_utf8();
            match escaped {
                '"' => decoded.push('"'),
                '\\' => decoded.push('\\'),
                'n' => decoded.push('\n'),
                'r' => decoded.push('\r'),
                't' => decoded.push('\t'),
                '0' => decoded.push('\0'),
                'u' => {
                    let (value, end) = self.scan_unicode_escape(cursor, escape_start)?;
                    decoded.push(value);
                    cursor = end;
                }
                _ => {
                    return Err(LexError::new(
                        format!("unknown string escape `\\{escaped}`"),
                        self.span(escape_start, cursor),
                    ));
                }
            }
        }

        Err(LexError::new(
            "unterminated string literal",
            self.span(start, self.source.len()),
        ))
    }

    fn scan_unicode_escape(
        &self,
        cursor: usize,
        escape_start: usize,
    ) -> Result<(char, usize), LexError> {
        if !self.source[cursor..].starts_with('{') {
            return Err(LexError::new(
                "Unicode escape must use `\\u{HEX}`",
                self.span(escape_start, cursor),
            ));
        }
        let digits_start = cursor + 1;
        let Some(close_offset) = self.source[digits_start..].find('}') else {
            return Err(LexError::new(
                "unterminated Unicode escape",
                self.span(escape_start, self.source.len()),
            ));
        };
        let digits_end = digits_start + close_offset;
        let digits = &self.source[digits_start..digits_end];
        if digits.is_empty() || digits.len() > 6 || !digits.chars().all(|ch| ch.is_ascii_hexdigit())
        {
            return Err(LexError::new(
                "Unicode escape must contain one to six hexadecimal digits",
                self.span(escape_start, digits_end + 1),
            ));
        }
        let value = u32::from_str_radix(digits, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or_else(|| {
                LexError::new(
                    "Unicode escape is not a valid scalar value",
                    self.span(escape_start, digits_end + 1),
                )
            })?;
        Ok((value, digits_end + 1))
    }

    fn scan_identifier(&mut self) -> Token {
        let start = self.offset;
        let mut end = start;
        for (relative, ch) in self.source[start..].char_indices() {
            if relative == 0 {
                if !is_identifier_start(ch) {
                    break;
                }
            } else if !is_identifier_continue(ch) {
                break;
            }
            end = start + relative + ch.len_utf8();
        }
        self.offset = end;
        let value = &self.source[start..end];
        self.token(
            if is_locale_tag(value) {
                TokenKind::LocaleTag(value.to_owned())
            } else {
                TokenKind::Ident(value.to_owned())
            },
            start,
            end,
        )
    }

    fn open_brace(&mut self, start: usize, end: usize) -> Result<(), LexError> {
        if self.brace_depth >= MAX_NESTING_DEPTH {
            return Err(LexError::new(
                format!("nesting exceeds maximum depth of {MAX_NESTING_DEPTH}"),
                self.span(start, end),
            ));
        }
        self.brace_depth += 1;
        Ok(())
    }

    fn close_brace(&mut self) {
        self.brace_depth = self.brace_depth.saturating_sub(1);
    }

    fn recover(&mut self, error: LexError) {
        let start = self.offset;
        let Some(ch) = self.current_char() else {
            self.errors.push(error);
            return;
        };
        let end = start + ch.len_utf8();
        self.offset = end;
        self.errors.push(error);
        self.tokens.push(self.token(
            TokenKind::Error(self.source[start..end].to_owned()),
            start,
            end,
        ));
    }

    fn invalid_current<T>(&self, message: &str) -> Result<T, LexError> {
        let start = self.offset;
        let end = self
            .current_char()
            .map_or(start, |character| start + character.len_utf8());
        Err(LexError::new(message, self.span(start, end)))
    }

    fn scan_newline(&self, start: usize) -> Option<usize> {
        let rest = &self.source[start..];
        if rest.starts_with("\r\n") {
            Some(start + 2)
        } else if rest.starts_with('\n') || rest.starts_with('\r') {
            Some(start + 1)
        } else {
            None
        }
    }

    fn scan_horizontal_whitespace(&self, start: usize) -> Option<usize> {
        let mut end = start;
        for (relative, ch) in self.source[start..].char_indices() {
            if ch == '\n' || ch == '\r' || !ch.is_whitespace() {
                break;
            }
            end = start + relative + ch.len_utf8();
        }
        (end > start).then_some(end)
    }

    fn scan_until(&self, start: usize, stop: impl Fn(&str, char) -> bool) -> usize {
        let mut end = start;
        for (relative, ch) in self.source[start..].char_indices() {
            let absolute = start + relative;
            if stop(&self.source[absolute..], ch) {
                break;
            }
            end = absolute + ch.len_utf8();
        }
        end
    }

    fn rest(&self) -> &'src str {
        &self.source[self.offset..]
    }

    fn current_char(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn span(&self, start: usize, end: usize) -> Span {
        Span::in_source(self.source_id, start, end)
    }

    fn token(&self, kind: TokenKind, start: usize, end: usize) -> Token {
        Token::new(kind, self.span(start, end))
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch == '-' || ch.is_alphanumeric()
}

fn is_locale_tag(value: &str) -> bool {
    let subtags: Vec<_> = value.split('-').collect();
    if subtags.len() < 2 || subtags.iter().any(|part| part.is_empty()) {
        return false;
    }
    if subtags[0].eq_ignore_ascii_case("x") {
        return subtags[1..].iter().all(|part| {
            (1..=8).contains(&part.len()) && part.chars().all(|ch| ch.is_ascii_alphanumeric())
        });
    }
    if !(2..=8).contains(&subtags[0].len())
        || !subtags[0].chars().all(|ch| ch.is_ascii_alphabetic())
    {
        return false;
    }

    let mut index = 1;
    if subtags[0].len() <= 3 {
        let mut extlangs = 0;
        while index < subtags.len()
            && extlangs < 3
            && subtags[index].len() == 3
            && subtags[index].chars().all(|ch| ch.is_ascii_alphabetic())
        {
            index += 1;
            extlangs += 1;
        }
    }
    if subtags
        .get(index)
        .is_some_and(|part| part.len() == 4 && part.chars().all(|ch| ch.is_ascii_alphabetic()))
    {
        index += 1;
    }
    if subtags.get(index).is_some_and(|part| {
        (part.len() == 2 && part.chars().all(|ch| ch.is_ascii_alphabetic()))
            || (part.len() == 3 && part.chars().all(|ch| ch.is_ascii_digit()))
    }) {
        index += 1;
    }

    let mut variants = BTreeSet::new();
    while index < subtags.len()
        && ((5..=8).contains(&subtags[index].len())
            && subtags[index].chars().all(|ch| ch.is_ascii_alphanumeric())
            || (subtags[index].len() == 4
                && subtags[index]
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_ascii_digit())
                && subtags[index].chars().all(|ch| ch.is_ascii_alphanumeric())))
    {
        if !variants.insert(subtags[index].to_ascii_lowercase()) {
            return false;
        }
        index += 1;
    }

    let mut extensions = BTreeSet::new();
    while index < subtags.len()
        && subtags[index].len() == 1
        && subtags[index].chars().all(|ch| ch.is_ascii_alphanumeric())
        && !subtags[index].eq_ignore_ascii_case("x")
    {
        if !extensions.insert(subtags[index].to_ascii_lowercase()) {
            return false;
        }
        index += 1;
        let start = index;
        while index < subtags.len()
            && (2..=8).contains(&subtags[index].len())
            && subtags[index].chars().all(|ch| ch.is_ascii_alphanumeric())
        {
            index += 1;
        }
        if index == start {
            return false;
        }
    }

    if index < subtags.len() && subtags[index].eq_ignore_ascii_case("x") {
        index += 1;
        let start = index;
        while index < subtags.len()
            && (1..=8).contains(&subtags[index].len())
            && subtags[index].chars().all(|ch| ch.is_ascii_alphanumeric())
        {
            index += 1;
        }
        if index == start {
            return false;
        }
    }

    index == subtags.len()
}

#[cfg(test)]
mod locale_tag_tests {
    use super::is_locale_tag;

    #[test]
    fn recognizes_bcp47_shape() {
        for valid in [
            "en-US",
            "es-419",
            "zh-Hant-TW",
            "de-DE-1996",
            "en-US-u-ca-gregory",
            "x-private",
        ] {
            assert!(is_locale_tag(valid), "{valid}");
        }

        for invalid in [
            "en-",
            "e-US",
            "en-12",
            "en-US-u",
            "en-US-u-ca-u-nu",
            "de-1996-1996",
        ] {
            assert!(!is_locale_tag(invalid), "{invalid}");
        }
    }
}
