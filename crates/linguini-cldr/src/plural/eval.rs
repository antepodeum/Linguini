use super::{
    error, Operand, OperandExpression, PluralParseError, PluralRule, RangeList, Relation,
    RelationOperator,
};

impl PluralRule {
    pub fn matches(&self, operands: &PluralOperands) -> bool {
        self.conditions.is_empty()
            || self.conditions.iter().any(|condition| {
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
            RelationOperator::In => value.is_integer && self.ranges.contains_integer(value),
            RelationOperator::NotIn => !(value.is_integer && self.ranges.contains_integer(value)),
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
        self.ranges
            .iter()
            .any(|range| value.number >= range.start as f64 && value.number <= range.end as f64)
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
        let source = source.trim();
        if source.is_empty() {
            return Err(error("expected plural sample number"));
        }
        let unsigned = source.trim_start_matches(['+', '-']);
        if unsigned.is_empty() {
            return Err(error("expected plural sample number"));
        }

        let mut parts = unsigned.split('.');
        let integer = parts.next().unwrap_or_default();
        let fraction = parts.next().unwrap_or_default();
        if parts.next().is_some() || integer.is_empty() {
            return Err(error(format!("invalid plural sample `{source}`")));
        }
        if !integer.chars().all(|character| character.is_ascii_digit())
            || !fraction.chars().all(|character| character.is_ascii_digit())
        {
            return Err(error(format!("invalid plural sample `{source}`")));
        }

        let trimmed_fraction = fraction.trim_end_matches('0');
        Ok(Self {
            n: unsigned.to_owned(),
            i: integer
                .parse()
                .map_err(|_| error(format!("invalid plural sample `{source}`")))?,
            v: fraction.len() as u64,
            w: trimmed_fraction.len() as u64,
            f: parse_fraction_digits(fraction, source)?,
            t: parse_fraction_digits(trimmed_fraction, source)?,
            c: 0,
            e: 0,
        })
    }

    fn value(&self, operand: Operand) -> OperandValue {
        match operand {
            Operand::N => OperandValue {
                number: self.n.parse().unwrap_or(self.i as f64),
                is_integer: self.v == 0,
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
    number: f64,
    is_integer: bool,
}

impl OperandValue {
    fn integer(value: u64) -> Self {
        Self {
            number: value as f64,
            is_integer: true,
        }
    }

    fn integer_value(self) -> Option<u64> {
        if self.is_integer && self.number >= 0.0 {
            Some(self.number as u64)
        } else {
            None
        }
    }

    fn modulo(self, modulo: u64) -> Self {
        if modulo == 0 {
            return self;
        }
        Self {
            number: self.number % modulo as f64,
            is_integer: self.is_integer,
        }
    }
}

pub fn evaluate_plural_rule(rule: &PluralRule, sample: &str) -> Result<bool, PluralParseError> {
    let operands = PluralOperands::parse(sample)?;
    Ok(rule.matches(&operands))
}

fn parse_fraction_digits(value: &str, sample: &str) -> Result<u64, PluralParseError> {
    if value.is_empty() {
        Ok(0)
    } else {
        value
            .parse()
            .map_err(|_| error(format!("invalid plural sample `{sample}`")))
    }
}
