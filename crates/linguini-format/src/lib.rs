mod engine;
mod ir;
mod semantics;

use linguini_syntax::{
    parse_locale_with_tokens, parse_schema_with_tokens, ParseError, Span, LOCALE_EXTENSION,
    SCHEMA_EXTENSION,
};
use semantics::FormatSemantics;
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SourceKind {
    Schema,
    Locale,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormatOptions {
    /// Number of ASCII spaces emitted for each structural indentation level.
    ///
    /// `0` disables indentation. Values greater than 64 are rejected to bound allocations.
    pub indent_width: usize,
    /// Preferred display-column width for safely wrappable structural lines.
    ///
    /// `0` disables wrapping. Message text, string contents, comments, and indivisible tokens are
    /// never split, so those lines may exceed this value. Width uses terminal display columns
    /// rather than UTF-8 bytes or Unicode scalar counts. Values greater than 1,000,000 are
    /// rejected to keep configuration mistakes bounded.
    pub max_line_width: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FormatError {
    Parse(Vec<ParseError>),
    UnsupportedExtension(String),
    InvalidOptions(String),
    InvalidTokenSpan(Span),
    InvalidSyntaxSpan(Span),
    SyntaxMismatch(String),
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
            Self::InvalidSyntaxSpan(span) => write!(
                f,
                "parser returned an invalid syntax span {}..{}",
                span.start, span.end
            ),
            Self::SyntaxMismatch(message) => {
                write!(f, "lexer and parser disagree: {message}")
            }
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
    /// Infers a Linguini source kind from an exact `.lgs` or `.lgl` extension.
    ///
    /// Unknown, missing, and non-UTF-8 extensions return `None`.
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|extension| extension.to_str()) {
            Some(SCHEMA_EXTENSION) => Some(Self::Schema),
            Some(LOCALE_EXTENSION) => Some(Self::Locale),
            _ => None,
        }
    }
}

/// Formats source after inferring its kind from `path`.
///
/// Unknown or missing extensions are rejected; they never default to locale syntax.
pub fn format_path_source(path: &Path, source: &str) -> Result<String, FormatError> {
    let kind = SourceKind::from_path(path).ok_or_else(|| {
        let extension = match path.extension() {
            Some(extension) => extension.to_str().unwrap_or("<non-UTF-8>"),
            None => "<none>",
        };
        FormatError::UnsupportedExtension(extension.to_owned())
    })?;
    format_source(kind, source, &FormatOptions::default())
}

/// Formats a complete, valid Linguini source file.
///
/// Parsing and semantic validation happen before rendering, so invalid input returns an error
/// without partial formatted output. Structural newlines follow the first newline sequence in the
/// source, while newline bytes inside raw text remain verbatim.
pub fn format_source(
    kind: SourceKind,
    source: &str,
    options: &FormatOptions,
) -> Result<String, FormatError> {
    validate_options(options)?;
    let (tokens, semantics) = match kind {
        SourceKind::Schema => {
            let parsed = parse_schema_with_tokens(source).map_err(FormatError::Parse)?;
            (parsed.tokens, FormatSemantics::schema())
        }
        SourceKind::Locale => {
            let parsed = parse_locale_with_tokens(source).map_err(FormatError::Parse)?;
            let semantics = FormatSemantics::locale(&parsed.ast, source)?;
            (parsed.tokens, semantics)
        }
    };

    semantics.validate_tokens(source, &tokens)?;
    engine::render_tokens(source, &tokens, &semantics, options, detect_newline(source))
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
        match *byte {
            b'\r' if bytes.get(index + 1) == Some(&b'\n') => return "\r\n",
            b'\r' => return "\r",
            b'\n' => return "\n",
            _ => {}
        }
    }
    "\n"
}

#[cfg(test)]
mod tests;
