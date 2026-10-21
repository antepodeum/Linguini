use super::super::formatters::{formatter_binding, FormatterRequest};
use super::*;
use linguini_cldr::{NumberFormatData, NumberPadding, NumberPatternPart};
use linguini_ir::{IrFormatter, IrFormatterArgument, IrFormatterKind};

fn request(kind: IrFormatterKind, arguments: &[(&str, &str)]) -> FormatterRequest {
    FormatterRequest::new(&IrFormatter {
        kind,
        arguments: arguments
            .iter()
            .map(|(name, value)| IrFormatterArgument {
                name: (*name).to_owned(),
                value: (*value).to_owned(),
            })
            .collect(),
    })
}
fn execute(script: &str) -> String {
    let mut child = Command::new("node")
        .args(["--input-type=module", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node.js is required");
    child
        .stdin
        .take()
        .expect("Node stdin")
        .write_all(script.as_bytes())
        .expect("write generated code");
    let output = child.wait_with_output().expect("wait for Node");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("UTF-8 output")
        .trim()
        .to_owned()
}
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn generated_number_runtime_receives_complete_pattern_semantics() {
    let part = NumberPatternPart {
        prefix: "pre",
        suffix: "%",
        min_integer_digits: 1,
        min_fraction_digits: 0,
        max_fraction_digits: 2,
        primary_group_size: Some(3),
        secondary_group_size: Some(2),
        min_significant_digits: Some(2),
        max_significant_digits: Some(4),
        rounding_increment: Some("5"),
        exponent_digits: Some(2),
        exponent_sign_always: true,
        scale: 100,
        padding: Some(NumberPadding {
            character: 'x',
            width: 12,
            position: NumberPaddingPosition::BeforeSuffix,
        }),
    };
    let pattern = NumberPattern {
        positive: part,
        negative: None,
    };
    let numbers = NumberFormatData {
        locale: "test",
        numbering_system: "latn",
        digits: "0123456789",
        decimal_symbol: ".",
        group_symbol: ",",
        decimal_pattern: pattern,
        percent_pattern: pattern,
    };

    let plan = number_plan(&pattern, None, &numbers, None);
    let generated = crate::templates::render(
        "formatters/number",
        minijinja::context! { name => "formatTest", plan, exported => false },
    );
    let helpers = crate::templates::render(
        "number_helpers",
        minijinja::context! { scale => true, scientific => true, significant => true, significant_padding => true },
    );
    assert!(!generated.contains("formatGeneratedNumber"));
    assert!(!generated.contains("undefined"));
    let script = format!("{generated}{helpers}console.log(JSON.stringify([formatTest('12.345'), formatTest('-12.345'), formatTest('0')]));");
    assert_eq!(
        execute(&script),
        "[\"pre1.235E+03%\",\"-pre1.235E+03%\",\"pre0.0E+00x%\"]"
    );

    let runtime = formatter_data_declaration("en", &[request(IrFormatterKind::Number, &[])], false);
    for unused in [
        "scaleGeneratedDecimal",
        "roundGeneratedSignificant",
        "scientificGeneratedDecimal",
        "padGeneratedNumber",
        "formatGeneratedNumber",
        "localizeGeneratedDigits",
    ] {
        assert!(!runtime.contains(unused), "unused operation {unused}");
    }
    assert_eq!(execute(&format!("{runtime}console.log(JSON.stringify([formatNumber('-0'), formatNumber(1234.56789), formatNumber(12345678901234567890n)]));")), "[\"-0\",\"1,234.568\",\"12,345,678,901,234,567,890\"]");
}

#[test]
fn numeric_operations_are_selected_independently_and_execute_without_unused_helpers() {
    for (operation, expected) in [
        ("plain", "12.346"),
        ("scale", "1,234.567"),
        ("scientific", "1.235E01"),
        ("significant", "12.3"),
        ("significant_padding", "12.34567"),
    ] {
        let numbers = compiled_number_formatting("en").unwrap();
        let mut pattern = numbers.decimal_pattern;
        match operation {
            "scale" => pattern.positive.scale = 100,
            "scientific" => pattern.positive.exponent_digits = Some(2),
            "significant" => pattern.positive.max_significant_digits = Some(3),
            "significant_padding" => {
                pattern.positive.min_significant_digits = Some(4);
                pattern.positive.max_fraction_digits = 5;
            }
            _ => {}
        }
        let plan = number_plan(&pattern, None, &numbers, None);
        let formatter = crate::templates::render(
            "formatters/number",
            minijinja::context! {
                name => "formatTest", plan, exported => false,
            },
        );
        let helpers = crate::templates::render(
            "number_helpers",
            minijinja::context! {
                scale => operation == "scale", scientific => operation == "scientific",
                significant => operation == "significant", significant_padding => operation == "significant_padding",
            },
        );
        for (name, selected) in [
            ("scaleGeneratedDecimal", "scale"),
            ("scientificGeneratedDecimal", "scientific"),
            ("roundGeneratedSignificant", "significant"),
            ("padGeneratedSignificant", "significant_padding"),
        ] {
            assert_eq!(
                helpers.contains(&format!("function {name}(")),
                operation == selected,
                "{operation}: {name}"
            );
        }
        assert_eq!(
            execute(&format!(
                "{formatter}{helpers}console.log(formatTest('12.34567'));"
            )),
            expected,
            "{operation}"
        );
        if operation == "significant_padding" {
            assert_eq!(
                execute(&format!(
                    "{formatter}{helpers}console.log(formatTest('12'));"
                )),
                "12.00"
            );
        }
    }
}

#[test]
fn currency_increment_is_specialized_before_runtime_execution() {
    let numbers = compiled_number_formatting("en").unwrap();
    let mut pattern = compiled_currency_formatting("en").unwrap().standard_pattern;
    pattern.positive.min_fraction_digits = 2;
    pattern.positive.max_fraction_digits = 2;
    let plan = number_plan(&pattern, Some("CHF"), &numbers, Some("5"));
    let formatter = crate::templates::render(
        "formatters/number",
        minijinja::context! {
            name => "formatCash", plan, exported => false,
        },
    );
    let helpers = crate::templates::render(
        "number_helpers",
        minijinja::context! {
            scale => false, scientific => false, significant => false, significant_padding => false,
        },
    );
    assert_eq!(execute(&format!("{formatter}{helpers}console.log(JSON.stringify(['1.23','1.22','-1.23'].map(formatCash)));")), r#"["CHF1.25","CHF1.20","-CHF1.25"]"#);
}

#[test]
fn generated_javascript_formatter_runtime_snapshot_is_stable() {
    let output = formatter_data_declaration(
        "ru",
        &[
            request(IrFormatterKind::Number, &[]),
            request(IrFormatterKind::Currency, &[]),
            request(IrFormatterKind::Date, &[]),
        ],
        true,
    );
    let snapshot = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/codegen-js-formatters-ru.js");
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&snapshot, &output).expect("write JavaScript formatter snapshot");
    }

    assert!(!output.contains("value: GeneratedNumeric"));
    assert!(!output.contains("type GeneratedNumeric ="));
    assert_eq!(
        output,
        std::fs::read_to_string(snapshot).expect("read JavaScript formatter snapshot")
    );
}

#[test]
fn generated_javascript_formatters_execute_pinned_locale_corpus() {
    let cases = [
        (
            "en",
            "[\"1,234,567.89\",\"ZZZ1,234,567.89\",\"1/2/24, 1:04\u{202f}PM\"]",
        ),
        (
            "ru",
            "[\"1\u{a0}234\u{a0}567,89\",\"1\u{a0}234\u{a0}567,89\u{a0}ZZZ\",\"02.01.2024, 13:04\"]",
        ),
        (
            "ar",
            "[\"1,234,567.89\",\"\u{200f}1,234,567.89\u{a0}ZZZ\",\"2\u{200f}/1\u{200f}/2024\u{60c} 1:04 PM\"]",
        ),
        (
            "hi",
            "[\"12,34,567.89\",\"ZZZ12,34,567.89\",\"2/1/24, 1:04 PM\"]",
        ),
    ];
    for (locale, expected) in cases {
        let currency = request(IrFormatterKind::Currency, &[("code", "ZZZ")]);
        let date = request(
            IrFormatterKind::Date,
            &[("style", "short"), ("time_style", "short")],
        );
        let generated = formatter_data_declaration(
            locale,
            &[
                request(IrFormatterKind::Number, &[]),
                currency.clone(),
                date.clone(),
            ],
            true,
        );
        assert!(!generated.contains("Intl."));
        assert!(!generated.contains("switch (options"));
        let currency = formatter_binding(&currency);
        let date = formatter_binding(&date);
        let script = format!(
            r#"{generated}
console.log(JSON.stringify([formatNumber("1234567.89"), {currency}("1234567.89"), {date}("2024-01-02T13:04:05Z")]));
"#
        );

        let mut child = Command::new("node")
            .args(["--input-type=module", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Node.js is required to execute generated formatter runtimes");
        child
            .stdin
            .take()
            .expect("Node stdin")
            .write_all(script.as_bytes())
            .expect("write generated formatter runtime to Node");
        let output = child.wait_with_output().expect("wait for Node");
        assert!(
            output.status.success(),
            "generated {locale} formatter runtime failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout)
                .expect("formatter output is UTF-8")
                .trim(),
            expected,
            "generated {locale} formatter output drifted"
        );
    }
}

#[test]
fn generated_date_runtime_uses_utc_fields_only() {
    let dates = linguini_cldr::compiled_date_formatting("en").expect("English date data");
    let emitted = [
        dates.date_formats.full,
        dates.date_formats.long,
        dates.date_formats.medium,
        dates.date_formats.short,
    ]
    .map(|pattern| date_pattern_expression(pattern, &dates))
    .join("\n");

    assert!(emitted.contains("date.getUTCFullYear()"));
    assert!(emitted.contains("date.getUTCMonth()"));
    assert!(emitted.contains("date.getUTCDate()"));
    assert!(emitted.contains("date.getUTCDay()"));
    for local_getter in [
        "date.getFullYear()",
        "date.getMonth()",
        "date.getDate()",
        "date.getDay()",
    ] {
        assert!(
            !emitted.contains(local_getter),
            "generated date pattern used host-local getter {local_getter}"
        );
    }
}

#[test]
fn generated_date_runtime_composes_cldr_date_and_time_widths() {
    let dates = linguini_cldr::compiled_date_formatting("en").expect("English date data");
    let mut requests = Vec::new();
    for style in ["full", "long", "medium", "short"] {
        requests.push(request(
            IrFormatterKind::Date,
            &[("style", style), ("time_style", style)],
        ));
        requests.push(request(IrFormatterKind::Date, &[("time_style", style)]));
    }
    let emitted = formatter_data_declaration("en", &requests, false);
    assert!(emitted.contains("date.getUTCHours() % 12 || 12"));
    assert!(emitted.contains("date.getUTCMinutes()"));
    assert!(emitted.contains("date.getUTCSeconds()"));
    assert!(!emitted.contains("options"));
    assert!(!emitted.contains("switch ("));
    for local in ["date.getHours()", "date.getMinutes()", "date.getSeconds()"] {
        assert!(!emitted.contains(local));
    }
    let short = formatter_binding(&request(
        IrFormatterKind::Date,
        &[("style", "short"), ("time_style", "short")],
    ));
    assert_eq!(
        execute(&format!(
            "{emitted}console.log({short}('2024-01-02T13:04:05Z'));"
        )),
        "1/2/24, 1:04\u{202f}PM"
    );
    let _ = dates;
}

#[test]
fn generated_date_runtime_rejects_invalid_values() {
    let emitted = formatter_data_declaration("en", &[request(IrFormatterKind::Date, &[])], false);

    assert!(emitted.contains("function coerceDate(value)"));
    assert!(emitted.contains("date = createUTCDate(year, month, day);"));
    assert!(emitted.contains("date = new Date(hasTimeZone ? value : `${value}Z`);"));
    assert!(emitted.contains("function createUTCDate(year, month, day)"));
    assert!(emitted.contains("date.setUTCFullYear(year, month - 1, day);"));
    assert!(emitted.contains("date.getUTCFullYear() !== year"));
    assert!(emitted.contains("date.getUTCMonth() !== month - 1"));
    assert!(emitted.contains("date.getUTCDate() !== day"));
    assert!(emitted.contains("if (!Number.isFinite(date.getTime()))"));
    assert_eq!(
        emitted
            .matches("throw new RangeError(\"Linguini: invalid date value\");")
            .count(),
        1
    );
}
