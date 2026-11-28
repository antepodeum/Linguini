//! Static scientific mantissa, precision, and carry decisions.

use super::NumberPrecisionPlan;
use crate::NumberRoundingPlan;
use linguini_cldr::{NumberFormatData, NumberPatternPart};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberScientificPlan {
    /// Fixed mantissa width; engineering notation uses one as its lower bound.
    pub integer_digits: u8,
    /// Exponents are multiples of this value. One selects ordinary scientific notation.
    pub exponent_step: u8,
    pub precision: NumberPrecisionPlan,
    pub exponent_digits: u8,
    pub separator: &'static str,
    pub positive_sign: &'static str,
    pub negative_sign: &'static str,
    pub carry_fraction: Option<String>,
}

impl NumberScientificPlan {
    pub(super) fn for_pattern(
        positive: &NumberPatternPart,
        numbers: &NumberFormatData,
        rounding: &NumberRoundingPlan,
    ) -> Option<Self> {
        let exponent_digits = positive.exponent_digits?;
        let explicit_significant = positive.max_significant_digits;
        let engineering = explicit_significant.is_none()
            && positive
                .max_integer_digits
                .is_some_and(|maximum| maximum > positive.min_integer_digits && maximum > 1);
        let integer_digits = if engineering || explicit_significant.is_some() {
            1
        } else {
            positive.min_integer_digits.max(1)
        };
        let precision = if let Some(maximum) = explicit_significant {
            NumberPrecisionPlan::Significant {
                minimum: u16::from(positive.min_significant_digits.unwrap_or(1)),
                maximum: u16::from(maximum),
            }
        } else if rounding.increment != "1" {
            NumberPrecisionPlan::Fraction
        } else if positive.min_integer_digits == 0
            && positive.max_fraction_digits == 0
            && !positive.decimal_separator_always
        {
            NumberPrecisionPlan::Unlimited
        } else if engineering || positive.min_integer_digits == 0 {
            let has_required_digits =
                positive.min_integer_digits > 0 || positive.min_fraction_digits > 0;
            NumberPrecisionPlan::Significant {
                minimum: (u16::from(if engineering {
                    1
                } else {
                    positive.min_integer_digits
                }) + u16::from(positive.min_fraction_digits))
                .max(1),
                maximum: u16::from(if has_required_digits {
                    positive.min_integer_digits
                } else {
                    1
                }) + u16::from(positive.max_fraction_digits),
            }
        } else {
            NumberPrecisionPlan::Fraction
        };
        let carry_fraction = match precision {
            NumberPrecisionPlan::Significant { maximum, .. } => {
                Some("0".repeat(usize::from(maximum).saturating_sub(1)))
            }
            NumberPrecisionPlan::Fraction => {
                rounding.scientific_carry_fraction_for_width(None, integer_digits)
            }
            NumberPrecisionPlan::Unlimited => None,
        };
        Some(Self {
            integer_digits,
            exponent_step: if engineering {
                positive
                    .max_integer_digits
                    .expect("engineering integer width")
            } else {
                1
            },
            precision,
            exponent_digits,
            separator: numbers.exponential_symbol,
            positive_sign: if positive.exponent_sign_always {
                numbers.plus_sign
            } else {
                ""
            },
            negative_sign: numbers.minus_sign,
            carry_fraction,
        })
    }
}
