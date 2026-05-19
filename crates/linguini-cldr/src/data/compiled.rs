use super::{
    CurrencyFormatData, DateFormatData, FormatWidths, NumberFormatData, PluralCategoryRule,
    PluralRules,
};
use crate::plural::{
    Condition, Operand, OperandExpression, PluralOperands, PluralRule, Range, RangeList, Relation,
    RelationOperator,
};

#[derive(Debug, Clone, Copy)]
pub struct CompiledPluralRules {
    pub locale: &'static str,
    pub categories: &'static [CompiledPluralCategory],
}

impl CompiledPluralRules {
    pub fn category_for(&self, sample: &str) -> Result<&'static str, String> {
        let operands = PluralOperands::parse(sample).map_err(|error| error.to_string())?;
        Ok(self.category_for_operands(&operands))
    }

    pub fn category_for_operands(&self, operands: &PluralOperands) -> &'static str {
        self.categories
            .iter()
            .find(|category| (category.matches)(operands))
            .map(|category| category.category)
            .unwrap_or("other")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledPluralCategory {
    pub category: &'static str,
    pub matches: fn(&PluralOperands) -> bool,
}

pub fn compiled_plural_rules(locale: &str) -> Option<CompiledPluralRules> {
    generated_plural_rules(locale)
}

pub fn built_in_plural_rules(locale: &str) -> Option<PluralRules> {
    generated_plural_rule_sources(locale)
}

pub fn built_in_text_direction(locale: &str) -> Option<&'static str> {
    generated_text_direction(locale).or_else(|| {
        let language = locale.split('-').next()?;
        generated_text_direction(language)
    })
}

include!(concat!(
    env!("OUT_DIR"),
    "/linguini_generated_plural_rules.rs"
));

pub fn compiled_number_formatting(locale: &str) -> Option<NumberFormatData> {
    generated_number_formatting(locale)
}

pub fn compiled_currency_formatting(locale: &str) -> Option<CurrencyFormatData> {
    generated_currency_formatting(locale)
}

pub fn compiled_date_formatting(locale: &str) -> Option<DateFormatData> {
    generated_date_formatting(locale)
}
