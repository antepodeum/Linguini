use linguini_cldr::{Operand, PluralRule, PluralRules, RelationOperator};

pub fn generate_plural_function(function_name: &str, rules: &PluralRules) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "export function {function_name}(value: number | string): string {{\n"
    ));
    output.push_str("  const operands = pluralOperands(value);\n");

    for category in rules
        .categories
        .iter()
        .filter(|category| category.category != "other")
    {
        output.push_str(&format!(
            "  if ({}) return \"{}\";\n",
            rule_expression(&category.rule),
            category.category
        ));
    }

    output.push_str("  return \"other\";\n");
    output.push_str("}\n\n");
    output.push_str(PLURAL_OPERANDS_HELPER);
    output
}

fn rule_expression(rule: &PluralRule) -> String {
    if rule.conditions.is_empty() {
        return "true".to_owned();
    }

    rule.conditions
        .iter()
        .map(|condition| {
            condition
                .relations
                .iter()
                .map(|relation| {
                    let value = operand_expression(relation.expression.operand);
                    let value = if let Some(modulo) = relation.expression.modulo {
                        format!("({value} % {modulo})")
                    } else {
                        value
                    };
                    relation_expression(&value, relation.operator, &relation.ranges.ranges)
                })
                .collect::<Vec<_>>()
                .join(" && ")
        })
        .map(|condition| format!("({condition})"))
        .collect::<Vec<_>>()
        .join(" || ")
}

fn relation_expression(
    value: &str,
    operator: RelationOperator,
    ranges: &[linguini_cldr::Range],
) -> String {
    let contains = ranges
        .iter()
        .map(|range| {
            if range.start == range.end {
                format!("{value} === {}", range.start)
            } else {
                format!("({value} >= {} && {value} <= {})", range.start, range.end)
            }
        })
        .collect::<Vec<_>>()
        .join(" || ");

    match operator {
        RelationOperator::Equal | RelationOperator::In | RelationOperator::Within => {
            format!("({contains})")
        }
        RelationOperator::NotEqual | RelationOperator::NotIn | RelationOperator::NotWithin => {
            format!("!({contains})")
        }
    }
}

fn operand_expression(operand: Operand) -> String {
    match operand {
        Operand::N => "operands.n",
        Operand::I => "operands.i",
        Operand::V => "operands.v",
        Operand::W => "operands.w",
        Operand::F => "operands.f",
        Operand::T => "operands.t",
        Operand::C => "operands.c",
        Operand::E => "operands.e",
    }
    .to_owned()
}

const PLURAL_OPERANDS_HELPER: &str = include_str!("templates/plural-operands.runtime.ts");

#[cfg(test)]
mod tests {
    use super::generate_plural_function;
    use linguini_cldr::built_in_plural_rules;

    #[test]
    fn generated_plural_function_snapshot_is_stable() {
        let rules = built_in_plural_rules("ru").expect("rules");
        let output = generate_plural_function("pluralRu", &rules);

        assert_eq!(
            output,
            include_str!("../../../tests/fixtures/golden/snapshots/codegen-ts-plural-ru.ts")
        );
    }
}
