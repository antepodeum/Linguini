mod cache_loaders;
mod compiled;
mod json;

use crate::plural::{PluralOperands, PluralRule};
pub use cache_loaders::{
    load_currency_formatting_from_cache, load_date_formatting_from_cache,
    load_number_formatting_from_cache, load_plural_rules, load_plural_rules_from_cache,
    load_text_direction_from_cache,
};
pub use compiled::{
    built_in_plural_rules, built_in_text_direction, compiled_currency_formatting,
    compiled_date_formatting, compiled_number_formatting, compiled_plural_rules,
    CompiledPluralCategory, CompiledPluralRules,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralRules {
    pub locale: String,
    pub categories: Vec<PluralCategoryRule>,
}

impl PluralRules {
    pub fn category_for(&self, sample: &str) -> Result<&str, String> {
        let operands = PluralOperands::parse(sample).map_err(|error| error.to_string())?;
        Ok(self.category_for_operands(&operands))
    }

    pub fn category_for_operands(&self, operands: &PluralOperands) -> &str {
        self.categories
            .iter()
            .find(|category| category.rule.matches(operands))
            .map(|category| category.category.as_str())
            .unwrap_or("other")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralCategoryRule {
    pub category: String,
    pub source: String,
    pub rule: PluralRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberFormatData {
    pub locale: String,
    pub decimal_symbol: String,
    pub group_symbol: String,
    pub decimal_pattern: String,
    pub percent_pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrencyFormatData {
    pub locale: String,
    pub standard_pattern: String,
    pub accounting_pattern: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateFormatData {
    pub locale: String,
    pub date_formats: FormatWidths,
    pub time_formats: FormatWidths,
    pub date_time_formats: FormatWidths,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatWidths {
    pub full: String,
    pub long: String,
    pub medium: String,
    pub short: String,
}

#[cfg(test)]
mod tests;
