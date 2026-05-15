mod engine;
mod ir;

use linguini_syntax::{
    lex_schema_with_recovery, lex_with_recovery, parse_locale_with_recovery,
    parse_schema_with_recovery, ParseError, LOCALE_EXTENSION, SCHEMA_EXTENSION,
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

    Ok(engine::render_tokens(source, &tokens, options))
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

#[cfg(test)]
mod tests;
