use super::{
    CurrencyFormatData, DateFormatData, FormatWidths, NumberFormatData, NumberPattern,
    NumberPatternPart, PluralCategoryRule, PluralRules,
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

fn resolve_locale_tag<T>(locale: &str, mut lookup: impl FnMut(&str) -> Option<T>) -> Option<T> {
    let mut tag = locale;
    loop {
        if let Some(value) = lookup(tag) {
            return Some(value);
        }
        tag = match tag.rfind('-') {
            Some(index) if index > 0 => &tag[..index],
            _ => break,
        };
    }
    None
}

pub fn compiled_plural_rules(locale: &str) -> Option<CompiledPluralRules> {
    resolve_locale_tag(locale, generated_plural_rules)
}

pub fn built_in_plural_rules(locale: &str) -> Option<PluralRules> {
    resolve_locale_tag(locale, generated_plural_rule_sources)
}

pub fn built_in_text_direction(locale: &str) -> Option<&'static str> {
    resolve_locale_tag(locale, generated_text_direction)
}

linguini_cldr_macros::compiled_cldr_tables!();

pub fn compiled_number_formatting(locale: &str) -> Option<NumberFormatData> {
    resolve_locale_tag(locale, generated_number_formatting)
}

pub fn compiled_currency_formatting(locale: &str) -> Option<CurrencyFormatData> {
    resolve_locale_tag(locale, generated_currency_formatting)
}

pub fn compiled_date_formatting(locale: &str) -> Option<DateFormatData> {
    resolve_locale_tag(locale, generated_date_formatting)
}
