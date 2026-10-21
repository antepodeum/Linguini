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
}
