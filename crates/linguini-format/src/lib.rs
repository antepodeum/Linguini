use linguini_syntax::{
    lex_schema_with_recovery, lex_with_recovery, parse_locale_with_recovery,
    parse_schema_with_recovery, ParseError, Span, Token, TokenKind, LOCALE_EXTENSION,
    SCHEMA_EXTENSION,
};
use std::borrow::Cow;
use std::fmt;
use std::path::Path;

pub const CRATE_PURPOSE: &str = "Linguini source formatting";

const ARROW_MARKER_START: char = '\u{E000}';
const ARROW_MARKER_END: char = '\u{E001}';

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SourceKind {
    Schema,
    Locale,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormatOptions {
    pub indent_width: usize,
    pub max_line_width: usize,
    pub sort_enum_variants: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormatError {
    pub errors: Vec<ParseError>,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(first) = self.errors.first() else {
            return f.write_str("formatting failed");
        };
        write!(f, "cannot format invalid source: {}", first.message)
    }
}

impl std::error::Error for FormatError {}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent_width: 2,
            max_line_width: 100,
            sort_enum_variants: false,
        }
    }
}

impl SourceKind {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|extension| extension.to_str()) {
            Some(SCHEMA_EXTENSION) => Some(Self::Schema),
            Some(LOCALE_EXTENSION) => Some(Self::Locale),
            _ => None,
        }
    }
}

pub fn format_path_source(path: &Path, source: &str) -> Result<String, FormatError> {
    let kind = SourceKind::from_path(path).unwrap_or(SourceKind::Locale);
    format_source(kind, source, &FormatOptions::default())
}

pub fn format_source(
    kind: SourceKind,
    source: &str,
    options: &FormatOptions,
) -> Result<String, FormatError> {
    validate_source(kind, source)?;
    let tokens = match kind {
        SourceKind::Schema => lex_schema_with_recovery(source).tokens,
        SourceKind::Locale => lex_with_recovery(source).tokens,
    };

    Ok(render_tokens(source, &tokens, options))
}

fn validate_source(kind: SourceKind, source: &str) -> Result<(), FormatError> {
    let errors = match kind {
        SourceKind::Schema => parse_schema_with_recovery(source).errors,
        SourceKind::Locale => parse_locale_with_recovery(source).errors,
    };

    if errors.is_empty() {
        Ok(())
    } else {
        Err(FormatError { errors })
    }
}

fn render_tokens(source: &str, tokens: &[Token], options: &FormatOptions) -> String {
    let mut out = String::new();
    let mut indent = 0usize;
    let mut at_line_start = true;
    let mut previous: Option<&TokenKind> = None;
    let mut pending_space = false;
    let mut brace_stack = Vec::new();

    for token in tokens {
        match &token.kind {
            TokenKind::Whitespace => {
                pending_space = !at_line_start;
            }
            TokenKind::Newline => {
                push_newline(&mut out);
                at_line_start = true;
                pending_space = false;
            }
            TokenKind::Comment(text) => {
                push_indent(&mut out, indent, options, &mut at_line_start);
                if !at_line_start && pending_space {
                    out.push(' ');
                }
                out.push_str("//");
                out.push_str(text.trim_end());
                pending_space = false;
                at_line_start = false;
            }
            TokenKind::DocComment(text) => {
                push_indent(&mut out, indent, options, &mut at_line_start);
                out.push_str("///");
                out.push_str(text.trim_end());
                pending_space = false;
                at_line_start = false;
            }
            TokenKind::RBrace => {
                let placeholder_brace = brace_stack.pop().unwrap_or(false);
                if !placeholder_brace {
                    indent = indent.saturating_sub(1);
                }
                push_token_text(
                    source,
                    token,
                    &mut out,
                    indent,
                    options,
                    &mut at_line_start,
                    &mut pending_space,
                    previous,
                    placeholder_brace,
                );
            }
            TokenKind::LBrace => {
                let placeholder_brace = matches!(
                    previous,
                    Some(TokenKind::Equals | TokenKind::Arrow | TokenKind::RawText(_))
                );
                push_token_text(
                    source,
                    token,
                    &mut out,
                    indent,
                    options,
                    &mut at_line_start,
                    &mut pending_space,
                    previous,
                    placeholder_brace,
                );
                brace_stack.push(placeholder_brace);
                if !placeholder_brace {
                    indent += 1;
                }
            }
            _ => push_token_text(
                source,
                token,
                &mut out,
                indent,
                options,
                &mut at_line_start,
                &mut pending_space,
                previous,
                *brace_stack.last().unwrap_or(&false),
            ),
        }

        if !matches!(token.kind, TokenKind::Whitespace | TokenKind::Newline) {
            previous = Some(&token.kind);
        }
    }

    trim_trailing_blank_lines(&mut out);
    out = align_marked_match_arms(&out);
    out.push('\n');
    out
}

fn push_token_text(
    source: &str,
    token: &Token,
    out: &mut String,
    indent: usize,
    options: &FormatOptions,
    at_line_start: &mut bool,
    pending_space: &mut bool,
    previous: Option<&TokenKind>,
    placeholder_brace: bool,
) {
    let source_text = token_source(source, token);
    let text = rendered_token_text(source_text, &token.kind);
    if text.is_empty() {
        return;
    }

    if should_preserve_line_start(&token.kind) && *at_line_start {
        *at_line_start = false;
    } else {
        push_indent(out, indent, options, at_line_start);
    }

    if should_space_before(
        previous,
        &token.kind,
        text.as_ref(),
        *pending_space,
        placeholder_brace,
    ) && !out.ends_with([' ', '\n'])
    {
        out.push(' ');
    }

    if matches!(token.kind, TokenKind::Arrow) {
        out.push(ARROW_MARKER_START);
        out.push_str(text.as_ref());
        out.push(ARROW_MARKER_END);
    } else {
        out.push_str(text.as_ref());
    }

    *pending_space = false;
    *at_line_start = false;
}

fn rendered_token_text<'a>(source_text: &'a str, kind: &TokenKind) -> Cow<'a, str> {
    if should_preserve_token_source(kind) {
        Cow::Borrowed(source_text)
    } else {
        Cow::Borrowed(source_text.trim())
    }
}

fn should_preserve_token_source(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::RawText(_) | TokenKind::String(_) | TokenKind::TripleQuote
    )
}

fn should_preserve_line_start(kind: &TokenKind) -> bool {
    matches!(kind, TokenKind::RawText(_) | TokenKind::TripleQuote)
}

fn token_source<'a>(source: &'a str, token: &Token) -> &'a str {
    source
        .get(span_range(token.span))
        .unwrap_or_default()
}

fn span_range(span: Span) -> std::ops::Range<usize> {
    span.start..span.end
}

fn push_indent(
    out: &mut String,
    indent: usize,
    options: &FormatOptions,
    at_line_start: &mut bool,
) {
    if *at_line_start {
        out.push_str(&" ".repeat(indent * options.indent_width));
        *at_line_start = false;
    }
}

fn push_newline(out: &mut String) {
    while out.ends_with(' ') {
        out.pop();
    }
    out.push('\n');
}

fn trim_trailing_blank_lines(out: &mut String) {
    while out.ends_with('\n') {
        out.pop();
    }
}

fn should_space_before(
    previous: Option<&TokenKind>,
    current: &TokenKind,
    current_text: &str,
    pending: bool,
    placeholder_brace: bool,
) -> bool {
    let Some(previous) = previous else {
        return false;
    };

    if matches!(
        current,
        TokenKind::Comma | TokenKind::Colon | TokenKind::LParen | TokenKind::RParen | TokenKind::Dot
    ) || matches!(previous, TokenKind::LParen | TokenKind::Dot | TokenKind::At)
    {
        return false;
    }

    if current_text.chars().next().is_some_and(char::is_whitespace) {
        return false;
    }

    if placeholder_brace
        && (matches!(previous, TokenKind::LBrace) || matches!(current, TokenKind::RBrace))
    {
        return false;
    }

    if matches!(previous, TokenKind::RBrace) && matches!(current, TokenKind::RawText(_)) {
        return true;
    }

    if matches!(previous, TokenKind::LBrace) || matches!(current, TokenKind::RBrace) {
        return true;
    }

    if matches!(previous, TokenKind::Comma | TokenKind::Colon | TokenKind::Equals | TokenKind::Arrow)
    {
        return true;
    }

    pending
        || matches!(current, TokenKind::LBrace | TokenKind::Arrow | TokenKind::Equals)
        || matches!(previous, TokenKind::Ident(_) | TokenKind::LocaleTag(_) | TokenKind::String(_))
            && matches!(
                current,
                TokenKind::Ident(_)
                    | TokenKind::LocaleTag(_)
                    | TokenKind::String(_)
                    | TokenKind::RawText(_)
            )
}

fn align_marked_match_arms(input: &str) -> String {
    let mut output = Vec::new();
    let mut group = Vec::new();

    for line in input.lines() {
        if line.contains(ARROW_MARKER_START) {
            group.push(line.to_owned());
        } else {
            flush_arm_group(&mut output, &mut group);
            output.push(remove_arrow_markers(line));
        }
    }

    flush_arm_group(&mut output, &mut group);
    output.join("\n")
}

fn flush_arm_group(output: &mut Vec<String>, group: &mut Vec<String>) {
    if group.is_empty() {
        return;
    }

    let max_before_width = group
        .iter()
        .filter_map(|line| marked_arrow_bounds(line).map(|(start, _)| display_width(line[..start].trim_end())))
        .max()
        .unwrap_or(0);

    output.extend(
        group
            .drain(..)
            .map(|line| align_marked_arm_line(&line, max_before_width)),
    );
}

fn align_marked_arm_line(line: &str, max_before_width: usize) -> String {
    let Some((start, end)) = marked_arrow_bounds(line) else {
        return remove_arrow_markers(line);
    };

    let before = line[..start].trim_end();
    let after = line[end..].trim_start();
    let padding = max_before_width.saturating_sub(display_width(before)) + 1;

    let mut aligned = String::new();
    aligned.push_str(before);
    aligned.push_str(&" ".repeat(padding));
    aligned.push_str("=>");
    if !after.is_empty() {
        aligned.push(' ');
        aligned.push_str(after);
    }
    aligned
}

fn marked_arrow_bounds(line: &str) -> Option<(usize, usize)> {
    let start = line.find(ARROW_MARKER_START)?;
    let arrow_start = start + ARROW_MARKER_START.len_utf8();
    let marker_end = line[arrow_start..].find(ARROW_MARKER_END)? + arrow_start;
    Some((start, marker_end + ARROW_MARKER_END.len_utf8()))
}

fn remove_arrow_markers(line: &str) -> String {
    line.replace(ARROW_MARKER_START, "")
        .replace(ARROW_MARKER_END, "")
}

fn display_width(text: &str) -> usize {
    text.chars().count()
}

#[cfg(test)]
mod tests;
