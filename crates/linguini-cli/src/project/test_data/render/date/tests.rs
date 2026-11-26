use super::*;

fn arguments(values: &[(&str, &str)]) -> Vec<IrFormatterArgument> {
    values
        .iter()
        .map(|(name, value)| IrFormatterArgument {
            name: (*name).into(),
            value: (*value).into(),
        })
        .collect()
}

#[test]
fn date_samples_use_selected_requests_and_preserve_input_validation() {
    for (locale, value, options, expected) in [
        ("en", "2024-01-02", vec![], "Jan 2, 2024"),
        ("gd", "2021-01-01", vec![("skeleton", "yMMM")], "Faoi 2020"),
        ("gd", "0000-01-01", vec![("skeleton", "yMMM")], "Faoi -1"),
        (
            "en",
            "2024-01-02",
            vec![("time_style", "short")],
            "12:00\u{202f}AM",
        ),
        (
            "en",
            "2024-01-02",
            vec![("skeleton", "Bhm")],
            "12:00 midnight",
        ),
        ("en", "0000-02-29", vec![("skeleton", "yQQQ")], "Q1 1"),
    ] {
        assert_eq!(
            format_date(locale, value, &arguments(&options)).unwrap(),
            expected
        );
    }
    for value in [
        "2023-02-29",
        "0001-02-29",
        "2024-02-30",
        "2024-13-01",
        "2024-00-12",
        "2024-01-00",
        "+001-01-02",
        "2024-é-01",
        "2024-01-2",
        "2024-01-020",
        "2024-01-02T00:00:00Z",
    ] {
        assert!(
            matches!(
                format_date("en", value, &[]),
                Err(RenderError::InvalidDate { .. })
            ),
            "{value}"
        );
    }
    for (options, diagnostic) in [
        (
            vec![("skeleton", "notASkeleton")],
            "no pinned Gregorian date pattern",
        ),
        (
            vec![("skeleton", "yMMM"), ("style", "short")],
            "cannot be combined",
        ),
        (
            vec![("skeleton", "yw-count-one")],
            "unsupported date field `w`",
        ),
    ] {
        assert!(format_date("en", "2024-01-02", &arguments(&options))
            .unwrap_err()
            .to_string()
            .contains(diagnostic));
    }
}

#[test]
fn date_samples_render_every_source_selectable_pinned_skeleton() {
    let mut locales = std::collections::BTreeSet::new();
    let mut count = 0;
    for locale in linguini_cldr::locale_resolution_candidates() {
        let Some(dates) = compiled_date_formatting(locale) else {
            continue;
        };
        if !locales.insert(dates.locale) {
            continue;
        }
        let calendar = compiled_calendar_formatting(dates.locale, "gregory").unwrap();
        for &(skeleton, _) in calendar.skeletons {
            if skeleton.bytes().all(|byte| byte.is_ascii_alphabetic()) {
                format_date(
                    dates.locale,
                    "2024-01-02",
                    &arguments(&[("skeleton", skeleton)]),
                )
                .unwrap_or_else(|error| panic!("{} {skeleton}: {error}", dates.locale));
                count += 1;
            }
        }
    }
    assert_eq!(locales.len(), 508);
    assert!(count > 10_000, "insufficient skeleton coverage: {count}");
}

#[test]
fn native_date_samples_match_real_generated_modules_across_requests_and_locales() {
    let locales = [
        "en", "en-GB", "gd", "ru", "fa", "ar-EG", "fi", "sms", "zh", "ja", "ff-Adlm", "ccp",
    ];
    let dates = [
        "0000-01-01",
        "0000-02-29",
        "0001-01-02",
        "2021-01-01",
        "2021-01-04",
        "2024-01-02",
        "2026-05-13",
        "9999-12-31",
    ];
    let mut requests = vec![Vec::new()];
    for style in ["full", "long", "medium", "short"] {
        requests.push(arguments(&[("style", style)]));
        requests.push(arguments(&[("time_style", style)]));
        for time in ["full", "long", "medium", "short"] {
            requests.push(arguments(&[("style", style), ("time_style", time)]));
        }
    }
    for skeleton in ["yMMM", "yMMMd", "yQQQ", "Hm", "Bhm"] {
        requests.push(arguments(&[("skeleton", skeleton)]));
    }
    let mut schema = String::new();
    let mut source = String::new();
    for (index, options) in requests.iter().enumerate() {
        schema.push_str(&format!("sample{index}(value: Date)\n"));
        let options = options
            .iter()
            .map(|argument| format!("{} = {:?}", argument.name, argument.value))
            .collect::<Vec<_>>()
            .join(", ");
        let options = if options.is_empty() {
            String::new()
        } else {
            format!("({options})")
        };
        source.push_str(&format!("sample{index} = {{value @date{options}}}\n"));
    }
    let mut cases = Vec::new();
    for locale in locales {
        for (index, options) in requests.iter().enumerate() {
            for value in dates {
                let expected = format_date(locale, value, options)
                    .unwrap_or_else(|error| panic!("{locale}/{index}/{value}: {error}"));
                cases.push(serde_json::json!({ "locale":locale, "index":index, "value":value, "expected":expected }));
            }
        }
    }
    super::super::parity::assert_generated_samples(&schema, &source, &locales, &cases);
}
