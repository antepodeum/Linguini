//! Backend-independent demand analysis for compiled CLDR plural predicates.

use linguini_cldr::{Operand, OperandExpression, PluralCategoryRule, PluralRules, Relation};

/// Ordered category decisions with a shared conjunction prefix evaluated once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralDecisionPlan {
    pub guards: Vec<Relation>,
    pub categories: Vec<PluralCategoryRule>,
}

impl PluralDecisionPlan {
    pub fn for_rules(rules: &PluralRules) -> Self {
        let mut categories = rules
            .categories
            .iter()
            .filter(|category| category.category != "other")
            .cloned()
            .collect::<Vec<_>>();
        let mut terms = categories
            .iter()
            .flat_map(|category| &category.rule.conditions);
        let mut guards = terms
            .next()
            .map(|term| term.relations.clone())
            .unwrap_or_default();
        let mut term_count = 1;
        for term in terms {
            term_count += 1;
            let shared = guards
                .iter()
                .zip(&term.relations)
                .take_while(|(a, b)| a == b)
                .count();
            guards.truncate(shared);
        }
        if term_count < 2 {
            guards.clear();
        }
        for term in categories
            .iter_mut()
            .flat_map(|category| &mut category.rule.conditions)
        {
            term.relations.drain(..guards.len());
        }
        Self { guards, categories }
    }
}

/// Operands and reusable modulo operations needed by the non-fallback rules.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PluralRequirements {
    operands: Vec<Operand>,
    moduli: Vec<OperandExpression>,
    integer_tail_digits: Option<usize>,
}

impl PluralRequirements {
    pub fn for_rules(rules: &PluralRules) -> Self {
        Self::for_relations(
            rules
                .categories
                .iter()
                .filter(|category| category.category != "other")
                .flat_map(|category| &category.rule.conditions)
                .flat_map(|condition| &condition.relations),
        )
    }

    pub fn for_relations<'a>(relations: impl IntoIterator<Item = &'a Relation>) -> Self {
        let mut requirements = Self::default();
        let mut integer_tail_digits = Some(0);
        for relation in relations {
            let operand = relation.expression.operand;
            if matches!(operand, Operand::N | Operand::I) {
                integer_tail_digits = integer_tail_digits.and_then(|digits| {
                    decimal_modulo_digits(relation.expression.modulo)
                        .map(|needed| digits.max(needed))
                });
            }
            if !requirements.operands.contains(&operand) {
                requirements.operands.push(operand);
            }
            if relation.expression.modulo.is_some_and(|modulo| modulo != 0)
                && !requirements.moduli.contains(&relation.expression)
            {
                requirements.moduli.push(relation.expression.clone());
            }
        }
        requirements.integer_tail_digits = integer_tail_digits.filter(|digits| *digits > 0);
        requirements
    }

    pub fn operands(&self) -> &[Operand] {
        &self.operands
    }
    pub fn moduli(&self) -> &[OperandExpression] {
        &self.moduli
    }
    pub fn needs(&self, operand: Operand) -> bool {
        self.operands.contains(&operand)
    }

    /// A bounded decimal integer suffix is sufficient only when every `n`/`i` relation
    /// uses a power-of-ten modulus. Fraction conditions on `n` remain independently required.
    pub fn integer_tail_digits(&self) -> Option<usize> {
        self.integer_tail_digits
    }
}

fn decimal_modulo_digits(modulo: Option<u64>) -> Option<usize> {
    let mut modulo = modulo?;
    let mut digits = 0;
    while modulo > 1 && modulo % 10 == 0 {
        modulo /= 10;
        digits += 1;
    }
    (modulo == 1 && digits > 0).then_some(digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_only_used_operands_and_deduplicates_moduli() {
        let rules = linguini_cldr::built_in_plural_rules("ru").unwrap();
        let plan = PluralDecisionPlan::for_rules(&rules);
        assert_eq!(plan.guards.len(), 1);
        assert_eq!(plan.guards[0].expression.operand, Operand::V);
        assert!(plan
            .categories
            .iter()
            .flat_map(|category| &category.rule.conditions)
            .flat_map(|condition| &condition.relations)
            .all(|relation| relation.expression.operand == Operand::I));
        for locale in ["en", "ar", "ja"] {
            assert!(PluralDecisionPlan::for_rules(
                &linguini_cldr::built_in_plural_rules(locale).unwrap()
            )
            .guards
            .is_empty());
        }
        let requirements = PluralRequirements::for_rules(&rules);
        assert_eq!(requirements.integer_tail_digits(), Some(2));
        assert_eq!(requirements.operands(), [Operand::V, Operand::I]);
        assert_eq!(
            requirements.moduli(),
            [
                OperandExpression {
                    operand: Operand::I,
                    modulo: Some(10)
                },
                OperandExpression {
                    operand: Operand::I,
                    modulo: Some(100)
                },
            ]
        );
        assert_eq!(requirements, PluralRequirements::for_rules(&rules));
        assert_eq!(
            PluralRequirements::for_rules(&linguini_cldr::built_in_plural_rules("ja").unwrap()),
            PluralRequirements::default()
        );
    }

    #[test]
    fn integer_suffix_requires_only_decimal_moduli_across_all_integer_relations() {
        use linguini_cldr::parse_plural_rule;
        for (source, digits) in [
            ("i % 10 = 1", Some(1)),
            ("n % 100 within 1..2 and i % 10 = 1", Some(2)),
            ("i % 10000000000000000000 = 1", Some(19)),
            ("i % 100 = 1 and v = 0", Some(2)),
            ("i % 100 = 1 or n = 1", None),
            ("i = 1 or n % 100 = 1", None),
            ("i % 10 = 1 and n % 3 = 1", None),
            ("i % 0 = 1", None),
            ("i % 1 = 0", None),
            ("n % 20 = 1", None),
            ("f % 100 = 1 or v = 2", None),
            ("", None),
        ] {
            let rule = parse_plural_rule(source).unwrap();
            let requirements = PluralRequirements::for_relations(
                rule.conditions
                    .iter()
                    .flat_map(|condition| &condition.relations),
            );
            assert_eq!(requirements.integer_tail_digits(), digits, "{source}");
        }
    }
}
