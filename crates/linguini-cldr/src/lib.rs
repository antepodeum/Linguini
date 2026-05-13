mod cache;
mod data;
mod plural;

pub use cache::{
    cache_root, fetch_cldr_from_dir, fetch_cldr_from_dir_for_locales,
    fetch_cldr_from_official_repo_for_locales, inspect_cache, require_offline_cache,
    required_cldr_json_paths, CacheStatus, CldrCacheError, CldrCacheResult,
    OFFICIAL_CLDR_JSON_REPO,
};
pub use data::{
    built_in_plural_rules, compiled_currency_formatting, compiled_date_formatting,
    compiled_number_formatting, compiled_plural_rules, load_currency_formatting_from_cache,
    load_date_formatting_from_cache, load_number_formatting_from_cache, load_plural_rules,
    load_plural_rules_from_cache, CompiledPluralCategory, CompiledPluralRules, CurrencyFormatData,
    DateFormatData, FormatWidths, NumberFormatData, PluralCategoryRule, PluralRules,
};
pub use plural::{
    evaluate_plural_rule, parse_plural_rule, Condition, Operand, OperandExpression, PluralOperands,
    PluralParseError, PluralRule, Range, RangeList, Relation, RelationOperator,
};

pub const CRATE_PURPOSE: &str = "CLDR ingestion and plural rules";

#[cfg(test)]
mod tests;
