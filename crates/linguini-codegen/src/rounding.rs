//! Static decimal rounding choices shared by target backends.

/// Exact positive quantum and its halfway threshold, resolved during generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberRoundingPlan {
    pub fraction_digits: u8,
    pub increment: String,
    pub half_integer: String,
    pub half_fraction: bool,
}

impl NumberRoundingPlan {
    /// Compile the fraction after a one-digit scientific mantissa carries.
    /// The carried result must itself be a multiple of the rounding quantum.
    pub fn scientific_carry_fraction(&self, significant_digits: Option<u8>) -> Option<String> {
        self.scientific_carry_fraction_for_width(significant_digits, 1)
    }

    /// Prove the carried mantissa for a fixed integer width during generation.
    /// Unproven increments require rounding the original input at the new exponent.
    pub(crate) fn scientific_carry_fraction_for_width(
        &self,
        significant_digits: Option<u8>,
        integer_digits: u8,
    ) -> Option<String> {
        if integer_digits == 0 {
            return None;
        }
        let digits = match significant_digits {
            Some(0) => return None,
            Some(digits) => digits.saturating_sub(integer_digits),
            None if self.increment == "1" => self.fraction_digits,
            None => {
                let increment = self.increment.parse::<u128>().ok().filter(|&n| n > 0)?;
                let mut remainder = 1_u128;
                // The carried mantissa is 10^(integer_digits - 1), in scaled units.
                for _ in 0..u16::from(self.fraction_digits) + u16::from(integer_digits) - 1 {
                    remainder = remainder.checked_mul(10)? % increment;
                }
                if remainder != 0 {
                    return None;
                }
                self.fraction_digits
            }
        };
        Some("0".repeat(usize::from(digits)))
    }

    /// CLDR's zero increment selects ordinary rounding to one scaled unit.
    pub fn new(fraction_digits: u8, increment: &str) -> Option<Self> {
        if increment.is_empty() || !increment.bytes().all(|digit| digit.is_ascii_digit()) {
            return None;
        }
        let increment = increment.trim_start_matches('0');
        let increment = if increment.is_empty() { "1" } else { increment };
        let mut carry = 0;
        let half_integer = increment
            .bytes()
            .map(|digit| {
                let value = carry * 10 + digit - b'0';
                carry = value % 2;
                char::from(b'0' + value / 2)
            })
            .collect::<String>();
        Some(Self {
            fraction_digits,
            increment: increment.to_owned(),
            half_integer: match half_integer.trim_start_matches('0') {
                "" => "0".to_owned(),
                value => value.to_owned(),
            },
            half_fraction: carry != 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_exact_half_quantums_without_bounded_integer_conversion() {
        for (input, increment, half, fraction) in [
            ("0", "1", "0", true),
            ("0001", "1", "0", true),
            ("5", "5", "2", true),
            ("10", "10", "5", false),
            (
                "18446744073709551617",
                "18446744073709551617",
                "9223372036854775808",
                true,
            ),
        ] {
            let plan = NumberRoundingPlan::new(2, input).unwrap();
            assert_eq!(plan.fraction_digits, 2);
            assert_eq!(
                (
                    plan.increment.as_str(),
                    plan.half_integer.as_str(),
                    plan.half_fraction
                ),
                (increment, half, fraction)
            );
        }
        for invalid in ["", "-1", "1.5", "1e2", " 5"] {
            assert!(NumberRoundingPlan::new(2, invalid).is_none());
        }
    }

    #[test]
    fn compiles_only_proven_normalized_scientific_carries() {
        for (fraction_digits, increment, significant, expected) in [
            (0, "1", None, Some("")),
            (2, "1", None, Some("00")),
            (3, "0", None, Some("000")),
            (2, "1", Some(1), Some("")),
            (2, "5", Some(4), Some("000")),
            (2, "5", None, Some("00")),
            (2, "125", None, None),
            (2, "1000", None, None),
            (2, "6", None, None),
            (2, "1005", None, None),
            (2, "18446744073709551617", None, None),
            (2, "1", Some(0), None),
        ] {
            let plan = NumberRoundingPlan::new(fraction_digits, increment).unwrap();
            assert_eq!(
                plan.scientific_carry_fraction(significant).as_deref(),
                expected
            );
        }
        for fraction_digits in 0..=37 {
            let one = 10_u128.pow(u32::from(fraction_digits));
            for increment in 1..=256_u128 {
                let plan =
                    NumberRoundingPlan::new(fraction_digits, &increment.to_string()).unwrap();
                assert_eq!(
                    plan.scientific_carry_fraction(None).is_some(),
                    one.is_multiple_of(increment),
                    "{fraction_digits}/{increment}"
                );
            }
        }
        let wide = NumberRoundingPlan::new(255, "5").unwrap();
        assert_eq!(wide.scientific_carry_fraction(None), Some("0".repeat(255)));
        let quantum = NumberRoundingPlan::new(1, "25").unwrap();
        assert_eq!(quantum.scientific_carry_fraction_for_width(None, 1), None);
        assert_eq!(
            quantum.scientific_carry_fraction_for_width(None, 3),
            Some("0".into())
        );
        assert_eq!(
            quantum.scientific_carry_fraction_for_width(Some(4), 3),
            Some("0".into())
        );
        assert_eq!(
            quantum.scientific_carry_fraction_for_width(Some(1), 3),
            Some(String::new())
        );
        assert_eq!(quantum.scientific_carry_fraction_for_width(None, 0), None);
    }
}
