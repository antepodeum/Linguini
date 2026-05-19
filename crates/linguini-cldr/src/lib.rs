mod data;
mod plural;

pub use data::{
    built_in_plural_rules, built_in_text_direction, compiled_currency_formatting,
    compiled_date_formatting, compiled_number_formatting, compiled_plural_rules,
    CompiledPluralCategory, CompiledPluralRules, CurrencyFormatData, DateFormatData, FormatWidths,
    NumberFormatData, PluralCategoryRule, PluralRules,
};
pub use plural::{
    evaluate_plural_rule, parse_plural_rule, Condition, Operand, OperandExpression, PluralOperands,
    PluralParseError, PluralRule, Range, RangeList, Relation, RelationOperator,
};

pub const CRATE_PURPOSE: &str = "compiled CLDR plural and formatting data";

#[cfg(test)]
mod tests;
