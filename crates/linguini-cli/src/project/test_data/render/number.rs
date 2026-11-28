//! Exact numeric previews from shared compiled formatter plans.

mod rounding;

use super::{
    increment_integer, missing_cldr_data, parse_decimal, DecimalParts, RenderError,
    MAX_DECIMAL_DIGITS,
};
use linguini_cldr::NumberPaddingPosition;
#[cfg(test)]
use linguini_cldr::{NumberFormatData, NumberPattern};
use linguini_codegen::{
    NumberFormatPlan, NumberGroupingPlan, NumberPrecisionPlan, NumberScientificPlan,
};
use rounding::{round_fraction, RoundingQuantum};

#[cfg(test)]
pub(super) fn format_number_pattern(
    value: &str,
    pattern: &NumberPattern,
    numbers: &NumberFormatData,
    currency_symbol: Option<&str>,
    currency_fraction: Option<(u8, u16)>,
) -> Result<String, RenderError> {
    let mut pattern = *pattern;
    let rounding = currency_fraction.map(|(digits, rounding)| {
        pattern.positive.min_fraction_digits = digits;
        pattern.positive.max_fraction_digits = digits;
        rounding.to_string()
    });
    let plan = NumberFormatPlan::new(&pattern, currency_symbol, numbers, rounding.as_deref())
        .ok_or_else(|| RenderError::Unsupported {
            detail: "invalid compiled numeric preview plan".to_owned(),
        })?;
    format_number_plan(value, &plan, numbers.locale)
}

pub(super) fn format_number_plan(
    value: &str,
    plan: &NumberFormatPlan,
    locale: &str,
) -> Result<String, RenderError> {
    let rounding_increment = if plan.precision() == NumberPrecisionPlan::Fraction {
        RoundingQuantum::for_plan(&plan.rounding).ok_or_else(|| RenderError::Unsupported {
            detail: "invalid compiled numeric rounding plan".to_owned(),
        })?
    } else {
        RoundingQuantum::Small(1)
    };
    let mut decimal = parse_decimal(value)?;
    decimal
        .fraction
        .truncate(decimal.fraction.trim_end_matches('0').len());
    if plan.scale_shift > 0 {
        if decimal.integer.len() + decimal.fraction.len() + usize::from(plan.scale_shift)
            > MAX_DECIMAL_DIGITS
        {
            return Err(RenderError::InvalidNumber {
                value: value.to_owned(),
            });
        }
        decimal = shift_decimal(&decimal, i32::from(plan.scale_shift));
    }
    let (mantissa, mut exponent) = if let Some(scientific) = &plan.scientific {
        normalize_scientific(&decimal, scientific)
    } else {
        (decimal, 0)
    };
    let mut decimal = mantissa.clone();
    round_precision(&mut decimal, plan, rounding_increment, value)?;
    if let Some(scientific) = &plan.scientific {
        if plan.precision() != NumberPrecisionPlan::Unlimited
            && magnitude(&decimal).is_some_and(|rounded| Some(rounded) != magnitude(&mantissa))
            && (scientific.exponent_step == 1
                || mantissa.integer.len() == usize::from(scientific.exponent_step))
        {
            exponent += i32::from(scientific.exponent_step);
            decimal = shift_decimal(&mantissa, -i32::from(scientific.exponent_step));
            round_precision(&mut decimal, plan, rounding_increment, value)?;
        }
    }
    let min_fraction_digits = usize::from(plan.minimum_fraction_digits());
    while decimal.fraction.len() > min_fraction_digits && decimal.fraction.ends_with('0') {
        decimal.fraction.pop();
    }
    while decimal.fraction.len() < min_fraction_digits {
        decimal.fraction.push('0');
    }
    while decimal.integer.len() < usize::from(plan.min_integer_digits) {
        decimal.integer.insert(0, '0');
    }
    if let Some(minimum) = plan.minimum_significant_digits() {
        let source = format!("{}{}", decimal.integer, decimal.fraction);
        let current = source
            .bytes()
            .position(|digit| digit != b'0')
            .map_or(1, |first| source.len() - first);
        decimal
            .fraction
            .push_str(&"0".repeat(usize::from(minimum).saturating_sub(current)));
    }

    let grouped = if plan.can_omit_integer_zero() && decimal.integer == "0" {
        String::new()
    } else {
        group_integer_digits(&decimal.integer, plan.grouping, plan.group_symbol)
    };
    let formatted = if decimal.fraction.is_empty() && !plan.decimal_separator_always {
        grouped
    } else {
        format!("{grouped}{}{}", plan.decimal_symbol, decimal.fraction)
    };
    let (prefix, suffix) = if decimal.negative {
        (&plan.affixes.negative_prefix, &plan.affixes.negative_suffix)
    } else {
        (&plan.affixes.prefix, &plan.affixes.suffix)
    };
    let digits = linguini_codegen::decimal_digit_symbols(plan.digits)
        .ok_or_else(|| missing_cldr_data("numeric symbols", locale))?;
    let mut numeric = String::new();
    append_localized_digits(&mut numeric, &formatted, &digits);
    if let Some(scientific) = &plan.scientific {
        numeric.push_str(scientific.separator);
        numeric.push_str(if exponent < 0 {
            scientific.negative_sign
        } else {
            scientific.positive_sign
        });
        let mut exponent_digits = exponent.unsigned_abs().to_string();
        if exponent_digits.len() < usize::from(scientific.exponent_digits) {
            exponent_digits.insert_str(
                0,
                &"0".repeat(usize::from(scientific.exponent_digits) - exponent_digits.len()),
            );
        }
        append_localized_digits(&mut numeric, &exponent_digits, &digits);
    }
    let padding = plan
        .padding
        .map(|padding| {
            padding
                .character
                .to_string()
                .repeat(usize::from(padding.width).saturating_sub(
                    prefix.chars().count() + numeric.chars().count() + suffix.chars().count(),
                ))
        })
        .unwrap_or_default();
    Ok(match plan.padding.map(|padding| padding.position) {
        Some(NumberPaddingPosition::BeforePrefix) => format!("{padding}{prefix}{numeric}{suffix}"),
        Some(NumberPaddingPosition::AfterPrefix) => format!("{prefix}{padding}{numeric}{suffix}"),
        Some(NumberPaddingPosition::BeforeSuffix) => format!("{prefix}{numeric}{padding}{suffix}"),
        _ => format!("{prefix}{numeric}{suffix}{padding}"),
    })
}

fn magnitude(decimal: &DecimalParts) -> Option<i32> {
    if decimal.integer != "0" {
        Some(decimal.integer.len() as i32 - 1)
    } else {
        decimal
            .fraction
            .bytes()
            .position(|digit| digit != b'0')
            .map(|first| -(first as i32) - 1)
    }
}

fn shift_decimal(decimal: &DecimalParts, shift: i32) -> DecimalParts {
    let source = format!("{}{}", decimal.integer, decimal.fraction);
    let position = decimal.integer.len() as i32 + shift;
    let (integer, fraction) = if position <= 0 {
        ("0".to_owned(), "0".repeat((-position) as usize) + &source)
    } else {
        let position = position as usize;
        if position >= source.len() {
            (
                source.clone() + &"0".repeat(position - source.len()),
                String::new(),
            )
        } else {
            (source[..position].to_owned(), source[position..].to_owned())
        }
    };
    DecimalParts {
        negative: decimal.negative,
        integer: {
            let integer = integer.trim_start_matches('0');
            if integer.is_empty() {
                "0".to_owned()
            } else {
                integer.to_owned()
            }
        },
        fraction: fraction.trim_end_matches('0').to_owned(),
    }
}

fn normalize_scientific(
    decimal: &DecimalParts,
    plan: &NumberScientificPlan,
) -> (DecimalParts, i32) {
    let Some(magnitude) = magnitude(decimal) else {
        return (decimal.clone(), 0);
    };
    let exponent = if plan.exponent_step > 1 {
        magnitude.div_euclid(i32::from(plan.exponent_step)) * i32::from(plan.exponent_step)
    } else {
        magnitude - i32::from(plan.integer_digits) + 1
    };
    (shift_decimal(decimal, -exponent), exponent)
}

fn round_precision(
    decimal: &mut DecimalParts,
    plan: &NumberFormatPlan,
    increment: RoundingQuantum<'_>,
    original: &str,
) -> Result<(), RenderError> {
    let digits = match plan.precision() {
        NumberPrecisionPlan::Unlimited => return Ok(()),
        NumberPrecisionPlan::Fraction => usize::from(plan.max_fraction_digits),
        NumberPrecisionPlan::Significant { maximum, .. } => {
            if maximum == 0 {
                return Err(RenderError::Unsupported {
                    detail: "zero significant precision".into(),
                });
            }
            let source = format!("{}{}", decimal.integer, decimal.fraction);
            let Some(first) = source.bytes().position(|digit| digit != b'0') else {
                decimal.integer = "0".into();
                decimal.fraction.clear();
                return Ok(());
            };
            let cut = first + usize::from(maximum);
            if cut < decimal.integer.len() {
                let mut integer = decimal.integer[..cut].to_owned();
                if decimal.integer.as_bytes()[cut] >= b'5' {
                    increment_integer(&mut integer);
                }
                integer.push_str(&"0".repeat(decimal.integer.len() - cut));
                decimal.integer = integer;
                decimal.fraction.clear();
                return Ok(());
            }
            cut - decimal.integer.len()
        }
    };
    if digits > MAX_DECIMAL_DIGITS {
        return Err(RenderError::InvalidNumber {
            value: original.to_owned(),
        });
    }
    round_fraction(
        &mut decimal.integer,
        &mut decimal.fraction,
        digits,
        if matches!(plan.precision(), NumberPrecisionPlan::Significant { .. }) {
            RoundingQuantum::Small(1)
        } else {
            increment
        },
    );
    Ok(())
}

pub(super) fn append_localized_digits(output: &mut String, value: &str, digits: &[char; 10]) {
    output.extend(value.chars().map(|character| {
        if character.is_ascii_digit() {
            digits[usize::from(character as u8 - b'0')]
        } else {
            character
        }
    }));
}

fn group_integer_digits(
    integer: &str,
    grouping: Option<NumberGroupingPlan>,
    symbol: &str,
) -> String {
    let Some(grouping) = grouping else {
        return integer.to_owned();
    };
    if integer.len() <= usize::from(grouping.threshold) {
        return integer.to_owned();
    }
    let primary = usize::from(grouping.primary_size);
    let secondary = usize::from(grouping.secondary_size);
    let primary_start = integer.len() - primary;
    let end = if primary == secondary {
        integer.len()
    } else {
        primary_start
    };
    let first = match end % secondary {
        0 => secondary,
        length => length,
    };
    let mut output = integer[..first].to_owned();
    let mut offset = first;
    while offset < end {
        output.push_str(symbol);
        output.push_str(&integer[offset..offset + secondary]);
        offset += secondary;
    }
    if primary != secondary {
        output.push_str(symbol);
        output.push_str(&integer[primary_start..]);
    }
    output
}
