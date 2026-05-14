use linguini_syntax::{
    lex_schema_with_recovery, lex_with_recovery, parse_locale_with_recovery,
    parse_schema_with_recovery, ParseError, Span, Token, TokenKind, LOCALE_EXTENSION,
    SCHEMA_EXTENSION,
};
use std::fmt;
use std::path::Path;

pub const CRATE_PURPOSE: &str = "Linguini source formatting";

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
    let text = token_source(source, token);
    if text.is_empty() {
        return;
    }

    push_indent(out, indent, options, at_line_start);

    if should_space_before(previous, &token.kind, *pending_space, placeholder_brace)
        && !out.ends_with([' ', '\n'])
    {
        out.push(' ');
    }

    out.push_str(text.trim());
    *pending_space = false;
    *at_line_start = false;
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
    if !out.ends_with('\n') {
        out.push('\n');
    }
}

fn trim_trailing_blank_lines(out: &mut String) {
    while out.ends_with('\n') {
        out.pop();
    }
}

fn should_space_before(
    previous: Option<&TokenKind>,
    current: &TokenKind,
    pending: bool,
    placeholder_brace: bool,
) -> bool {
    let Some(previous) = previous else {
        return false;
    };

    if matches!(
        current,
        TokenKind::Comma | TokenKind::Colon | TokenKind::RParen | TokenKind::Dot
    ) || matches!(previous, TokenKind::LParen | TokenKind::Dot | TokenKind::At)
    {
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

#[cfg(test)]
mod tests;
