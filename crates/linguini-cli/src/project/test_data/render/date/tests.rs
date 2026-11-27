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
fn utc_date_inputs_preserve_clock_precision_and_cross_calendar_boundaries() {
    for (value, expected) in [
        (
            "2024-01-02T23:45:56.789+05:30",
            (2024, 1, 2, 2, 18, 15, 56, 789),
        ),
        (
            "2024-01-02T01:23:45.12-03:30",
            (2024, 1, 2, 2, 4, 53, 45, 120),
        ),
        (
            "2024-02-29T23:59:59.123456Z",
            (2024, 2, 29, 4, 23, 59, 59, 123),
        ),
        ("2024-01-02T12:34:56.0009Z", (2024, 1, 2, 2, 12, 34, 56, 0)),
        ("0000-01-01T00:00+00:01", (-1, 12, 31, 5, 23, 59, 0, 0)),
        ("9999-12-31T24:00-23:59", (10000, 1, 1, 6, 23, 59, 0, 0)),
    ] {
        let date = parse_date(value).unwrap();
        assert_eq!(
            (
                date.year,
                date.month,
                date.day,
                date.weekday,
                date.hour,
                date.minute,
                date.second,
                date.millisecond
            ),
            expected,
            "{value}"
        );
    }
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
        (
            "en",
            "2024-01-02T23:45:56.789+05:30",
            vec![("skeleton", "Hm")],
            "18:15",
        ),
        (
            "en",
            "2024-01-02T12:00:30.123Z",
            vec![("skeleton", "Bhm")],
            "12:00 noon",
        ),
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
        "2024-01-02T24:00:00.0009Z",
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
        "2024-01-02T00:00",
        "2024-01-02T00:00Z",
        "2024-01-02T12:00:30.123Z",
        "2024-01-02T12:01:00Z",
        "2024-01-02T23:45:56.789+05:30",
        "2024-01-02T01:23:45.12-03:30",
        "2024-02-29T23:59:59.123456Z",
        "2024-02-29T24:00:00.000Z",
        "2023-12-31T24:00",
        "2024-01-01T00:00+23:59",
        "2024-12-31T23:59-23:59",
        "0000-01-01T00:00+00:01",
        "0000-02-29T00:00+23:59",
        "0001-01-01T00:00+23:59",
        "0099-12-31T23:59-00:01",
        "9999-12-31T24:00-23:59",
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
    for value in [
        "2023-02-29T12:00Z",
        "2024-02-30T12:00Z",
        "2024-00-01T12:00Z",
        "2024-01-02T25:00Z",
        "2024-01-02T24:01Z",
        "2024-01-02T24:00:01Z",
        "2024-01-02T24:00:00.0009Z",
        "2024-01-02T12:60Z",
        "2024-01-02T12:00:60Z",
        "2024-01-02T12:00:00.Z",
        "2024-01-02T12:00:00.1.2Z",
        "2024-01-02T12:00.1Z",
        "2024-01-02T12:00+24:00",
        "2024-01-02T12:00+01:60",
        "2024-01-02T12:00+0100",
        "2024-01-02T12:00Zx",
        "2024-01-02t12:00Z",
        "2024-01-02T12:00z",
        "2024-01-02T1:00Z",
        "2024-01-02T12:0Z",
        "2024-01-02T12:00:0Z",
        "2024-01-02Té:00Z",
        "2024-01-02Z",
        "0",
    ] {
        assert!(
            matches!(
                format_date("en", value, &[]),
                Err(RenderError::InvalidDate { .. })
            ),
            "{value}"
        );
        cases.push(serde_json::json!({ "locale":"en", "index":0, "value":value, "invalid":true }));
    }
    super::super::parity::assert_generated_samples_in_time_zones(
        &schema,
        &source,
        &locales,
        &cases,
        &["UTC", "America/Los_Angeles", "Asia/Kolkata"],
    );
}

#[test]
fn numeric_date_samples_keep_their_input_kind_through_forwarded_calls() {
    use super::super::{Renderer, SampleValue};
    use linguini_ir::{lower_locale, lower_schema};
    use linguini_syntax::{parse_locale, parse_schema};
    use std::collections::BTreeMap;

    let schema_source = concat!(
        "enum Mode { receipt }\n",
        "type Instant = Date\n",
        "sample0(value: Instant)\n",
        "sample1(value: Instant)\n",
        "sample2(value: Instant)\n",
        "sample3(value: Instant)\n",
        "sample4(value: Instant, mode: Mode)\n",
        "sample5(value: Instant)\n",
    );
    let locale_source = concat!(
        "impl Mode { receipt { form label(when: Instant) { _ => ",
        "{when @date(skeleton = \"yMMMd\")} / {when @date(skeleton = \"Hms\")} } } }\n",
        "fn Echo(text: String) { _ => {text} }\n",
        "fn Receipt(value: Instant) { _ => ",
        "{value @date(skeleton = \"yMMMd\")} / {value @date(skeleton = \"Hms\")} }\n",
        "fn Forward(value: Instant) { _ => {Receipt(value)} }\n",
        "sample0 = {value @date(skeleton = \"yMMMd\")} / {value @date(skeleton = \"Hms\")}\n",
        "sample1 = {Receipt(value)}\n",
        "sample2 = {Forward(value)}\n",
        "sample3 = {fn(saved: value) { _ => {Receipt(saved)} }}\n",
        "sample4 = {mode.label(value)}\n",
        "sample5 = {Echo(value @date(skeleton = \"yMMMd\"))} / ",
        "{Echo(value @date(skeleton = \"Hms\"))}\n",
    );
    let schema = lower_schema(&parse_schema(schema_source).unwrap());
    let module = lower_locale(&parse_locale(locale_source).unwrap());
    let mut cases = Vec::new();
    for locale in ["en", "ru", "ar-EG", "ff-Adlm"] {
        let renderer = Renderer::new(&schema, &module, locale);
        for (milliseconds, iso) in [
            (0.0, "1970-01-01T00:00:00Z"),
            (0.99, "1970-01-01T00:00:00Z"),
            (-0.99, "1970-01-01T00:00:00Z"),
            (-1.0, "1969-12-31T23:59:59.999Z"),
            (1_715_852_096_789.0, "2024-05-16T09:34:56.789Z"),
            (-62_167_219_200_000.0, "0000-01-01T00:00:00Z"),
        ] {
            for index in 0..6 {
                let name = format!("sample{index}");
                let expected = renderer
                    .render_message(
                        &name,
                        &BTreeMap::from([
                            ("mode".into(), SampleValue::String("receipt".into())),
                            ("value".into(), SampleValue::String(iso.into())),
                        ]),
                    )
                    .unwrap();
                let actual = renderer
                    .render_message(
                        &name,
                        &BTreeMap::from([
                            ("mode".into(), SampleValue::String("receipt".into())),
                            (
                                "value".into(),
                                SampleValue::Number(milliseconds.to_string()),
                            ),
                        ]),
                    )
                    .unwrap();
                assert_eq!(actual, expected, "{locale}/{name}/{milliseconds}");
                cases.push(serde_json::json!({"locale":locale,"index":index,"args":if index == 4 { serde_json::json!([milliseconds,"receipt"]) } else { serde_json::json!([milliseconds]) },"expected":expected}));
            }
        }
        for index in 0..6 {
            assert!(matches!(
                renderer.render_message(
                    &format!("sample{index}"),
                    &BTreeMap::from([
                        ("mode".into(), SampleValue::String("receipt".into())),
                        ("value".into(), SampleValue::String("0".into()))
                    ])
                ),
                Err(RenderError::InvalidDate { .. })
            ));
        }
    }
    super::super::parity::assert_generated_samples_in_time_zones(
        schema_source,
        locale_source,
        &["en", "ru", "ar-EG", "ff-Adlm"],
        &cases,
        &["UTC", "America/Los_Angeles", "Asia/Kolkata"],
    );
}

#[test]
fn numeric_date_fields_match_time_clip_across_the_javascript_date_range() {
    let output = std::process::Command::new("node")
        .args([
            "--input-type=module",
            "-e",
            r#"
const values = [0,-0,0.99,-0.99,1,-1,86400000,-86400000,
  -62167219200000,-2203977600000,951782400000,
  -8640000000000000,8640000000000000,-8640000000000001,8640000000000001,
  NaN,Infinity,-Infinity];
let seed = 1;
for(let index=0;index<1024;index++) {
  seed = (Math.imul(seed,1664525)+1013904223)>>>0;
  values.push((seed/0xffffffff*2-1)*8640000000000000);
}
console.log(JSON.stringify(values.map(value=>{
  const date = new Date(value);
  return {value:String(value),fields:Number.isFinite(date.getTime())?
    [date.getUTCFullYear(),date.getUTCMonth()+1,date.getUTCDate(),date.getUTCDay(),
     date.getUTCHours(),date.getUTCMinutes(),date.getUTCSeconds(),date.getUTCMilliseconds()]:null};
})));
"#,
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cases: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    for case in cases {
        let value = case["value"].as_str().unwrap();
        let result = super::input::parse_timestamp(value);
        if case["fields"].is_null() {
            assert!(
                matches!(result, Err(RenderError::InvalidDate { .. })),
                "{case}"
            );
        } else {
            let date = result.unwrap_or_else(|error| panic!("{case}: {error}"));
            assert_eq!(
                serde_json::json!([
                    date.year,
                    date.month,
                    date.day,
                    date.weekday,
                    date.hour,
                    date.minute,
                    date.second,
                    date.millisecond
                ]),
                case["fields"],
                "{case}"
            );
        }
    }
}

#[test]
fn native_year_week_previews_match_built_messages_across_week_outcomes() {
    use super::super::{Renderer, SampleValue};
    use linguini_ir::{lower_locale, lower_schema};
    use linguini_syntax::{parse_locale, parse_schema};
    use std::collections::BTreeMap;
    let schema_source = "type WeekDate = Date @date(skeleton = \"yw\")\nsample0(value: WeekDate)\nsample1(value: Date)\n";
    let source = concat!(
        "fn Receipt(value: Date) { _ => {value @date(skeleton = \"yw\")} }\n",
        "sample0 = {value}\nsample1 = {Receipt(value)}\n",
    );
    let schema = lower_schema(&parse_schema(schema_source).unwrap());
    let module = lower_locale(&parse_locale(source).unwrap());
    let locales = [
        "en", "en-GB", "fo", "fa", "ar-EG", "ff-Adlm", "mt", "fil", "hy",
    ];
    let mut cases = Vec::new();
    for locale in locales {
        let renderer = Renderer::new(&schema, &module, locale);
        for week in 0..54 {
            let milliseconds = 1_577_836_800_000_i64 + week * 604_800_000;
            for index in 0..2 {
                let expected = renderer
                    .render_message(
                        &format!("sample{index}"),
                        &BTreeMap::from([(
                            "value".into(),
                            SampleValue::Number(milliseconds.to_string()),
                        )]),
                    )
                    .unwrap();
                cases.push(serde_json::json!({"locale":locale,"index":index,"value":milliseconds,"expected":expected}));
            }
        }
        for value in [
            "0000-01-01",
            "2021-01-01",
            "2021-01-04",
            "2020-12-31T23:00-03:00",
        ] {
            for index in 0..2 {
                let expected = renderer
                    .render_message(
                        &format!("sample{index}"),
                        &BTreeMap::from([("value".into(), SampleValue::String(value.into()))]),
                    )
                    .unwrap();
                cases.push(serde_json::json!({"locale":locale,"index":index,"value":value,"expected":expected}));
            }
        }
    }
    super::super::parity::assert_generated_samples_in_time_zones(
        schema_source,
        source,
        &locales,
        &cases,
        &["UTC", "America/Los_Angeles", "Asia/Kolkata"],
    );
}
