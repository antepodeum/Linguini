use super::*;

fn result<'a, T>(plan: &'a IntegerPluralPlan<T>, value: &PluralOperands) -> &'a T {
    plan.branches
        .iter()
        .find(|branch| branch.rule.matches(value))
        .map(|branch| &branch.result)
        .unwrap_or(&plan.fallback)
}

#[test]
fn bounded_outcomes_remove_fractional_work_and_redundant_categories() {
    let rules = linguini_cldr::built_in_plural_rules("ru").unwrap();
    let mut projected = Vec::new();
    let plan = IntegerPluralPlan::for_rules(&rules, 1, 53, |category| {
        projected.push(category.to_owned());
        category == "one"
    })
    .unwrap();
    assert_eq!(
        projected
            .iter()
            .filter(|name| name.as_str() == "other")
            .count(),
        1
    );
    assert_eq!(plan.branches.len(), 1);
    assert!(!plan.fallback);
    assert_eq!(plan.result_for_integer(0), None);
    assert_eq!(plan.result_for_integer(54), None);
    let relations = &plan.branches[0].rule.conditions[0].relations;
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].expression.modulo, Some(10));
    assert_eq!(relations[1].expression.modulo, None);
    assert!(relations
        .iter()
        .all(|relation| relation.expression.operand == Operand::I));
    for value in 1..=53 {
        let operands = PluralOperands::parse(&value.to_string()).unwrap();
        assert_eq!(
            *result(&plan, &operands),
            rules.category_for_operands(&operands) == "one"
        );
    }
    let constant = IntegerPluralPlan::for_rules(&rules, 1, 53, |_| "same").unwrap();
    assert!(constant.branches.is_empty());
    assert_eq!(constant.fallback, "same");
}

#[test]
fn bounded_merging_preserves_overlapping_ordered_rules_and_limits() {
    use linguini_cldr::{parse_plural_rule, PluralCategoryRule};
    let rules = PluralRules {
        locale: "overlap".into(),
        categories: [
            ("one", "i = 1..3"),
            ("few", "i = 3..5"),
            ("many", "i = 1..8"),
        ]
        .into_iter()
        .map(|(category, source)| PluralCategoryRule {
            category: category.into(),
            source: source.into(),
            rule: parse_plural_rule(source).unwrap(),
        })
        .collect(),
    };
    for project in [
        |category: &str| category == "few",
        |category: &str| category != "few",
    ] {
        let plan = IntegerPluralPlan::for_rules(&rules, 0, 10, project).unwrap();
        for value in 0..=10 {
            let operands = PluralOperands::parse(&value.to_string()).unwrap();
            assert_eq!(
                *result(&plan, &operands),
                project(rules.category_for_operands(&operands))
            );
        }
    }
    assert!(IntegerPluralPlan::for_rules(&rules, 2, 1, |_| ()).is_none());
    assert!(IntegerPluralPlan::for_rules(&rules, 0, 4096, |_| ()).is_none());
    assert!(IntegerPluralPlan::for_rules(&rules, 0, u64::MAX, |_| ()).is_none());
    assert!(IntegerPluralPlan::for_rules(&rules, u64::MAX, u64::MAX, |_| ()).is_some());
    let mut invalid = rules;
    invalid.categories[0].rule.conditions[0].relations[0]
        .expression
        .modulo = Some(0);
    assert!(IntegerPluralPlan::for_rules(&invalid, 1, 53, |_| ()).is_none());
}

#[test]
fn bounded_integer_categories_match_all_pinned_plural_families() {
    let mut checked = std::collections::BTreeSet::new();
    for locale in linguini_cldr::locale_resolution_candidates() {
        let Some(rules) = linguini_cldr::built_in_plural_rules(locale) else {
            continue;
        };
        if !checked.insert(rules.locale.clone()) {
            continue;
        }
        let plan = IntegerPluralPlan::for_rules(&rules, 1, 53, str::to_owned).unwrap();
        for value in 1..=53 {
            let operands = PluralOperands::parse(&value.to_string()).unwrap();
            assert_eq!(
                result(&plan, &operands),
                rules.category_for_operands(&operands),
                "{locale}/{value}"
            );
        }
    }
    assert!(
        checked.len() > 100,
        "insufficient pinned plural families: {}",
        checked.len()
    );
}
