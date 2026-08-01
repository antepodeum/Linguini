use super::{
    error_at, Operand, OperandExpression, PluralParseError, PluralParseErrorKind, PluralRule,
    RangeList, Relation, RelationOperator,
};

impl PluralRule {
    pub fn matches(&self, operands: &PluralOperands) -> bool {
        !self.conditions.is_empty()
            && self.conditions.iter().any(|condition| {
                condition
                    .relations
                    .iter()
                    .all(|relation| relation.matches(operands))
            })
    }
}

impl Relation {
    fn matches(&self, operands: &PluralOperands) -> bool {
        let value = self.expression.evaluate(operands);
        match self.operator {
            RelationOperator::Equal => self.ranges.contains_integer(value),
            RelationOperator::NotEqual => !self.ranges.contains_integer(value),
            RelationOperator::In => !value.has_fraction && self.ranges.contains_integer(value),
            RelationOperator::NotIn => value.has_fraction || !self.ranges.contains_integer(value),
            RelationOperator::Within => self.ranges.contains_number(value),
            RelationOperator::NotWithin => !self.ranges.contains_number(value),
        }
    }
}

impl OperandExpression {
    fn evaluate(&self, operands: &PluralOperands) -> OperandValue {
        let mut value = operands.value(self.operand);
        if let Some(modulo) = self.modulo {
            value = value.modulo(modulo);
        }
        value
    }
}

impl RangeList {
    fn contains_integer(&self, value: OperandValue) -> bool {
        value.integer_value().is_some_and(|integer| {
            self.ranges
                .iter()
                .any(|range| integer >= range.start && integer <= range.end)
        })
    }

    fn contains_number(&self, value: OperandValue) -> bool {
        self.ranges.iter().any(|range| {
            value.integer >= range.start
                && (value.integer < range.end
                    || (value.integer == range.end && !value.has_fraction))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralOperands {
    pub n: String,
    pub i: u64,
    pub v: u64,
    pub w: u64,
    pub f: u64,
    pub t: u64,
    pub c: u64,
    pub e: u64,
}

impl PluralOperands {
    pub fn parse(source: &str) -> Result<Self, PluralParseError> {
        let leading_whitespace = source.len() - source.trim_start().len();
        let sample = source.trim();
        if sample.is_empty() {
            return Err(error_at(
                PluralParseErrorKind::EmptyInput,
                leading_whitespace,
                "expected plural sample number",
            ));
        }

        let (unsigned, unsigned_offset) = match sample.as_bytes().first() {
            Some(b'+' | b'-') => (&sample[1..], leading_whitespace + 1),
            _ => (sample, leading_whitespace),
        };
        if unsigned.is_empty() {
            return Err(invalid_number(
                source,
                unsigned_offset,
                "expected digits after the sign",
            ));
        }
        if matches!(unsigned.as_bytes().first(), Some(b'+' | b'-')) {
            return Err(invalid_number(
                source,
                unsigned_offset,
                "plural samples may contain only one leading sign",
            ));
        }

        let exponent_marker = unsigned
            .char_indices()
            .find(|(_, character)| *character == 'c');
        let (mantissa, exponent, exponent_offset) = if let Some((index, _)) = exponent_marker {
            let exponent_source = &unsigned[index + 1..];
            if exponent_source.is_empty() {
                return Err(invalid_number(
                    source,
                    unsigned_offset + index + 1,
                    "expected a compact decimal exponent",
                ));
            }
            let exponent_offset = unsigned_offset + index + 1;
            let exponent = parse_digits(
                exponent_source,
                exponent_offset,
                source,
                "compact decimal exponent",
            )?;
            (&unsigned[..index], exponent, exponent_offset)
        } else {
            (unsigned, 0, unsigned_offset + unsigned.len())
        };

        if mantissa.is_empty() {
            return Err(invalid_number(
                source,
                unsigned_offset,
                "expected digits before the compact decimal exponent",
            ));
        }

        let (integer, fraction, fraction_offset) = if let Some(decimal_offset) = mantissa.find('.')
        {
            let fraction_offset = unsigned_offset + decimal_offset + 1;
            let fraction = &mantissa[decimal_offset + 1..];
            if let Some(extra_decimal) = fraction.find('.') {
                return Err(invalid_number(
                    source,
                    fraction_offset + extra_decimal,
                    "plural samples may contain only one decimal point",
                ));
            }
            (&mantissa[..decimal_offset], fraction, fraction_offset)
        } else {
            (mantissa, "", unsigned_offset + mantissa.len())
        };

        if integer.is_empty() {
            return Err(invalid_number(
                source,
                unsigned_offset,
                "expected integer digits",
            ));
        }

        validate_digits(integer, unsigned_offset, source)?;
        validate_digits(fraction, fraction_offset, source)?;

        let fraction_length = u64::try_from(fraction.len())
            .map_err(|_| overflow(source, fraction_offset, "too many visible fraction digits"))?;
        let consumed_fraction_digits = exponent.min(fraction_length) as usize;
        let shifted_integer_fraction = &fraction[..consumed_fraction_digits];
        let shifted_fraction = &fraction[consumed_fraction_digits..];

        let mut integer_value = 0_u64;
        accumulate_digits(
            &mut integer_value,
            integer,
            unsigned_offset,
            source,
            "integer operand",
        )?;
        accumulate_digits(
            &mut integer_value,
            shifted_integer_fraction,
            fraction_offset,
            source,
            "integer operand",
        )?;

        let remaining_shift = exponent - consumed_fraction_digits as u64;
        if integer_value != 0 && remaining_shift != 0 {
            let power = u32::try_from(remaining_shift)
                .ok()
                .and_then(|power| 10_u64.checked_pow(power))
                .ok_or_else(|| {
                    overflow(
                        source,
                        exponent_offset,
                        "compact exponent overflows the integer operand",
                    )
                })?;
            integer_value = integer_value.checked_mul(power).ok_or_else(|| {
                overflow(
                    source,
                    exponent_offset,
                    "compact exponent overflows the integer operand",
                )
            })?;
        }

        let trimmed_fraction = shifted_fraction.trim_end_matches('0');
        let fraction_value = parse_digits(
            shifted_fraction,
            fraction_offset + consumed_fraction_digits,
            source,
            "fraction operand",
        )?;
        let trimmed_fraction_value = parse_digits(
            trimmed_fraction,
            fraction_offset + consumed_fraction_digits,
            source,
            "trimmed fraction operand",
        )?;
        let visible_fraction_digits = u64::try_from(shifted_fraction.len()).map_err(|_| {
            overflow(
                source,
                fraction_offset + consumed_fraction_digits,
                "too many visible fraction digits",
            )
        })?;
        let trimmed_visible_fraction_digits =
            u64::try_from(trimmed_fraction.len()).map_err(|_| {
                overflow(
                    source,
                    fraction_offset + consumed_fraction_digits,
                    "too many visible fraction digits",
                )
            })?;

        let numeric_source = if exponent_marker.is_some() {
            if shifted_fraction.is_empty() {
                integer_value.to_string()
            } else {
                format!("{integer_value}.{shifted_fraction}")
            }
        } else {
            unsigned.to_owned()
        };

        Ok(Self {
            n: numeric_source,
            i: integer_value,
            v: visible_fraction_digits,
            w: trimmed_visible_fraction_digits,
            f: fraction_value,
            t: trimmed_fraction_value,
            c: exponent,
            e: exponent,
        })
    }

    fn value(&self, operand: Operand) -> OperandValue {
        match operand {
            Operand::N => OperandValue {
                integer: self.i,
                has_fraction: self.f != 0,
            },
            Operand::I => OperandValue::integer(self.i),
            Operand::V => OperandValue::integer(self.v),
            Operand::W => OperandValue::integer(self.w),
            Operand::F => OperandValue::integer(self.f),
            Operand::T => OperandValue::integer(self.t),
            Operand::C => OperandValue::integer(self.c),
            Operand::E => OperandValue::integer(self.e),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct OperandValue {
    integer: u64,
    has_fraction: bool,
}

impl OperandValue {
    fn integer(value: u64) -> Self {
        Self {
            integer: value,
            has_fraction: false,
        }
    }

    fn integer_value(self) -> Option<u64> {
        if self.has_fraction {
            None
        } else {
            Some(self.integer)
        }
    }

    fn modulo(self, modulo: u64) -> Self {
        if modulo == 0 {
            return self;
        }
        Self {
            integer: self.integer % modulo,
            has_fraction: self.has_fraction,
        }
    }
}

pub fn evaluate_plural_rule(rule: &PluralRule, sample: &str) -> Result<bool, PluralParseError> {
    let operands = PluralOperands::parse(sample)?;
    Ok(rule.matches(&operands))
}

fn validate_digits(value: &str, offset: usize, source: &str) -> Result<(), PluralParseError> {
    if let Some((index, _)) = value
        .char_indices()
        .find(|(_, character)| !character.is_ascii_digit())
    {
        Err(invalid_number(
            source,
            offset + index,
            "expected an ASCII decimal digit",
        ))
    } else {
        Ok(())
    }
}

fn parse_digits(
    value: &str,
    offset: usize,
    source: &str,
    operand: &str,
) -> Result<u64, PluralParseError> {
    let mut parsed = 0_u64;
    accumulate_digits(&mut parsed, value, offset, source, operand)?;
    Ok(parsed)
}

fn accumulate_digits(
    parsed: &mut u64,
    value: &str,
    offset: usize,
    source: &str,
    operand: &str,
) -> Result<(), PluralParseError> {
    for (index, byte) in value.bytes().enumerate() {
        if !byte.is_ascii_digit() {
            return Err(invalid_number(
                source,
                offset + index,
                "expected an ASCII decimal digit",
            ));
        }
        let digit = u64::from(byte - b'0');
        *parsed = parsed
            .checked_mul(10)
            .and_then(|current| current.checked_add(digit))
            .ok_or_else(|| {
                overflow(
                    source,
                    offset + index,
                    format!("{operand} exceeds the supported u64 range"),
                )
            })?;
    }
    Ok(())
}

fn invalid_number(source: &str, offset: usize, detail: impl AsRef<str>) -> PluralParseError {
    error_at(
        PluralParseErrorKind::InvalidNumber,
        offset,
        format!("invalid plural sample `{source}`: {}", detail.as_ref()),
    )
}

fn overflow(source: &str, offset: usize, detail: impl AsRef<str>) -> PluralParseError {
    error_at(
        PluralParseErrorKind::NumericOverflow,
        offset,
        format!("plural sample `{source}` is too large: {}", detail.as_ref()),
    )
}
