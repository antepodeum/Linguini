mod engine;
mod ir;

use linguini_syntax::{
    lex_schema_with_recovery, lex_with_recovery, parse_locale_with_recovery,
    parse_schema_with_recovery, ParseError, Span, LOCALE_EXTENSION, SCHEMA_EXTENSION,
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
    /// Maximum width for structural lines. `0` disables structural wrapping.
    ///
    /// Raw message text is never wrapped because doing so would change its value.
    pub max_line_width: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FormatError {
    Parse(Vec<ParseError>),
    UnsupportedExtension(String),
    InvalidOptions(String),
    InvalidTokenSpan(Span),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(errors) => {
                let Some(first) = errors.first() else {
                    return f.write_str("formatting failed");
                };
                write!(f, "cannot format invalid source: {}", first.message)
            }
            Self::UnsupportedExtension(extension) => {
                write!(f, "unsupported Linguini file extension `{extension}`")
            }
            Self::InvalidOptions(message) => write!(f, "invalid formatter options: {message}"),
            Self::InvalidTokenSpan(span) => write!(
                f,
                "lexer returned an invalid token span {}..{}",
                span.start, span.end
            ),
        }
    }
}

impl std::error::Error for FormatError {}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent_width: 2,
            max_line_width: 100,
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
    let kind = SourceKind::from_path(path).ok_or_else(|| {
        FormatError::UnsupportedExtension(
            path.extension()
                .and_then(|extension| extension.to_str())
                .unwrap_or("<non-UTF-8>")
                .to_owned(),
        )
    })?;
    format_source(kind, source, &FormatOptions::default())
}

pub fn format_source(
    kind: SourceKind,
    source: &str,
    options: &FormatOptions,
) -> Result<String, FormatError> {
    validate_options(options)?;
    validate_source(kind, source)?;
    let tokens = match kind {
        SourceKind::Schema => lex_schema_with_recovery(source).tokens,
        SourceKind::Locale => lex_with_recovery(source).tokens,
    };

    let newline = detect_newline(source);
    let formatted = engine::render_tokens(source, &tokens, options)?;
    if newline == "\r\n" {
        Ok(formatted.replace('\n', "\r\n"))
    } else {
        Ok(formatted)
    }
}

const MAX_INDENT_WIDTH: usize = 64;
const MAX_LINE_WIDTH: usize = 1_000_000;

fn validate_options(options: &FormatOptions) -> Result<(), FormatError> {
    if options.indent_width > MAX_INDENT_WIDTH {
        return Err(FormatError::InvalidOptions(format!(
            "indent_width must be at most {MAX_INDENT_WIDTH}"
        )));
    }
    if options.max_line_width > MAX_LINE_WIDTH {
        return Err(FormatError::InvalidOptions(format!(
            "max_line_width must be 0 or at most {MAX_LINE_WIDTH}"
        )));
    }
    Ok(())
}

fn detect_newline(source: &str) -> &'static str {
    let bytes = source.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            return if index > 0 && bytes[index - 1] == b'\r' {
                "\r\n"
            } else {
                "\n"
            };
        }
    }
    "\n"
}

fn validate_source(kind: SourceKind, source: &str) -> Result<(), FormatError> {
    let errors = match kind {
        SourceKind::Schema => parse_schema_with_recovery(source).errors,
        SourceKind::Locale => parse_locale_with_recovery(source).errors,
    };

    if errors.is_empty() {
        Ok(())
    } else {
        Err(FormatError::Parse(errors))
    }
}

#[cfg(test)]
mod tests;
