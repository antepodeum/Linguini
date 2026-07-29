use super::{
    CurrencyFormatData, CurrencyFractionData, DateFormatData, DateSymbolWidths, FormatWidths,
    NumberFormatData, NumberPattern, NumberPatternPart, PluralCategoryRule, PluralRules,
};
use crate::plural::{
    Condition, Operand, OperandExpression, PluralOperands, PluralRule, Range, RangeList, Relation,
    RelationOperator,
};
use crate::{canonicalize_locale, locale_fallback_chain_for, LocaleFallbackComponent};

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
            .filter(|category| category.category != "other")
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

/// Verified identity and coverage manifest for the checked-in CLDR artifact.
pub const CLDR_DATA_MANIFEST_JSON: &str = include_str!("generated/manifest.json");

fn resolve_locale_tag<T>(
    locale: &str,
    component: LocaleFallbackComponent,
    mut lookup: impl FnMut(&str) -> Option<T>,
) -> Option<T> {
    let canonical = canonicalize_locale(locale).ok()?;
    let allow_root_data = canonical
        .split('-')
        .next()
        .is_some_and(|language| language == "und");
    for tag in locale_fallback_chain_for(locale, component).ok()? {
        if tag == "und" && !allow_root_data {
            break;
        }
        if let Some(value) = lookup(&tag) {
            return Some(value);
        }
    }
    None
}

pub fn compiled_plural_rules(locale: &str) -> Option<CompiledPluralRules> {
    resolve_locale_tag(
        locale,
        LocaleFallbackComponent::Plurals,
        generated_plural_rules,
    )
}

pub fn built_in_plural_rules(locale: &str) -> Option<PluralRules> {
    resolve_locale_tag(
        locale,
        LocaleFallbackComponent::Plurals,
        generated_plural_rule_sources,
    )
}

pub fn built_in_text_direction(locale: &str) -> Option<&'static str> {
    resolve_locale_tag(
        locale,
        LocaleFallbackComponent::Main,
        generated_text_direction,
    )
}

include!("generated/cldr_tables.rs");

pub fn compiled_number_formatting(locale: &str) -> Option<NumberFormatData> {
    resolve_locale_tag(
        locale,
        LocaleFallbackComponent::Main,
        generated_number_formatting,
    )
}

pub fn compiled_currency_formatting(locale: &str) -> Option<CurrencyFormatData> {
    resolve_locale_tag(
        locale,
        LocaleFallbackComponent::Main,
        generated_currency_formatting,
    )
}

/// Returns CLDR supplemental fraction and rounding rules for a currency code.
///
/// CLDR's `DEFAULT` rule applies to well-formed three-letter codes without a
/// currency-specific override. Malformed codes return `None`.
pub fn compiled_currency_fraction(currency: &str) -> Option<CurrencyFractionData> {
    if currency.len() != 3 || !currency.bytes().all(|byte| byte.is_ascii_alphabetic()) {
        return None;
    }
    let currency = currency.to_ascii_uppercase();
    generated_currency_fraction(&currency).or_else(|| generated_currency_fraction("DEFAULT"))
}

pub fn compiled_date_formatting(locale: &str) -> Option<DateFormatData> {
    resolve_locale_tag(
        locale,
        LocaleFallbackComponent::Main,
        generated_date_formatting,
    )
}
