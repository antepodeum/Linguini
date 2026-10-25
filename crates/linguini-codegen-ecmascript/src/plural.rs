//! Build-time CLDR plural-rule lowering for canonical runtime templates.

use linguini_cldr::{Operand, PluralRules, Relation, RelationOperator};
use minijinja::context;

pub fn generate_plural_function(function_name: &str, rules: &PluralRules) -> String {
    generate_plural_function_with_visibility(function_name, rules, true)
}

pub(crate) fn generate_plural_function_with_visibility(
    function_name: &str,
    rules: &PluralRules,
    exported: bool,
) -> String {
    let plan = linguini_codegen::PluralDecisionPlan::for_rules(rules);
    let integer_tail_digits =
        linguini_codegen::PluralRequirements::for_rules(rules).integer_tail_digits();
    let integer_tail = integer_tail_digits.is_some();
    let mut defined_moduli = Vec::new();
    let mut defined_operations = Vec::new();
    let mut take_operations = |requirements: &linguini_codegen::PluralRequirements| {
        requirements
            .operands()
            .iter()
            .flat_map(|operand| operand_operations(*operand, integer_tail))
            .copied()
            .filter(|operation| {
                if defined_operations.contains(operation) {
                    false
                } else {
                    defined_operations.push(*operation);
                    true
                }
            })
            .collect::<Vec<_>>()
    };
    let mut take_moduli = |requirements: &linguini_codegen::PluralRequirements| {
        requirements
            .moduli()
            .iter()
            .filter_map(|expression| {
                let name = modulo_name(expression.operand, expression.modulo, integer_tail_digits)?;
                if defined_moduli.contains(&name) {
                    return None;
                }
                defined_moduli.push(name.clone());
                Some(context! {
                    name, binding => operand_binding(expression.operand, integer_tail),
                    modulo => expression.modulo,
                    bigint => is_bigint(expression.operand, integer_tail_digits),
                })
            })
            .collect::<Vec<_>>()
    };
    let guards = plan
        .guards
        .iter()
        .map(|relation| {
            let requirements = linguini_codegen::PluralRequirements::for_relations([relation]);
            let operations = take_operations(&requirements);
            let moduli = take_moduli(&requirements);
            context! { relation => relation_context(relation, integer_tail_digits), operations, moduli }
        })
        .collect::<Vec<_>>();
    let categories = plan
        .categories
        .iter()
        .map(|entry| {
            let requirements = linguini_codegen::PluralRequirements::for_relations(
                entry
                    .rule
                    .conditions
                    .iter()
                    .flat_map(|condition| &condition.relations),
            );
            let operations = take_operations(&requirements);
            let moduli = take_moduli(&requirements);
            let conditions = entry
                .rule
                .conditions
                .iter()
                .map(|condition| {
                    condition
                        .relations
                        .iter()
                        .map(|relation| relation_context(relation, integer_tail_digits))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            context! { name => &entry.category, conditions, operations, moduli }
        })
        .collect::<Vec<_>>();
    crate::templates::render(
        "plural",
        context! {
            name => function_name, exported, categories, guards, integer_tail_digits,
        },
    )
}

fn relation_context(relation: &Relation, integer_tail_digits: Option<usize>) -> minijinja::Value {
    let ranges = relation
        .ranges
        .ranges
        .iter()
        .map(|range| {
            context! { start => range.start, end => range.end }
        })
        .collect::<Vec<_>>();
    context! {
        operand => operand_name(relation.expression.operand),
        binding => operand_binding(relation.expression.operand, integer_tail_digits.is_some()),
        modulo_name => modulo_name(relation.expression.operand, relation.expression.modulo, integer_tail_digits),
        bigint => is_bigint(relation.expression.operand, integer_tail_digits),
        allow_fraction => matches!(relation.operator, RelationOperator::Within | RelationOperator::NotWithin),
        negated => matches!(relation.operator, RelationOperator::NotEqual | RelationOperator::NotIn | RelationOperator::NotWithin),
        ranges,
    }
}

fn is_bigint(operand: Operand, integer_tail_digits: Option<usize>) -> bool {
    match operand {
        Operand::N | Operand::I => !integer_tail_digits.is_some_and(|digits| digits <= 15),
        Operand::F | Operand::T => true,
        _ => false,
    }
}

fn modulo_name(
    operand: Operand,
    modulo: Option<u64>,
    integer_tail_digits: Option<usize>,
) -> Option<String> {
    modulo
        .filter(|value| *value != 0)
        .filter(|value| {
            // The selected decimal suffix is already smaller than its largest modulus.
            !(matches!(operand, Operand::N | Operand::I)
                && integer_tail_digits.is_some_and(|digits| *value == 10_u64.pow(digits as u32)))
        })
        .map(|value| {
            format!(
                "{}Modulo{value}",
                operand_binding(operand, integer_tail_digits.is_some())
            )
        })
}

fn operand_operations(operand: Operand, integer_tail: bool) -> &'static [&'static str] {
    match operand {
        Operand::N if integer_tail => &["digits", "integer_tail", "fraction", "has_fraction"],
        Operand::I if integer_tail => &["digits", "integer_tail"],
        Operand::N => &[
            "digits",
            "integer",
            "integer_value",
            "fraction",
            "has_fraction",
        ],
        Operand::I => &["digits", "integer", "integer_value"],
        Operand::V => &["visible_fraction_digits"],
        Operand::W => &[
            "digits",
            "fraction",
            "trimmed_fraction",
            "significant_fraction_digits",
        ],
        Operand::F => &["digits", "fraction", "fraction_value"],
        Operand::T => &[
            "digits",
            "fraction",
            "trimmed_fraction",
            "significant_fraction_value",
        ],
        Operand::C | Operand::E => &["compact_exponent"],
    }
}

fn operand_binding(operand: Operand, integer_tail: bool) -> &'static str {
    match operand {
        Operand::N | Operand::I if integer_tail => "integerTail",
        Operand::N | Operand::I => "integerValue",
        Operand::V => "visibleFractionDigits",
        Operand::W => "significantFractionDigits",
        Operand::F => "fractionValue",
        Operand::T => "significantFractionValue",
        Operand::C | Operand::E => "compactExponent",
    }
}

fn operand_name(operand: Operand) -> &'static str {
    match operand {
        Operand::N => "n",
        Operand::I => "i",
        Operand::V => "v",
        Operand::W => "w",
        Operand::F => "f",
        Operand::T => "t",
        Operand::C => "c",
        Operand::E => "e",
    }
}

#[cfg(test)]
mod tests;
