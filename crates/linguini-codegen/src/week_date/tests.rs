use super::*;
use linguini_cldr::{built_in_plural_rules, compiled_calendar_formatting, PluralOperands};

#[test]
fn every_pinned_week_family_resolves_only_live_patterns_and_exact_outcomes() {
    let mut locales = std::collections::BTreeSet::new();
    let mut branching = 0;
    for locale in linguini_cldr::locale_resolution_candidates() {
        let Some(calendar) = compiled_calendar_formatting(locale, "gregory") else {
            continue;
        };
        let Some(rules) = built_in_plural_rules(locale) else {
            continue;
        };
        // Week conventions and plurals can differ even when calendar symbols share a record.
        if !locales.insert(locale) {
            continue;
        }
        let plan = WeekDatePlan::new(locale, &calendar, &rules)
            .unwrap_or_else(|error| panic!("{locale}: {error}"));
        branching += usize::from(!plan.selection.branches.is_empty());
        let mut used = std::collections::BTreeSet::new();
        let mut expected_patterns = BTreeMap::new();
        for value in 1..=53 {
            let operands = PluralOperands::parse(&value.to_string()).unwrap();
            let category = rules.category_for_operands(&operands);
            let key = format!("yw-count-{category}");
            let source = calendar
                .skeletons
                .iter()
                .find(|(name, _)| *name == key)
                .or_else(|| {
                    calendar
                        .skeletons
                        .iter()
                        .find(|(name, _)| *name == "yw-count-other")
                })
                .or_else(|| calendar.skeletons.iter().find(|(name, _)| *name == "yw"))
                .unwrap()
                .1;
            let expected = expected_patterns
                .entry(source)
                .or_insert_with(|| DatePatternPlan::for_week(locale, source, &calendar).unwrap());
            let index = plan
                .selection
                .branches
                .iter()
                .find(|branch| branch.rule.matches(&operands))
                .map(|branch| branch.result)
                .unwrap_or(plan.selection.fallback);
            used.insert(index);
            assert_eq!(&plan.patterns[index], expected, "{locale}/{value}");
            assert!(expected
                .fields
                .iter()
                .all(|field| matches!(field, 'Y' | 'w')));
        }
        assert_eq!(used.len(), plan.patterns.len(), "{locale}: unused pattern");
    }
    assert!(
        locales.len() > 500,
        "insufficient locale coverage: {}",
        locales.len()
    );
    assert_eq!(
        branching, 9,
        "pinned locale tags with different live week wording"
    );
}

#[test]
fn equal_patterns_omit_classification_and_missing_families_fail() {
    let calendar = compiled_calendar_formatting("en", "gregory").unwrap();
    let rules = built_in_plural_rules("en").unwrap();
    let plan = WeekDatePlan::new("en", &calendar, &rules).unwrap();
    assert_eq!(plan.patterns.len(), 1);
    assert!(plan.selection.branches.is_empty());
    let mut missing = calendar;
    missing.skeletons = &[];
    assert!(matches!(
        WeekDatePlan::new("en", &missing, &rules),
        Err(DatePlanError::Pattern(
            DatePatternError::MissingSkeleton { .. }
        ))
    ));
}

#[test]
fn canonical_week_requests_reject_conflicts_and_other_calendar_kinds() {
    use linguini_ir::{IrFormatter, IrFormatterArgument, IrFormatterKind};
    let mut formatter = IrFormatter {
        kind: IrFormatterKind::Date,
        arguments: vec![IrFormatterArgument {
            name: "skeleton".into(),
            value: "yw".into(),
        }],
    };
    assert!(
        week_date_request_plan("en", &crate::FormatterRequest::new(&formatter))
            .unwrap()
            .is_some()
    );
    formatter.kind = IrFormatterKind::Currency;
    assert!(
        week_date_request_plan("en", &crate::FormatterRequest::new(&formatter))
            .unwrap()
            .is_none()
    );
    formatter.kind = IrFormatterKind::Date;
    formatter.arguments.push(IrFormatterArgument {
        name: "style".into(),
        value: "short".into(),
    });
    assert!(matches!(
        week_date_request_plan("en", &crate::FormatterRequest::new(&formatter)),
        Err(DatePlanError::Pattern(
            DatePatternError::ConflictingSkeletonStyles
        ))
    ));
    formatter.arguments[0].value = "yMMMd".into();
    assert!(
        week_date_request_plan("en", &crate::FormatterRequest::new(&formatter))
            .unwrap()
            .is_none()
    );
    let mut calendar = compiled_calendar_formatting("en", "gregory").unwrap();
    calendar.calendar = "buddhist";
    assert!(WeekDatePlan::new("en", &calendar, &built_in_plural_rules("en").unwrap()).is_err());
}
