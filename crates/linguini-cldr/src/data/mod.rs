mod compiled;

use crate::plural::{PluralOperands, PluralRule};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberFormatData {
    pub locale: &'static str,
    pub decimal_symbol: &'static str,
    pub group_symbol: &'static str,
    pub decimal_pattern: NumberPattern,
    pub percent_pattern: NumberPattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrencyFormatData {
    pub locale: &'static str,
    pub standard_pattern: NumberPattern,
    pub accounting_pattern: Option<NumberPattern>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateFormatData {
    pub locale: &'static str,
    pub date_formats: FormatWidths,
    pub time_formats: FormatWidths,
    pub date_time_formats: FormatWidths,
    pub months: DateSymbolWidths,
    pub weekdays: DateSymbolWidths,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatWidths {
    pub full: &'static str,
    pub long: &'static str,
    pub medium: &'static str,
    pub short: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateSymbolWidths {
    pub wide: &'static [&'static str],
    pub abbreviated: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberPattern {
    pub positive: NumberPatternPart,
    pub negative: Option<NumberPatternPart>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberPatternPart {
    pub prefix: &'static str,
    pub suffix: &'static str,
    pub min_integer_digits: u8,
    pub min_fraction_digits: u8,
    pub max_fraction_digits: u8,
    pub primary_group_size: Option<u8>,
    pub secondary_group_size: Option<u8>,
}

#[cfg(test)]
mod tests;
