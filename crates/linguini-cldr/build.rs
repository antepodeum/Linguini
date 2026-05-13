use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const OFFICIAL_CLDR_JSON_REPO: &str = "https://github.com/unicode-org/cldr-json";
const PLURALS_RELATIVE_PATH: &str = "cldr-json/cldr-core/supplemental/plurals.json";
const GENERATED_FILE: &str = "linguini_generated_plural_rules.rs";

fn main() {
    println!("cargo:rerun-if-env-changed=LINGUINI_CLDR_PLURALS_JSON");
    println!("cargo:rerun-if-env-changed=LINGUINI_CLDR_SOURCE_DIR");

    if let Err(error) = run() {
        panic!("failed to generate built-in CLDR plural rules: {error}");
    }
}

fn run() -> Result<(), String> {
    let plurals = plural_source_path()?;
    println!("cargo:rerun-if-changed={}", plurals.display());

    let source = fs::read_to_string(&plurals)
        .map_err(|error| format!("{}: {error}", plurals.display()))?;
    let generated = generate_plural_tables(&source)?;
    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(|error| error.to_string())?);
    let output = out_dir.join(GENERATED_FILE);
    fs::write(&output, generated).map_err(|error| format!("{}: {error}", output.display()))?;
    Ok(())
}

fn plural_source_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("LINGUINI_CLDR_PLURALS_JSON") {
        return Ok(PathBuf::from(path));
    }

    if let Ok(source_dir) = env::var("LINGUINI_CLDR_SOURCE_DIR") {
        let source_dir = PathBuf::from(source_dir);
        for candidate in [
            source_dir.join(PLURALS_RELATIVE_PATH),
            source_dir.join("cldr-core/supplemental/plurals.json"),
            source_dir.join("supplemental/plurals.json"),
        ] {
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
        return Err(format!(
            "LINGUINI_CLDR_SOURCE_DIR={} does not contain {PLURALS_RELATIVE_PATH}",
            source_dir.display()
        ));
    }

    download_official_plural_rules()
}

fn download_official_plural_rules() -> Result<PathBuf, String> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(|error| error.to_string())?);
    let source_dir = out_dir.join("cldr-json-source");
    if source_dir.exists() {
        fs::remove_dir_all(&source_dir)
            .map_err(|error| format!("{}: {error}", source_dir.display()))?;
    }

    let source_dir_arg = source_dir.to_string_lossy().into_owned();
    run_git(&[
        "clone",
        "--filter=blob:none",
        "--no-checkout",
        "--depth=1",
        OFFICIAL_CLDR_JSON_REPO,
        source_dir_arg.as_str(),
    ])?;
    run_git(&[
        "-C",
        source_dir_arg.as_str(),
        "sparse-checkout",
        "set",
        "--no-cone",
        PLURALS_RELATIVE_PATH,
    ])?;
    run_git(&["-C", source_dir_arg.as_str(), "checkout"])?;

    Ok(source_dir.join(PLURALS_RELATIVE_PATH))
}

fn run_git(args: &[&str]) -> Result<(), String> {
    let status = Command::new("git")
        .args(args)
        .status()
        .map_err(|error| format!("failed to execute git: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("git {} failed with status {status}", args.join(" ")))
    }
}

fn generate_plural_tables(source: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(source).map_err(|error| error.to_string())?;
    let cardinal = value
        .get("supplemental")
        .and_then(|value| value.get("plurals-type-cardinal"))
        .and_then(Value::as_object)
        .ok_or_else(|| "missing supplemental.plurals-type-cardinal".to_owned())?;

    let mut locales: Vec<_> = cardinal.iter().collect();
    locales.sort_by(|(left, _), (right, _)| left.cmp(right));

    let mut compiled_match_arms = String::new();
    let mut source_match_arms = String::new();
    let mut category_tables = String::new();
    let mut predicate_functions = String::new();
    let mut source_functions = String::new();

    for (locale, value) in locales {
        let object = value
            .as_object()
            .ok_or_else(|| format!("plural rules for locale `{locale}` are not an object"))?;
        let mut categories = Vec::new();
        for (key, value) in object {
            let Some(category) = key.strip_prefix("pluralRule-count-") else {
                continue;
            };
            let rule_source = value.as_str().ok_or_else(|| {
                format!("plural rule `{key}` for locale `{locale}` is not a string")
            })?;
            let rule = parse_plural_rule(rule_source)
                .map_err(|error| format!("{locale}/{category}: {error}"))?;
            categories.push((category.to_owned(), rule_source.to_owned(), rule));
        }
        if categories.is_empty() {
            return Err(format!("locale `{locale}` has no plural categories"));
        }
        categories.sort_by(|left, right| {
            category_rank(&left.0)
                .cmp(&category_rank(&right.0))
                .then_with(|| left.0.cmp(&right.0))
        });

        let const_name = const_name(locale);
        let source_function = format!("plural_rule_source_{}", rust_identifier(locale));
        compiled_match_arms.push_str(&format!(
            "        {} => Some(CompiledPluralRules {{ locale: {}, categories: PLURAL_CATEGORIES_{const_name} }}),\n",
            rust_string(locale),
            rust_string(locale)
        ));
        source_match_arms.push_str(&format!(
            "        {} => Some({source_function}()),\n",
            rust_string(locale)
        ));
        category_tables.push_str(&format!(
            "const PLURAL_CATEGORIES_{const_name}: &[CompiledPluralCategory] = &[\n"
        ));
        source_functions.push_str(&format!(
            "fn {source_function}() -> PluralRules {{\n    PluralRules {{\n        locale: {}.to_owned(),\n        categories: vec![\n",
            rust_string(locale)
        ));

        for (category, rule_source, rule) in &categories {
            let function = format!(
                "plural_{}_{}",
                rust_identifier(locale),
                rust_identifier(category)
            );
            category_tables.push_str(&format!(
                "    CompiledPluralCategory {{ category: {}, matches: {function} }},\n",
                rust_string(category)
            ));
            source_functions.push_str(&format!(
                "            PluralCategoryRule {{ category: {}.to_owned(), source: {}.to_owned(), rule: {} }},\n",
                rust_string(category),
                rust_string(rule_source),
                plural_rule_literal(rule)
            ));
            let parameter = if plural_rule_uses_operands(rule) {
                "operands"
            } else {
                "_operands"
            };
            let body = plural_rule_to_rust(rule);
            predicate_functions.push_str(&format!(
                "fn {function}({parameter}: &PluralOperands) -> bool {{\n    {body}\n}}\n\n",
            ));
        }
        category_tables.push_str("];\n\n");
        source_functions.push_str("        ],\n    }\n}\n\n");
    }

    Ok(format!(
        "// generated by crates/linguini-cldr/build.rs from Unicode CLDR plural rules\n\
         fn generated_plural_rules(locale: &str) -> Option<CompiledPluralRules> {{\n\
             match locale {{\n{compiled_match_arms}        _ => None,\n    }}\n}}\n\n\
         fn generated_plural_rule_sources(locale: &str) -> Option<PluralRules> {{\n\
             match locale {{\n{source_match_arms}        _ => None,\n    }}\n}}\n\n\
         fn integer_value(value: (f64, bool)) -> Option<u64> {{\n\
             if value.1 && value.0 >= 0.0 {{ Some(value.0 as u64) }} else {{ None }}\n\
         }}\n\n\
         {category_tables}{source_functions}{predicate_functions}"
    ))
}

fn category_rank(category: &str) -> usize {
    match category {
        "zero" => 0,
        "one" => 1,
        "two" => 2,
        "few" => 3,
        "many" => 4,
        "other" => 5,
        _ => 6,
    }
}

fn const_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn rust_identifier(value: &str) -> String {
    let mut identifier = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            identifier.push(character.to_ascii_lowercase());
        } else {
            identifier.push('_');
        }
    }
    if identifier
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        identifier.insert(0, '_');
    }
    identifier
}

fn rust_string(value: &str) -> String {
    format!("{value:?}")
}


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
            (0, end) => format!("{value} <= {end}"),
            (start, end) => format!("{value} >= {start} && {value} <= {end}"),
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
                format!("{value} >= {}f64 && {value} <= {}f64", range.start, range.end)
            }
        })
        .collect::<Vec<_>>()
        .join(" || ")
}
