fn plural_rule_literal(rule: &PluralRule) -> String {
    format!(
        "PluralRule {{ conditions: vec![{}] }}",
        rule.conditions
            .iter()
            .map(condition_literal)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn condition_literal(condition: &Condition) -> String {
    format!(
        "Condition {{ relations: vec![{}] }}",
        condition
            .relations
            .iter()
            .map(relation_literal)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn relation_literal(relation: &Relation) -> String {
    format!(
        "Relation {{ expression: {}, operator: {}, ranges: {} }}",
        operand_expression_literal(&relation.expression),
        relation_operator_literal(relation.operator),
        range_list_literal(&relation.ranges)
    )
}

fn operand_expression_literal(expression: &OperandExpression) -> String {
    format!(
        "OperandExpression {{ operand: {}, modulo: {} }}",
        operand_literal(expression.operand),
        option_u64_literal(expression.modulo)
    )
}

fn range_list_literal(ranges: &RangeList) -> String {
    format!(
        "RangeList {{ ranges: vec![{}] }}",
        ranges
            .ranges
            .iter()
            .map(|range| format!(
                "Range {{ start: {}, end: {} }}",
                range.start, range.end
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn option_u64_literal(value: Option<u64>) -> String {
    value.map_or_else(|| "None".to_owned(), |value| format!("Some({value})"))
}

fn operand_literal(operand: Operand) -> &'static str {
    match operand {
        Operand::N => "Operand::N",
        Operand::I => "Operand::I",
        Operand::V => "Operand::V",
        Operand::W => "Operand::W",
        Operand::F => "Operand::F",
        Operand::T => "Operand::T",
        Operand::C => "Operand::C",
        Operand::E => "Operand::E",
    }
}

fn relation_operator_literal(operator: RelationOperator) -> &'static str {
    match operator {
        RelationOperator::Equal => "RelationOperator::Equal",
        RelationOperator::NotEqual => "RelationOperator::NotEqual",
        RelationOperator::In => "RelationOperator::In",
        RelationOperator::NotIn => "RelationOperator::NotIn",
        RelationOperator::Within => "RelationOperator::Within",
        RelationOperator::NotWithin => "RelationOperator::NotWithin",
    }
}

#[derive(Debug, Clone)]
struct PluralRule {
    conditions: Vec<Condition>,
}

#[derive(Debug, Clone)]
struct Condition {
    relations: Vec<Relation>,
}

#[derive(Debug, Clone)]
struct Relation {
    expression: OperandExpression,
    operator: RelationOperator,
    ranges: RangeList,
}

#[derive(Debug, Clone, Copy)]
enum RelationOperator {
    Equal,
    NotEqual,
    In,
    NotIn,
    Within,
    NotWithin,
}

#[derive(Debug, Clone)]
struct OperandExpression {
    operand: Operand,
    modulo: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
enum Operand {
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
struct RangeList {
    ranges: Vec<Range>,
}

#[derive(Debug, Clone, Copy)]
struct Range {
    start: u64,
    end: u64,
}

fn parse_plural_rule(source: &str) -> Result<PluralRule, String> {
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

fn plural_rule_uses_operands(rule: &PluralRule) -> bool {
    !rule.conditions.is_empty()
}

fn plural_rule_to_rust(rule: &PluralRule) -> String {
    if rule.conditions.is_empty() {
        return "true".to_owned();
    }
    rule.conditions
        .iter()
        .map(condition_to_rust)
        .collect::<Vec<_>>()
        .join(" || ")
}

fn condition_to_rust(condition: &Condition) -> String {
    condition
        .relations
        .iter()
        .map(relation_to_rust)
        .collect::<Vec<_>>()
        .join(" && ")
}

fn relation_to_rust(relation: &Relation) -> String {
    let expression = operand_expression_to_rust(&relation.expression);
    match relation.operator {
        RelationOperator::Equal | RelationOperator::In => {
            let ranges = ranges_to_rust(&relation.ranges.ranges, "value");
            format!("integer_value({expression}).is_some_and(|value| {ranges})")
        }
        RelationOperator::NotEqual | RelationOperator::NotIn => {
            let ranges = ranges_to_rust(&relation.ranges.ranges, "value");
            format!("!integer_value({expression}).is_some_and(|value| {ranges})")
        }
        RelationOperator::Within => {
            let ranges = ranges_to_rust_float(&relation.ranges.ranges, "value");
            format!("{{ let value = ({expression}).0; value >= 0.0 && ({ranges}) }}")
        }
        RelationOperator::NotWithin => {
            let ranges = ranges_to_rust_float(&relation.ranges.ranges, "value");
            format!("{{ let value = ({expression}).0; !(value >= 0.0 && ({ranges})) }}")
        }
    }
}

fn operand_expression_to_rust(expression: &OperandExpression) -> String {
    let operand = match expression.operand {
        Operand::N => "(operands.n.parse::<f64>().unwrap_or(operands.i as f64), operands.v == 0)",
        Operand::I => "(operands.i as f64, true)",
        Operand::V => "(operands.v as f64, true)",
        Operand::W => "(operands.w as f64, true)",
        Operand::F => "(operands.f as f64, true)",
        Operand::T => "(operands.t as f64, true)",
        Operand::C => "(operands.c as f64, true)",
        Operand::E => "(operands.e as f64, true)",
    };
    expression.modulo.map_or_else(
        || operand.to_owned(),
        |modulo| format!("{{ let value = {operand}; (value.0 % {modulo}f64, value.1) }}"),
    )
}

fn ranges_to_rust(ranges: &[Range], value: &str) -> String {
    ranges
        .iter()
        .map(|range| match (range.start, range.end) {
            (start, end) if start == end => format!("{value} == {start}"),
            (start, end) => format!("({start}..={end}).contains(&{value})"),
        })
        .collect::<Vec<_>>()
        .join(" || ")
}

fn ranges_to_rust_float(ranges: &[Range], value: &str) -> String {
    ranges
        .iter()
        .map(|range| {
            if range.start == range.end {
                format!("{value} == {}f64", range.start)
            } else {
                format!("({}f64..={}f64).contains(&{value})", range.start, range.end)
            }
        })
        .collect::<Vec<_>>()
        .join(" || ")
}
