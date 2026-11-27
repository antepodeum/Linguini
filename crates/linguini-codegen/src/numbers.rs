//! Target-independent choices resolved from a selected CLDR number pattern.

use linguini_cldr::{NumberAffix, NumberAffixPart, NumberFormatData, NumberPadding, NumberPattern};

use crate::NumberRoundingPlan;

mod currency;

/// Literal affixes after currency replacement and implicit negative-sign expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberAffixPlan {
    pub prefix: String,
    pub suffix: String,
    pub negative_prefix: String,
    pub negative_suffix: String,
}

/// Group widths and the precomputed boundary at which grouping becomes visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberGroupingPlan {
    pub primary_size: u8,
    pub secondary_size: u8,
    /// Group only when the integer's length exceeds this value.
    pub threshold: u16,
}

/// Selected scientific symbols and a proven carry result, when one is available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberScientificPlan {
    pub exponent_digits: u8,
    pub separator: &'static str,
    pub positive_sign: &'static str,
    pub negative_sign: &'static str,
    pub carry_fraction: Option<String>,
}

/// Semantic number-format choices with no target syntax or runtime helper names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberFormatPlan {
    pub affixes: NumberAffixPlan,
    pub min_integer_digits: u8,
    pub min_fraction_digits: u8,
    pub max_fraction_digits: u8,
    pub decimal_separator_always: bool,
    pub min_significant_digits: Option<u8>,
    pub max_significant_digits: Option<u8>,
    pub decimal_symbol: &'static str,
    pub group_symbol: &'static str,
    pub digits: &'static str,
    pub grouping: Option<NumberGroupingPlan>,
    pub scale: u16,
    pub scale_shift: u8,
    pub rounding: NumberRoundingPlan,
    pub scientific: Option<NumberScientificPlan>,
    pub padding: Option<NumberPadding>,
}

impl NumberFormatPlan {
    /// Compile a selected, validated CLDR pattern and its effective number symbols.
    /// Currency callers supply the currency's resolved separators and optional fraction quantum.
    /// Unsupported scales, zero selected group sizes, or malformed quanta return `None`.
    pub fn new(
        pattern: &NumberPattern,
        symbol: Option<&str>,
        numbers: &NumberFormatData,
        rounding: Option<&str>,
    ) -> Option<Self> {
        let positive = &pattern.positive;
        let affix = |value| resolve_affix(value, symbol);
        let prefix = affix(positive.prefix);
        let suffix = affix(positive.suffix);
        let negative_prefix = pattern.negative.as_ref().map_or_else(
            || format!("{}{prefix}", numbers.minus_sign),
            |negative| affix(negative.prefix),
        );
        let negative_suffix = pattern
            .negative
            .as_ref()
            .map_or_else(|| suffix.clone(), |negative| affix(negative.suffix));
        let grouping = match positive.primary_group_size {
            Some(primary) => {
                let secondary = positive.secondary_group_size.unwrap_or(primary);
                if primary == 0 || secondary == 0 || numbers.minimum_grouping_digits == 0 {
                    return None;
                }
                Some(NumberGroupingPlan {
                    primary_size: primary,
                    secondary_size: secondary,
                    threshold: u16::from(primary) + u16::from(numbers.minimum_grouping_digits) - 1,
                })
            }
            None => None,
        };
        let scale_shift = match positive.scale {
            1 => 0,
            100 => 2,
            1000 => 3,
            _ => return None,
        };
        let rounding = NumberRoundingPlan::new(
            positive.max_fraction_digits,
            rounding.or(positive.rounding_increment).unwrap_or("1"),
        )?;
        let scientific = positive
            .exponent_digits
            .map(|exponent_digits| NumberScientificPlan {
                exponent_digits,
                separator: numbers.exponential_symbol,
                positive_sign: if positive.exponent_sign_always {
                    numbers.plus_sign
                } else {
                    ""
                },
                negative_sign: numbers.minus_sign,
                carry_fraction: rounding.scientific_carry_fraction(positive.max_significant_digits),
            });
        Some(Self {
            affixes: NumberAffixPlan {
                prefix,
                suffix,
                negative_prefix,
                negative_suffix,
            },
            min_integer_digits: positive.min_integer_digits,
            min_fraction_digits: positive.min_fraction_digits,
            max_fraction_digits: positive.max_fraction_digits,
            decimal_separator_always: positive.decimal_separator_always,
            min_significant_digits: positive.min_significant_digits,
            max_significant_digits: positive.max_significant_digits,
            decimal_symbol: numbers.decimal_symbol,
            group_symbol: numbers.group_symbol,
            digits: numbers.digits,
            grouping,
            scale: positive.scale,
            scale_shift,
            rounding,
            scientific,
            padding: positive.padding,
        })
    }
}

fn resolve_affix(affix: NumberAffix, symbol: Option<&str>) -> String {
    match affix {
        NumberAffix::Literal(text) => text.to_owned(),
        NumberAffix::CurrencySymbol => symbol.unwrap_or("¤").to_owned(),
        NumberAffix::Parts(parts) => parts
            .iter()
            .map(|part| match part {
                NumberAffixPart::Literal(text) => *text,
                NumberAffixPart::CurrencySymbol => symbol.unwrap_or("¤"),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests;
