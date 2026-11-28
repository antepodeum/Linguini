//! Compiler bindings that source-owned values must leave available.

pub(super) fn catalog_name(name: &str) -> bool {
    matches!(name, "lgl" | "normalizeMessageArgs")
        || ["formatNumber", "formatCurrency", "formatDate"]
            .iter()
            .any(|base| {
                name == *base
                    || name
                        .strip_prefix(base)
                        .is_some_and(|tail| tail.starts_with('_'))
            })
        || name
            .strip_prefix("plural")
            .is_some_and(|tail| tail.as_bytes().first().is_some_and(u8::is_ascii_uppercase))
}

pub(super) fn inline_name(name: &str) -> bool {
    catalog_name(name)
        || plural_name(name)
        || matches!(
            name,
            "message"
                | "MAX_GENERATED_DECIMAL_DIGITS"
                | "parseGeneratedDecimal"
                | "scientificGeneratedDecimal"
                | "shiftScientificMantissa"
                | "roundGeneratedFraction"
                | "throwInvalidNumber"
                | "padNumber"
                | "coerceDate"
                | "createUTCDate"
                | "validateDateFields"
                | "throwInvalidDate"
                | "getWeekYear"
                | "localizeDigit"
        )
        || [
            "scaleBy",
            "roundTo",
            "padTo",
            "scientificWith",
            "engineeringWith",
        ]
        .iter()
        .any(|prefix| {
            name.strip_prefix(prefix)
                .is_some_and(|tail| tail.as_bytes().first().is_some_and(u8::is_ascii_digit))
        })
}

pub(super) fn plural_name(name: &str) -> bool {
    matches!(
        name,
        "MAX_PLURAL_DECIMAL_DIGITS" | "throwInvalidPluralNumber"
    )
}

/// Predefined types and contextual words that make declarations/references ambiguous.
pub(super) fn type_keyword(name: &str) -> bool {
    matches!(
        name,
        "any"
            | "unknown"
            | "never"
            | "number"
            | "bigint"
            | "boolean"
            | "string"
            | "symbol"
            | "object"
            | "undefined"
            | "as"
            | "infer"
            | "keyof"
            | "readonly"
            | "unique"
    )
}
