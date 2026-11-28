//! Exact decimal quantum rounding, with a small-integer fast path.

use linguini_codegen::NumberRoundingPlan;

use super::super::{compare_decimal, increment_integer};

#[derive(Clone, Copy)]
pub(super) enum RoundingQuantum<'a> {
    Small(u32),
    Large(&'a NumberRoundingPlan),
}

impl<'a> RoundingQuantum<'a> {
    pub(super) fn for_plan(plan: &'a NumberRoundingPlan) -> Option<Self> {
        match plan.increment.parse::<u32>() {
            Ok(value) => Some(Self::Small(value.max(1))),
            Err(_)
                if !plan.increment.is_empty()
                    && plan.increment.bytes().all(|digit| digit.is_ascii_digit())
                    && plan.increment.bytes().any(|digit| digit != b'0') =>
            {
                Some(Self::Large(plan))
            }
            Err(_) => None,
        }
    }
}

pub(super) fn round_fraction(
    integer: &mut String,
    fraction: &mut String,
    max_digits: usize,
    quantum: RoundingQuantum<'_>,
) {
    let source_fraction = std::mem::take(fraction);
    let mut kept_fraction = source_fraction
        .get(..max_digits.min(source_fraction.len()))
        .unwrap_or_default()
        .to_owned();
    kept_fraction.extend(std::iter::repeat_n(
        '0',
        max_digits.saturating_sub(kept_fraction.len()),
    ));
    let discarded = source_fraction
        .get(max_digits.min(source_fraction.len())..)
        .unwrap_or_default();
    let scaled = format!("{integer}{kept_fraction}");
    let mut rounded = match quantum {
        RoundingQuantum::Small(quantum) => {
            let (mut quotient, remainder) = divide_decimal_by_u32(&scaled, quantum);
            if should_round_up(remainder, discarded, quantum) {
                increment_integer(&mut quotient);
            }
            multiply_decimal_by_u32(&quotient, quantum)
        }
        RoundingQuantum::Large(plan) => round_large_quantum(&scaled, discarded, plan),
    };
    let required_length = max_digits.saturating_add(1);
    if rounded.len() < required_length {
        rounded.insert_str(0, &"0".repeat(required_length - rounded.len()));
    }
    if max_digits == 0 {
        *integer = rounded;
        return;
    }
    let split = rounded.len() - max_digits;
    *integer = rounded[..split].to_owned();
    *fraction = rounded[split..].to_owned();
}

fn divide_decimal_by_u32(value: &str, divisor: u32) -> (String, u32) {
    debug_assert_ne!(divisor, 0);
    let mut quotient = String::with_capacity(value.len());
    let mut remainder = 0_u32;
    for byte in value.bytes() {
        let current = u64::from(remainder) * 10 + u64::from(byte - b'0');
        let digit = u32::try_from(current / u64::from(divisor)).expect("quotient digit fits");
        remainder = u32::try_from(current % u64::from(divisor)).expect("remainder fits");
        if !quotient.is_empty() || digit != 0 {
            quotient.push(char::from(
                b'0' + u8::try_from(digit).expect("single decimal digit"),
            ));
        }
    }
    if quotient.is_empty() {
        quotient.push('0');
    }
    (quotient, remainder)
}

fn should_round_up(remainder: u32, discarded: &str, quantum: u32) -> bool {
    let half = quantum / 2;
    remainder > half
        || (remainder == half
            && (quantum.is_multiple_of(2)
                || discarded
                    .as_bytes()
                    .first()
                    .is_some_and(|digit| *digit >= b'5')))
}

fn multiply_decimal_by_u32(value: &str, multiplier: u32) -> String {
    let mut reversed = Vec::with_capacity(value.len() + 10);
    let mut carry = 0_u64;
    for byte in value.bytes().rev() {
        let product = u64::from(byte - b'0') * u64::from(multiplier) + carry;
        reversed.push(b'0' + u8::try_from(product % 10).expect("single decimal digit"));
        carry = product / 10;
    }
    while carry != 0 {
        reversed.push(b'0' + u8::try_from(carry % 10).expect("single decimal digit"));
        carry /= 10;
    }
    decimal_from_reversed(reversed)
}

fn round_large_quantum(scaled: &str, discarded: &str, plan: &NumberRoundingPlan) -> String {
    let remainder = decimal_remainder(scaled, &plan.increment);
    let round_up = match compare_decimal(&remainder, &plan.half_integer) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Equal => {
            !plan.half_fraction
                || discarded
                    .as_bytes()
                    .first()
                    .is_some_and(|digit| *digit >= b'5')
        }
        std::cmp::Ordering::Less => false,
    };
    let rounded = subtract_decimal(scaled, &remainder);
    if round_up {
        add_decimal(&rounded, &plan.increment)
    } else {
        rounded
    }
}

fn decimal_remainder(value: &str, divisor: &str) -> String {
    let mut remainder = "0".to_owned();
    for digit in value.bytes() {
        if remainder == "0" {
            remainder.clear();
        }
        remainder.push(char::from(digit));
        // The previous remainder was below the divisor, so this step needs at most nine subtractions.
        while compare_decimal(&remainder, divisor).is_ge() {
            remainder = subtract_decimal(&remainder, divisor);
        }
    }
    remainder
}

fn subtract_decimal(left: &str, right: &str) -> String {
    debug_assert!(compare_decimal(left, right).is_ge());
    let mut right = right.bytes().rev();
    let mut borrow = 0_i16;
    let mut reversed = Vec::with_capacity(left.len());
    for digit in left.bytes().rev() {
        let mut difference =
            i16::from(digit - b'0') - i16::from(right.next().unwrap_or(b'0') - b'0') - borrow;
        if difference < 0 {
            difference += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        reversed.push(b'0' + u8::try_from(difference).expect("single decimal digit"));
    }
    debug_assert_eq!(borrow, 0);
    debug_assert!(right.all(|digit| digit == b'0'));
    decimal_from_reversed(reversed)
}

fn add_decimal(left: &str, right: &str) -> String {
    let mut left_digits = left.bytes().rev();
    let mut right_digits = right.bytes().rev();
    let mut carry = 0;
    let mut reversed = Vec::with_capacity(left.len().max(right.len()) + 1);
    for _ in 0..left.len().max(right.len()) {
        let left = left_digits.next().unwrap_or(b'0') - b'0';
        let right = right_digits.next().unwrap_or(b'0') - b'0';
        let sum = left + right + carry;
        reversed.push(b'0' + sum % 10);
        carry = sum / 10;
    }
    if carry > 0 {
        reversed.push(b'0' + carry);
    }
    decimal_from_reversed(reversed)
}

fn decimal_from_reversed(mut digits: Vec<u8>) -> String {
    while digits.len() > 1 && digits.last() == Some(&b'0') {
        digits.pop();
    }
    digits.reverse();
    String::from_utf8(digits).expect("ASCII decimal digits")
}
