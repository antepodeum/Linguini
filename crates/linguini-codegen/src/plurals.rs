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
        for relation in relations {
            let operand = relation.expression.operand;
            if !requirements.operands.contains(&operand) {
                requirements.operands.push(operand);
            }
            if relation.expression.modulo.is_some_and(|modulo| modulo != 0)
                && !requirements.moduli.contains(&relation.expression)
            {
                requirements.moduli.push(relation.expression.clone());
            }
        }
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
}
