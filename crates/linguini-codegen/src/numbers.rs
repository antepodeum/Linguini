//! Target-independent choices resolved from a selected CLDR number pattern.

use linguini_cldr::{NumberAffix, NumberAffixPart, NumberFormatData, NumberPadding, NumberPattern};

use crate::NumberRoundingPlan;

mod currency;
mod scientific;

pub use scientific::NumberScientificPlan;

/// Rounding selected from a compiled numeric pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberPrecisionPlan {
    Fraction,
    Significant { minimum: u16, maximum: u16 },
    Unlimited,
}

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
    /// Effective precision, including significant precision implied by scientific patterns.
    pub fn precision(&self) -> NumberPrecisionPlan {
        if let Some(scientific) = &self.scientific {
            scientific.precision
        } else if let Some(maximum) = self.max_significant_digits {
            NumberPrecisionPlan::Significant {
                minimum: u16::from(self.min_significant_digits.unwrap_or(1)),
                maximum: u16::from(maximum),
            }
        } else {
            NumberPrecisionPlan::Fraction
        }
    }

    /// Scientific patterns with derived significant precision pad total digits instead.
    pub fn minimum_fraction_digits(&self) -> u8 {
        if self.scientific.is_some()
            && self.max_significant_digits.is_none()
            && matches!(self.precision(), NumberPrecisionPlan::Significant { .. })
        {
            0
        } else {
            self.min_fraction_digits
        }
    }

    /// Minimum significant padding can be requested independently of rounding precision.
    pub fn minimum_significant_digits(&self) -> Option<u16> {
        self.min_significant_digits
            .map(u16::from)
            .or_else(|| match self.precision() {
                NumberPrecisionPlan::Significant { minimum, .. } if minimum > 1 => Some(minimum),
                _ => None,
            })
    }

    /// Whether a fixed-fraction pattern may omit its sole integer zero.
    pub fn can_omit_integer_zero(&self) -> bool {
        self.min_integer_digits == 0
            && self.min_fraction_digits > 0
            && self.min_significant_digits.is_none()
            && self.max_significant_digits.is_none()
            && self.scientific.is_none()
    }

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
        let scientific = NumberScientificPlan::for_pattern(positive, numbers, &rounding);
        Some(Self {
            affixes: NumberAffixPlan {
                prefix,
                suffix,
                negative_prefix,
                negative_suffix,
            },
            min_integer_digits: scientific
                .as_ref()
                .map_or(positive.min_integer_digits, |plan| plan.integer_digits),
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
