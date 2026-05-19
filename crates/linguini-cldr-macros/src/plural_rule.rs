#[derive(Debug, Clone)]
pub(crate) struct PluralRule {
    pub(crate) conditions: Vec<Condition>,
}

#[derive(Debug, Clone)]
pub(crate) struct Condition {
    pub(crate) relations: Vec<Relation>,
}

#[derive(Debug, Clone)]
pub(crate) struct Relation {
    pub(crate) expression: OperandExpression,
    pub(crate) operator: RelationOperator,
    pub(crate) ranges: RangeList,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RelationOperator {
    Equal,
    NotEqual,
    In,
    NotIn,
    Within,
    NotWithin,
}

#[derive(Debug, Clone)]
pub(crate) struct OperandExpression {
    pub(crate) operand: Operand,
    pub(crate) modulo: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Operand {
    N,
    I,
    V,
    W,
    F,
    T,
    C,
    E,
}

#[derive(Debug, Clone)]
pub(crate) struct RangeList {
    pub(crate) ranges: Vec<Range>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Range {
    pub(crate) start: u64,
    pub(crate) end: u64,
}

pub(crate) fn parse_plural_rule(source: &str) -> Result<PluralRule, String> {
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

fn parse_relation(source: &str) -> Result<Relation, String> {
    let tokens = tokenize(source);
    let mut cursor = Cursor::new(tokens);
    let expression = parse_operand_expression(&mut cursor)?;
    let operator = parse_operator(&mut cursor)?;
    let ranges = parse_range_list(&mut cursor)?;

    if let Some(token) = cursor.peek() {
        return Err(format!("unexpected token `{token}`"));
    }

    Ok(Relation {
        expression,
        operator,
        ranges,
    })
}

fn parse_operand_expression(cursor: &mut Cursor) -> Result<OperandExpression, String> {
    let operand = match cursor.next().as_deref() {
        Some("n") => Operand::N,
        Some("i") => Operand::I,
        Some("v") => Operand::V,
        Some("w") => Operand::W,
        Some("f") => Operand::F,
        Some("t") => Operand::T,
        Some("c") => Operand::C,
        Some("e") => Operand::E,
        Some(token) => return Err(format!("expected plural operand, got `{token}`")),
        None => return Err("expected plural operand".to_owned()),
    };

    let modulo = if cursor.consume("%") || cursor.consume("mod") {
        Some(parse_number(cursor.next().as_deref())?)
    } else {
        None
    };

    Ok(OperandExpression { operand, modulo })
}

fn parse_operator(cursor: &mut Cursor) -> Result<RelationOperator, String> {
    if cursor.consume("=") || cursor.consume("is") {
        if cursor.consume("not") {
            return Ok(RelationOperator::NotEqual);
        }
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
        return Err("expected relation operator after `not`".to_owned());
    }

    if cursor.consume("in") {
        return Ok(RelationOperator::In);
    }

    if cursor.consume("within") {
        return Ok(RelationOperator::Within);
    }

    Err("expected relation operator".to_owned())
}

fn parse_range_list(cursor: &mut Cursor) -> Result<RangeList, String> {
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

fn parse_number(value: Option<&str>) -> Result<u64, String> {
    let Some(value) = value else {
        return Err("expected number".to_owned());
    };
    value
        .parse()
        .map_err(|_| format!("expected number, got `{value}`"))
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
