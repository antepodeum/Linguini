mod compiled;

use crate::plural::{PluralOperands, PluralRule};
pub use compiled::{
    built_in_plural_rules, built_in_text_direction, compiled_calendar_formatting,
    compiled_currency_formatting, compiled_currency_fraction, compiled_currency_symbol,
    compiled_date_formatting, compiled_day_period_rules, compiled_number_formatting,
    compiled_plural_rules, compiled_time_zone_formatting, compiled_week_data, japanese_era_starts,
    CompiledPluralCategory, CompiledPluralRules, CLDR_DATA_MANIFEST_JSON,
};
pub(crate) use compiled::{
    generated_extension_key_alias, generated_extension_type_alias, generated_language_alias,
    generated_likely_subtag, generated_parent_locale, generated_regular_week_region,
    generated_regular_week_subdivision_region, generated_script_alias, generated_territory_alias,
    generated_variant_alias, generated_week_subdivision_region, GENERATED_LOCALE_CANDIDATES,
    GENERATED_NONLIKELY_SCRIPT_PARENT,
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
            .filter(|category| category.category != "other")
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
    pub numbering_system: &'static str,
    pub digits: &'static str,
    pub decimal_symbol: &'static str,
    pub group_symbol: &'static str,
    /// Localized minus symbol, including CLDR bidirectional marks.
    pub minus_sign: &'static str,
    /// Localized plus symbol, including CLDR bidirectional marks.
    pub plus_sign: &'static str,
    /// Literal symbol separating a scientific mantissa from its exponent.
    pub exponential_symbol: &'static str,
    /// Digits required beyond the primary group before separators are shown.
    pub minimum_grouping_digits: u8,
    pub decimal_pattern: NumberPattern,
    pub percent_pattern: NumberPattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrencyFormatData {
    pub locale: &'static str,
    /// Currency decimal separator, resolved against the default number symbols.
    pub decimal_symbol: &'static str,
    /// Currency grouping separator, resolved against the default number symbols.
    pub group_symbol: &'static str,
    pub standard_pattern: NumberPattern,
    pub accounting_pattern: Option<NumberPattern>,
    /// Full alternative selected when the currency boundary is a Unicode letter.
    pub standard_alpha_next_to_number: Option<&'static NumberPattern>,
    /// Accounting alternative; absence retains the selected base accounting pattern.
    pub accounting_alpha_next_to_number: Option<&'static NumberPattern>,
}

/// CLDR supplemental currency fraction and rounding rules.
///
/// Rounding values are integer increments at the corresponding number of
/// fraction digits. For example, `rounding = 5` with `digits = 2` means an
/// increment of `0.05`. A zero increment means ordinary decimal rounding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrencyFractionData {
    pub digits: u8,
    pub rounding: u16,
    pub cash_digits: u8,
    pub cash_rounding: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateFormatData {
    pub locale: &'static str,
    pub digits: &'static str,
    pub date_formats: FormatWidths,
    pub time_formats: FormatWidths,
    pub date_time_formats: FormatWidths,
    pub months: DateSymbolWidths<12>,
    pub weekdays: DateSymbolWidths<7>,
}

/// Pinned CLDR calendar patterns and flattened symbol paths (for example,
/// `months.stand-alone.wide.1`). Symbol paths retain CLDR context and width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarFormatData {
    pub locale: &'static str,
    pub calendar: &'static str,
    pub date_formats: FormatWidths,
    pub time_formats: FormatWidths,
    pub date_time_formats: FormatWidths,
    pub symbols: &'static [&'static [(&'static str, &'static str)]],
    pub skeletons: &'static [(&'static str, &'static str)],
    /// Patterns whose year uses CLDR's algorithmic Japanese first-year form.
    pub japanese_year_patterns: &'static [&'static str],
}

impl CalendarFormatData {
    /// Resolve one context/width-qualified symbol from the pinned calendar data.
    pub fn symbol(&self, path: &str) -> Option<&'static str> {
        self.symbols
            .iter()
            .flat_map(|table| table.iter())
            .find(|(name, _)| *name == path)
            .map(|(_, value)| *value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayPeriodRule {
    pub name: &'static str,
    pub at: Option<u16>,
    pub from: Option<u16>,
    pub before: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeekData {
    /// Sunday is zero, matching ECMAScript weekday numbering.
    pub first_day: u8,
    pub min_days: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeZoneFormatData {
    pub gmt_format: &'static str,
    pub gmt_zero_format: &'static str,
    pub hour_format: &'static str,
    /// Explicit localized UTC standard names; absent names use localized GMT fallback.
    pub utc_standard_long: Option<&'static str>,
    pub utc_standard_short: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatWidths {
    pub full: &'static str,
    pub long: &'static str,
    pub medium: &'static str,
    pub short: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateSymbolWidths<const N: usize> {
    pub wide: [&'static str; N],
    pub abbreviated: [&'static str; N],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberPattern {
    pub positive: NumberPatternPart,
    pub negative: Option<NumberPatternPart>,
}

/// A compiled numeric affix, with currency substitutions distinct from literal text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberAffix {
    Literal(&'static str),
    CurrencySymbol,
    CurrencyCode,
    Parts(&'static [NumberAffixPart]),
}

/// Text is owned while compiling CLDR and borrowed in the checked data tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberAffixPart<Text = &'static str> {
    Literal(Text),
    CurrencySymbol,
    CurrencyCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberPatternPart {
    pub prefix: NumberAffix,
    pub suffix: NumberAffix,
    pub min_integer_digits: u8,
    /// Fixed-digit scientific mantissa width, including optional placeholders.
    /// Ordinary decimal and significant-digit patterns use `None`.
    pub max_integer_digits: Option<u8>,
    pub min_fraction_digits: u8,
    pub max_fraction_digits: u8,
    /// Retain the decimal separator even when the formatted fraction is empty.
    pub decimal_separator_always: bool,
    pub primary_group_size: Option<u8>,
    pub secondary_group_size: Option<u8>,
    pub min_significant_digits: Option<u8>,
    pub max_significant_digits: Option<u8>,
    /// ASCII rounding increment digits at `max_fraction_digits` scale.
    pub rounding_increment: Option<&'static str>,
    pub exponent_digits: Option<u8>,
    pub exponent_sign_always: bool,
    /// Decimal magnitude applied before formatting: 1, 100, or 1000.
    pub scale: u16,
    pub padding: Option<NumberPadding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberPadding {
    pub character: char,
    pub width: u16,
    pub position: NumberPaddingPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberPaddingPosition {
    BeforePrefix,
    AfterPrefix,
    BeforeSuffix,
    AfterSuffix,
}

#[cfg(test)]
mod tests;
