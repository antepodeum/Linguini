mod cache;
mod data;
mod plural;

pub use cache::{
    cache_root, fetch_cldr_from_dir, inspect_cache, require_offline_cache, CacheStatus,
    CldrCacheError, CldrCacheResult,
};
pub use data::{
    load_currency_formatting_from_cache, load_date_formatting_from_cache,
    load_number_formatting_from_cache, load_plural_rules, load_plural_rules_from_cache,
    CurrencyFormatData, DateFormatData, FormatWidths, NumberFormatData, PluralCategoryRule,
    PluralRules,
};
pub use plural::{
    evaluate_plural_rule, parse_plural_rule, Operand, OperandExpression, PluralOperands,
    PluralParseError, PluralRule, Range, RangeList, Relation, RelationOperator,
};

pub const CRATE_PURPOSE: &str = "CLDR ingestion and plural rules";

#[cfg(test)]
mod tests;
