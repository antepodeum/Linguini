pub const CRATE_PURPOSE: &str = "shared core Linguini types";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FormatterKind {
    Number,
    Currency,
    Date,
    Unknown,
}

impl FormatterKind {
    pub fn from_name(value: &str) -> Self {
        match value {
            "number" => Self::Number,
            "currency" => Self::Currency,
            "date" => Self::Date,
            _ => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::Currency => "currency",
            Self::Date => "date",
            Self::Unknown => "unknown",
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
}
