use super::*;
use linguini_cldr::{built_in_plural_rules, compiled_calendar_formatting, PluralOperands};

#[test]
fn pinned_month_week_families_specialize_their_exact_reachable_domain() {
    let mut checked = 0;
    let mut branching = std::collections::BTreeSet::new();
    for locale in linguini_cldr::locale_resolution_candidates() {
        let Some(calendar) = compiled_calendar_formatting(locale, "gregory") else {
            continue;
        };
        let Some(rules) = built_in_plural_rules(locale) else {
            continue;
        };
        let plan = WeekDatePlan::for_skeleton(locale, &calendar, &rules, "MMMMW").unwrap();
        checked += 1;
        if !plan.selection.branches.is_empty() {
            branching.insert(*locale);
        }
        let week = linguini_cldr::compiled_week_data(locale).unwrap();
        assert_eq!(plan.selection.minimum, u64::from(week.min_days == 1));
        assert_eq!(
            plan.selection.maximum,
            u64::from((37 - week.min_days) / 7 + 1)
        );
        let mut used = std::collections::BTreeSet::new();
        for value in plan.selection.minimum..=plan.selection.maximum {
            let operands = PluralOperands::parse(&value.to_string()).unwrap();
            let category = rules.category_for_operands(&operands);
            let key = format!("MMMMW-count-{category}");
            let source = calendar
                .skeletons
                .iter()
                .find(|(name, _)| *name == key)
                .or_else(|| {
                    calendar
                        .skeletons
                        .iter()
                        .find(|(name, _)| *name == "MMMMW-count-other")
                })
                .unwrap()
                .1;
            let index = *plan.selection.result_for_integer(value).unwrap();
            used.insert(index);
            let expected = DatePatternPlan::for_week(locale, source, &calendar).unwrap();
            assert_eq!(plan.patterns[index], expected, "{locale}/{value}");
            assert!(expected
                .fields
                .iter()
                .all(|field| matches!(field, 'M' | 'L' | 'W')));
        }
        assert_eq!(
            used.len(),
            plan.patterns.len(),
            "{locale}: unused month-week pattern"
        );
    }
    assert!(checked > 500);
    assert!(branching.contains("mt"));
    assert!(branching.contains("sq"));
}

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

#[test]
fn constant_families_do_not_require_unavailable_cardinal_rules() {
    use linguini_ir::{IrFormatter, IrFormatterArgument, IrFormatterKind};
    let mut checked = std::collections::BTreeSet::new();
    for &locale in linguini_cldr::locale_resolution_candidates()
        .iter()
        .chain([&"sa"])
    {
        if built_in_plural_rules(locale).is_some()
            || crate::GregorianWeekYearPlan::for_locale(locale).is_none()
        {
            continue;
        }
        let Some(calendar) = compiled_calendar_formatting(locale, "gregory") else {
            continue;
        };
        for skeleton in ["yw", "MMMMW"] {
            let request = crate::FormatterRequest::new(&IrFormatter {
                kind: IrFormatterKind::Date,
                arguments: vec![IrFormatterArgument {
                    name: "skeleton".into(),
                    value: skeleton.into(),
                }],
            });
            let plan = week_date_request_plan(locale, &request)
                .unwrap_or_else(|error| panic!("{locale}/{skeleton}: {error}"))
                .unwrap();
            assert!(plan.selection.branches.is_empty());
            assert_eq!(plan.patterns.len(), 1);
            let prefix = format!("{skeleton}-count-");
            for &(name, pattern) in calendar.skeletons {
                if name.starts_with(&prefix) {
                    assert_eq!(
                        plan.patterns[0],
                        DatePatternPlan::for_week(locale, pattern, &calendar).unwrap(),
                        "{locale}/{name}"
                    );
                }
            }
            checked.insert(locale);
        }
    }
    assert!(checked.contains("sa"));
    assert!(
        checked.len() > 50,
        "insufficient rule-free locale coverage: {}",
        checked.len()
    );
}

#[test]
fn missing_rules_require_equal_compiled_variants_and_a_fallback() {
    let mut calendar = compiled_calendar_formatting("en", "gregory").unwrap();
    calendar.skeletons = &[("yw-count-one", "w 'of' Y"), ("yw-count-other", "w' of 'Y")];
    let plan = WeekDatePlan::compile("en", &calendar, None, "yw").unwrap();
    assert_eq!(plan.patterns.len(), 1);
    assert!(plan.selection.branches.is_empty());
    calendar.skeletons = &[
        ("yw-count-one", "w 'one' Y"),
        ("yw-count-other", "w 'many' Y"),
    ];
    assert!(matches!(
        WeekDatePlan::compile("en", &calendar, None, "yw"),
        Err(DatePlanError::Pattern(
            DatePatternError::MissingSkeleton { .. }
        ))
    ));
    calendar.skeletons = &[("yw-count-one", "w 'only' Y")];
    assert!(matches!(
        WeekDatePlan::compile("en", &calendar, None, "yw"),
        Err(DatePlanError::Pattern(
            DatePatternError::MissingSkeleton { .. }
        ))
    ));
}
