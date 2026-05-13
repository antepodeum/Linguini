use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralRule {
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Condition {
    pub relations: Vec<Relation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relation {
    pub expression: OperandExpression,
    pub operator: RelationOperator,
    pub ranges: RangeList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationOperator {
    Equal,
    NotEqual,
    In,
    NotIn,
    Within,
    NotWithin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperandExpression {
    pub operand: Operand,
    pub modulo: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    N,
    I,
    V,
    W,
    F,
    T,
    C,
    E,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeList {
    pub ranges: Vec<Range>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralParseError {
    pub message: String,
}

impl Display for PluralParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PluralParseError {}

pub fn parse_plural_rule(source: &str) -> Result<PluralRule, PluralParseError> {
    let rule = source.split('@').next().unwrap_or(source).trim();
    if rule.is_empty() {
        return Ok(PluralRule { conditions: vec![] });
    }

    let mut conditions = Vec::new();
    for condition in split_keyword(rule, "or") {
        let mut relations = Vec::new();
        for relation in split_keyword(condition, "and") {
            relations.push(parse_relation(relation.trim())?);
        }
        conditions.push(Condition { relations });
    }

    Ok(PluralRule { conditions })
}

fn parse_relation(source: &str) -> Result<Relation, PluralParseError> {
    let tokens = tokenize(source);
    let mut cursor = Cursor::new(tokens);
    let expression = parse_operand_expression(&mut cursor)?;
    let operator = parse_operator(&mut cursor)?;
    let ranges = parse_range_list(&mut cursor)?;

    if cursor.peek().is_some() {
        return Err(error(format!(
            "unexpected token `{}`",
            cursor.peek().unwrap()
        )));
    }

    Ok(Relation {
        expression,
        operator,
        ranges,
    })
}

fn parse_operand_expression(cursor: &mut Cursor) -> Result<OperandExpression, PluralParseError> {
    let operand = match cursor.next().as_deref() {
        Some("n") => Operand::N,
        Some("i") => Operand::I,
        Some("v") => Operand::V,
        Some("w") => Operand::W,
        Some("f") => Operand::F,
        Some("t") => Operand::T,
        Some("c") => Operand::C,
        Some("e") => Operand::E,
        Some(token) => return Err(error(format!("expected plural operand, got `{token}`"))),
        None => return Err(error("expected plural operand")),
    };

    let modulo = if cursor.consume("%") || cursor.consume("mod") {
        Some(parse_number(cursor.next().as_deref())?)
    } else {
        None
    };

    Ok(OperandExpression { operand, modulo })
}

fn parse_operator(cursor: &mut Cursor) -> Result<RelationOperator, PluralParseError> {
    if cursor.consume("=") || cursor.consume("is") {
        return Ok(RelationOperator::Equal);
    }

    if cursor.consume("!=") {
        return Ok(RelationOperator::NotEqual);
    }

    if cursor.consume("not") {
        if cursor.consume("=") || cursor.consume("is") {
            return Ok(RelationOperator::NotEqual);
        }
        if cursor.consume("in") {
            return Ok(RelationOperator::NotIn);
        }
        if cursor.consume("within") {
            return Ok(RelationOperator::NotWithin);
        }
        return Err(error("expected relation operator after `not`"));
    }

    if cursor.consume("in") {
        return Ok(RelationOperator::In);
    }

    if cursor.consume("within") {
        return Ok(RelationOperator::Within);
    }

    Err(error("expected relation operator"))
}

fn parse_range_list(cursor: &mut Cursor) -> Result<RangeList, PluralParseError> {
    let mut ranges = Vec::new();
    loop {
        let start = parse_number(cursor.next().as_deref())?;
        let end = if cursor.consume("..") {
            parse_number(cursor.next().as_deref())?
        } else {
            start
        };
        ranges.push(Range { start, end });

        if !cursor.consume(",") {
            break;
        }
    }

    Ok(RangeList { ranges })
}

fn split_keyword<'a>(source: &'a str, keyword: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut index = 0;

    while let Some(relative) = source[index..].find(keyword) {
        let absolute = index + relative;
        let before = source[..absolute].chars().last();
        let after = source[absolute + keyword.len()..].chars().next();
        let bounded_before = before.map_or(true, char::is_whitespace);
        let bounded_after = after.map_or(true, char::is_whitespace);

        if bounded_before && bounded_after {
            parts.push(source[start..absolute].trim());
            start = absolute + keyword.len();
        }

        index = absolute + keyword.len();
    }

    parts.push(source[start..].trim());
    parts.into_iter().filter(|part| !part.is_empty()).collect()
}

fn tokenize(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = source.chars().peekable();

    while let Some(character) = chars.next() {
        match character {
            ' ' | '\t' | '\n' | '\r' => push_current(&mut tokens, &mut current),
            ',' | '%' | '=' => {
                push_current(&mut tokens, &mut current);
                tokens.push(character.to_string());
            }
            '!' if chars.peek() == Some(&'=') => {
                chars.next();
                push_current(&mut tokens, &mut current);
                tokens.push("!=".to_owned());
            }
            '.' if chars.peek() == Some(&'.') => {
                chars.next();
                push_current(&mut tokens, &mut current);
                tokens.push("..".to_owned());
            }
            _ => current.push(character),
        }
    }

    push_current(&mut tokens, &mut current);
    tokens
}

fn push_current(tokens: &mut Vec<String>, current: &mut String) {
    if !current.is_empty() {
        tokens.push(std::mem::take(current));
    }
}

fn parse_number(value: Option<&str>) -> Result<u64, PluralParseError> {
    let Some(value) = value else {
        return Err(error("expected number"));
    };
    value
        .parse()
        .map_err(|_| error(format!("expected number, got `{value}`")))
}

fn error(message: impl Into<String>) -> PluralParseError {
    PluralParseError {
        message: message.into(),
    }
}

struct Cursor {
    tokens: Vec<String>,
    index: usize,
}

impl Cursor {
    fn new(tokens: Vec<String>) -> Self {
        Self { tokens, index: 0 }
    }

    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.index).map(String::as_str)
    }

    fn next(&mut self) -> Option<String> {
        let value = self.tokens.get(self.index).cloned();
        if value.is_some() {
            self.index += 1;
        }
        value
    }

    fn consume(&mut self, expected: &str) -> bool {
        if self.peek() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_plural_rule, Operand, RelationOperator};

    #[test]
    fn parses_english_plural_rule() {
        let rule = parse_plural_rule("i = 1 and v = 0 @integer 1").expect("parse");

        assert_eq!(rule.conditions.len(), 1);
        assert_eq!(rule.conditions[0].relations.len(), 2);
        assert_eq!(
            rule.conditions[0].relations[0].expression.operand,
            Operand::I
        );
    }

    #[test]
    fn parses_russian_plural_rule_with_modulo_and_ranges() {
        let rule =
            parse_plural_rule("v = 0 and i % 10 = 2..4 and i % 100 != 12..14").expect("parse");

        let relation = &rule.conditions[0].relations[1];
        assert_eq!(relation.expression.operand, Operand::I);
        assert_eq!(relation.expression.modulo, Some(10));
        assert_eq!(relation.ranges.ranges[0].start, 2);
        assert_eq!(relation.ranges.ranges[0].end, 4);
    }

    #[test]
    fn parses_arabic_plural_rule_with_or_and_comma_list() {
        let rule = parse_plural_rule("n % 100 = 3..10 or n = 103, 104").expect("parse");

        assert_eq!(rule.conditions.len(), 2);
        assert_eq!(rule.conditions[1].relations[0].ranges.ranges.len(), 2);
    }

    #[test]
    fn parses_in_within_and_negated_operators() {
        let rule = parse_plural_rule("n in 2..4 and v not within 1, 3").expect("parse");

        assert_eq!(
            rule.conditions[0].relations[0].operator,
            RelationOperator::In
        );
        assert_eq!(
            rule.conditions[0].relations[1].operator,
            RelationOperator::NotWithin
        );
    }
}
