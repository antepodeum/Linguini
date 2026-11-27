//! Compile plural outcomes for a proven finite domain of non-negative integers.

use std::collections::BTreeMap;

use linguini_cldr::{
    Condition, Operand, PluralOperands, PluralRule, PluralRules, RelationOperator,
};

/// A category predicate with its already projected application result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerPluralBranch<Result> {
    pub rule: PluralRule,
    pub result: Result,
}

/// Direct integer predicates for a caller-proven domain. No decimal parsing is required in
/// generated code: only `i` and its demanded remainders survive. Equivalent results can share
/// a branch even when their CLDR categories differ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerPluralPlan<Result> {
    pub minimum: u64,
    pub maximum: u64,
    pub branches: Vec<IntegerPluralBranch<Result>>,
    pub fallback: Result,
}

impl<Result: Clone + Eq> IntegerPluralPlan<Result> {
    /// Resolve a domain value from the already compiled integer predicates.
    pub fn result_for_integer(&self, value: u64) -> Option<&Result> {
        if !(self.minimum..=self.maximum).contains(&value) {
            return None;
        }
        let operands = PluralOperands {
            n: value.to_string(),
            i: value,
            v: 0,
            w: 0,
            f: 0,
            t: 0,
            c: 0,
            e: 0,
        };
        Some(
            self.branches
                .iter()
                .find(|branch| branch.rule.matches(&operands))
                .map(|branch| &branch.result)
                .unwrap_or(&self.fallback),
        )
    }

    /// Project each distinct category once, then prove pruning/merging over the whole domain.
    /// Return `None` for empty domains, more than 4,096 values, or zero-modulus relations.
    /// Backends must preserve the domain and choose an exact integer representation.
    pub fn for_rules(
        rules: &PluralRules,
        minimum: u64,
        maximum: u64,
        mut project: impl FnMut(&str) -> Result,
    ) -> Option<Self> {
        if maximum.checked_sub(minimum)?.checked_add(1)? > 4096
            || rules
                .categories
                .iter()
                .flat_map(|category| &category.rule.conditions)
                .flat_map(|condition| &condition.relations)
                .any(|relation| relation.expression.modulo == Some(0))
        {
            return None;
        }
        let domain = (minimum..=maximum)
            .map(|value| {
                PluralOperands::parse(&value.to_string()).expect("non-negative integer operands")
            })
            .collect::<Vec<_>>();
        let mut results = BTreeMap::new();
        for category in rules
            .categories
            .iter()
            .map(|category| category.category.as_str())
            .chain(std::iter::once("other"))
        {
            results.entry(category).or_insert_with(|| project(category));
        }
        let expected = domain
            .iter()
            .map(|value| results[rules.category_for_operands(value)].clone())
            .collect::<Vec<_>>();
        let mut remaining = domain.iter().collect::<Vec<_>>();
        let mut branches = Vec::new();
        let mut fallback = results["other"].clone();
        for category in rules
            .categories
            .iter()
            .filter(|category| category.category != "other")
        {
            let rule = integer_rule(&category.rule, &domain, maximum);
            let matched = remaining.iter().filter(|value| rule.matches(value)).count();
            if matched == 0 {
                continue;
            }
            if matched == remaining.len() {
                fallback = results[category.category.as_str()].clone();
                break;
            }
            remaining.retain(|value| !rule.matches(value));
            branches.push(IntegerPluralBranch {
                rule,
                result: results[category.category.as_str()].clone(),
            });
        }
        // An ordered rule can overlap another category. Verify each merge rather than
        // assuming category disjointness or moving a result across an intervening branch.
        let mut index = 0;
        while index < branches.len() {
            let mut next = index + 1;
            while next < branches.len() {
                if branches[index].result != branches[next].result {
                    next += 1;
                    continue;
                }
                let mut candidate = branches.clone();
                let later = candidate.remove(next);
                for condition in later.rule.conditions {
                    if !candidate[index].rule.conditions.contains(&condition) {
                        candidate[index].rule.conditions.push(condition);
                    }
                }
                if preserves_results(&candidate, &fallback, &domain, &expected) {
                    branches = candidate;
                } else {
                    next += 1;
                }
            }
            index += 1;
        }
        for index in (0..branches.len()).rev() {
            let mut candidate = branches.clone();
            candidate.remove(index);
            if preserves_results(&candidate, &fallback, &domain, &expected) {
                branches = candidate;
            }
        }
        Some(Self {
            minimum,
            maximum,
            branches,
            fallback,
        })
    }
}

fn integer_rule(rule: &PluralRule, domain: &[PluralOperands], maximum: u64) -> PluralRule {
    let mut conditions = Vec::new();
    for condition in &rule.conditions {
        let mut relations = Vec::new();
        let mut possible = true;
        for relation in &condition.relations {
            let single = PluralRule {
                conditions: vec![Condition {
                    relations: vec![relation.clone()],
                }],
            };
            let matched = domain.iter().filter(|value| single.matches(value)).count();
            if matched == 0 {
                possible = false;
                break;
            }
            if matched == domain.len() {
                continue;
            }
            let mut relation = relation.clone();
            // All fractional and compact operands are zero; any surviving relation uses n/i.
            debug_assert!(matches!(
                relation.expression.operand,
                Operand::N | Operand::I
            ));
            relation.expression.operand = Operand::I;
            if relation
                .expression
                .modulo
                .is_some_and(|modulus| modulus > maximum)
            {
                relation.expression.modulo = None;
            }
            relation.operator = match relation.operator {
                RelationOperator::In | RelationOperator::Within => RelationOperator::Equal,
                RelationOperator::NotIn | RelationOperator::NotWithin => RelationOperator::NotEqual,
                operator => operator,
            };
            if !relations.contains(&relation) {
                relations.push(relation);
            }
        }
        let condition = Condition { relations };
        if possible && !conditions.contains(&condition) {
            conditions.push(condition);
        }
    }
    PluralRule { conditions }
}

fn preserves_results<Result: Eq>(
    branches: &[IntegerPluralBranch<Result>],
    fallback: &Result,
    domain: &[PluralOperands],
    expected: &[Result],
) -> bool {
    domain.iter().zip(expected).all(|(value, expected)| {
        branches
            .iter()
            .find(|branch| branch.rule.matches(value))
            .map(|branch| &branch.result)
            .unwrap_or(fallback)
            == expected
    })
}

#[cfg(test)]
mod tests;
