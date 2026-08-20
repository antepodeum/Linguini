#![deny(missing_docs)]

//! Language-wide primitive and formatter identities.
//!
//! This crate owns only closed names that must agree across syntax, analysis,
//! IR, code generation, and tooling. It contains no parser or target behavior.

/// Canonical source-language name of the built-in CLDR plural selector type and
/// its numeric conversion intrinsic.
pub const PLURAL_TYPE_NAME: &str = "Plural";

/// Returns whether `name` is the exact, case-sensitive plural intrinsic.
pub fn is_plural_intrinsic(name: &str) -> bool {
    name == PLURAL_TYPE_NAME
}

/// Formatter identity retained from source through validated IR.
///
/// Unknown names remain representable so diagnostics can report the original
/// spelling instead of erasing it during parsing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FormatterKind {
    /// Locale decimal-number formatting.
    Number,
    /// Locale currency formatting.
    Currency,
    /// Locale date formatting.
    Date,
    /// Unrecognized source spelling retained for diagnostics.
    Unknown(String),
}

impl FormatterKind {
    /// Classifies a source formatter name without discarding unknown names.
    pub fn from_name(value: &str) -> Self {
        match value {
            "number" => Self::Number,
            "currency" => Self::Currency,
            "date" => Self::Date,
            _ => Self::Unknown(value.to_owned()),
        }
    }

    /// Returns the canonical known name or the retained unknown spelling.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Number => "number",
            Self::Currency => "currency",
            Self::Date => "date",
            Self::Unknown(name) => name,
        }
    }
}

/// Primitive type identity shared by every compiler layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeKind {
    /// Unicode text.
    String,
    /// General numeric input.
    Number,
    /// Precision-preserving decimal input.
    Decimal,
    /// Date input.
    Date,
    /// Boolean input.
    Boolean,
}

impl TypeKind {
    /// Resolves an exact, case-sensitive source primitive name.
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "String" => Some(Self::String),
            "Number" => Some(Self::Number),
            "Decimal" => Some(Self::Decimal),
            "Date" => Some(Self::Date),
            "Boolean" => Some(Self::Boolean),
            _ => None,
        }
    }

    /// Returns the canonical source-language spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Number => "Number",
            Self::Decimal => "Decimal",
            Self::Date => "Date",
            Self::Boolean => "Boolean",
        }
    }

    /// Lists every built-in primitive in stable source order.
    pub const fn all() -> &'static [Self] {
        &[
            Self::String,
            Self::Number,
            Self::Decimal,
            Self::Date,
            Self::Boolean,
        ]
    }

    /// Returns the formatter applied to an unannotated interpolation, if any.
    pub const fn default_formatter(self) -> Option<FormatterKind> {
        match self {
            Self::Number | Self::Decimal => Some(FormatterKind::Number),
            Self::Date => Some(FormatterKind::Date),
            Self::String | Self::Boolean => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_plural_intrinsic, FormatterKind, TypeKind, PLURAL_TYPE_NAME};

    #[test]
    fn plural_intrinsic_has_one_canonical_name() {
        assert!(is_plural_intrinsic(PLURAL_TYPE_NAME));
        assert!(!is_plural_intrinsic("plural"));
        assert!(!is_plural_intrinsic("PLURAL"));
    }

    #[test]
    fn formatter_kind_round_trips_known_names() {
        for kind in [
            FormatterKind::Number,
            FormatterKind::Currency,
            FormatterKind::Date,
        ] {
            assert_eq!(FormatterKind::from_name(kind.as_str()), kind);
        }
        assert_eq!(
            FormatterKind::from_name("custom"),
            FormatterKind::Unknown("custom".to_owned())
        );
        assert_eq!(FormatterKind::from_name("custom").as_str(), "custom");
    }

    #[test]
    fn type_kind_round_trips_known_names() {
        for &kind in TypeKind::all() {
            assert_eq!(TypeKind::from_name(kind.as_str()), Some(kind));
        }
        assert_eq!(TypeKind::from_name("Void"), None);
    }

    #[test]
    fn primitive_contract_centralizes_default_formatting() {
        assert_eq!(
            TypeKind::Decimal.default_formatter(),
            Some(FormatterKind::Number)
        );
        assert_eq!(
            TypeKind::Date.default_formatter(),
            Some(FormatterKind::Date)
        );
    }
}
