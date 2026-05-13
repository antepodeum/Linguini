mod cache;
mod data;
mod plural;

pub use cache::{
    cache_root, fetch_cldr_from_dir, inspect_cache, require_offline_cache, CacheStatus,
    CldrCacheError, CldrCacheResult,
};
pub use data::{load_plural_rules, load_plural_rules_from_cache, PluralCategoryRule, PluralRules};
pub use plural::{
    parse_plural_rule, Operand, OperandExpression, PluralParseError, PluralRule, Range, RangeList,
    Relation, RelationOperator,
};

pub const CRATE_PURPOSE: &str = "CLDR ingestion and plural rules";
