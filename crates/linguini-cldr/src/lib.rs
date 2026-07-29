mod data;
mod locale;
mod plural;

pub use data::{
    built_in_plural_rules, built_in_text_direction, compiled_currency_formatting,
    compiled_currency_fraction, compiled_date_formatting, compiled_number_formatting,
    compiled_plural_rules, CompiledPluralCategory, CompiledPluralRules, CurrencyFormatData,
    CurrencyFractionData, DateFormatData, DateSymbolWidths, FormatWidths, NumberFormatData,
    NumberPattern, NumberPatternPart, PluralCategoryRule, PluralRules, CLDR_DATA_MANIFEST_JSON,
};
pub use locale::{
    canonicalize_locale, locale_fallback_chain, locale_fallback_chain_for,
    locale_resolution_candidates, maximize_locale, LocaleError, LocaleFallbackComponent,
};
pub use plural::{
    evaluate_plural_rule, parse_plural_rule, Condition, Operand, OperandExpression, PluralOperands,
    PluralParseError, PluralParseErrorKind, PluralRule, Range, RangeList, Relation,
    RelationOperator,
};

pub const CRATE_PURPOSE: &str = "compiled CLDR plural and formatting data";

#[cfg(test)]
mod tests;
