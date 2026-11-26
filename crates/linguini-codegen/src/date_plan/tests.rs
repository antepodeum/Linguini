use super::*;

#[test]
fn selected_contexts_periods_and_field_demand_are_resolved_together() {
    let calendar = linguini_cldr::compiled_calendar_formatting("ru", "gregory").unwrap();
    let plan =
        DatePatternPlan::new("ru", "MMMM LLLL b B BBB 'm s S' zzzz yyyy G", &calendar).unwrap();
    assert_eq!(plan.fields, BTreeSet::from(['M', 'L', 'b', 'B', 'y', 'G']));
    assert!(plan.can_localize_whole_result);
    let fields = plan
        .parts
        .iter()
        .filter_map(|part| match part {
            DateFormatPart::Field(field) => Some(field),
            DateFormatPart::Literal(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(fields[0].symbols.names.as_ref().unwrap()[0], "января");
    assert_eq!(fields[1].symbols.names.as_ref().unwrap()[0], "январь");
    assert_eq!(plan.periods.len(), 2);
    assert_eq!(fields[2].period, Some(0));
    assert_eq!(fields[3].period, Some(1));
    assert_eq!(fields[4].period, Some(1));
    assert_eq!(plan.periods[0], day_period_plan(&calendar, 'b', 1).unwrap());
    assert_eq!(plan.periods[1], day_period_plan(&calendar, 'B', 1).unwrap());
    assert!(plan.parts.iter().any(|part| matches!(part, DateFormatPart::Literal(value) if value.contains("m s S") && value.contains("Всемирное координированное время"))));
}

#[test]
fn literal_and_symbol_digits_keep_localization_proofs_and_missing_data_is_structured() {
    let mut calendar = linguini_cldr::compiled_calendar_formatting("en", "gregory").unwrap();
    calendar.symbols = &[&[("eras.eraAbbr.1", "AD1"), ("eras.eraAbbr.0", "BC2")]];
    assert!(
        DatePatternPlan::new("en", "y", &calendar)
            .unwrap()
            .can_localize_whole_result
    );
    for pattern in ["y G", "y '9'"] {
        assert!(
            !DatePatternPlan::new("en", pattern, &calendar)
                .unwrap()
                .can_localize_whole_result
        );
    }
    let plan = DatePatternPlan::new("en", "'m s S'", &calendar).unwrap();
    assert!(plan.fields.is_empty());
    assert!(plan.periods.is_empty());
    assert_eq!(plan.parts, [DateFormatPart::Literal("m s S".into())]);
    assert_eq!(
        DatePatternPlan::new("probe", "MMMM", &calendar),
        Err(DatePlanError::MissingFieldSymbols {
            locale: "probe".into(),
            symbol: 'M',
            width: 4,
        })
    );
    assert!(matches!(
        DatePatternPlan::new("en", "é QQQQQQ", &calendar),
        Err(DatePlanError::Pattern(DatePatternError::UnsupportedField {
            offset: 3,
            ..
        }))
    ));
}

#[test]
fn all_source_selectable_pinned_gregorian_skeletons_resolve_their_symbol_plans() {
    let mut locales = BTreeSet::new();
    let mut checked = 0;
    for locale in linguini_cldr::locale_resolution_candidates() {
        let Some(dates) = linguini_cldr::compiled_date_formatting(locale) else {
            continue;
        };
        if !locales.insert(dates.locale) {
            continue;
        }
        let calendar =
            linguini_cldr::compiled_calendar_formatting(dates.locale, "gregory").unwrap();
        for &(skeleton, pattern) in calendar.skeletons {
            if skeleton.bytes().all(|byte| byte.is_ascii_alphabetic()) {
                DatePatternPlan::new(locale, pattern, &calendar)
                    .unwrap_or_else(|error| panic!("{locale}/{skeleton}: {error}"));
                checked += 1;
            }
        }
    }
    assert_eq!(locales.len(), 508);
    assert!(
        checked > 10_000,
        "insufficient pinned skeleton coverage: {checked}"
    );
}
