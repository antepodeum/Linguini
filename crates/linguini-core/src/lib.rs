/// Canonical source-language name of the built-in CLDR plural selector type and
/// its numeric conversion intrinsic.
pub const PLURAL_TYPE_NAME: &str = "Plural";

pub fn is_plural_intrinsic(name: &str) -> bool {
    name == PLURAL_TYPE_NAME
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FormatterKind {
    Number,
    Currency,
    Date,
    Unknown(String),
}

impl FormatterKind {
    pub fn from_name(value: &str) -> Self {
        match value {
            "number" => Self::Number,
            "currency" => Self::Currency,
            "date" => Self::Date,
            _ => Self::Unknown(value.to_owned()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Number => "number",
            Self::Currency => "currency",
            Self::Date => "date",
            Self::Unknown(name) => name,
        }
    }

    pub const fn all_known() -> &'static [Self] {
        &[Self::Number, Self::Currency, Self::Date]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeKind {
    String,
    Number,
    Decimal,
    Date,
    Boolean,
}

impl TypeKind {
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

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Number => "Number",
            Self::Decimal => "Decimal",
            Self::Date => "Date",
            Self::Boolean => "Boolean",
        }
    }

    pub const fn all() -> &'static [Self] {
        &[
            Self::String,
            Self::Number,
            Self::Decimal,
            Self::Date,
            Self::Boolean,
        ]
    }

    pub const fn supports_dispatch(self) -> bool {
        matches!(self, Self::String | Self::Number | Self::Decimal)
    }

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
        for kind in FormatterKind::all_known() {
            assert_eq!(FormatterKind::from_name(kind.as_str()), *kind);
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
    fn primitive_contract_centralizes_dispatch_and_formatting() {
        assert!(TypeKind::String.supports_dispatch());
        assert!(TypeKind::Number.supports_dispatch());
        assert!(TypeKind::Decimal.supports_dispatch());
        assert!(!TypeKind::Date.supports_dispatch());
        assert!(!TypeKind::Boolean.supports_dispatch());
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
