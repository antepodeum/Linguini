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
        minijinja::context! { name => "formatTest", plan => plan.clone(), exported => false },
    );
    let helpers = number_helpers(&[plan]);
    assert!(!generated.contains("formatGeneratedNumber"));
    assert!(!generated.contains("undefined"));
    let script = format!("{generated}{helpers}console.log(JSON.stringify([formatTest('12.345'), formatTest('-12.345'), formatTest('0')]));");
    assert_eq!(
        execute(&script),
        "[\"pre1.235E+03%\",\"-pre1.235E+03%\",\"pre0.0E+00x%\"]"
    );

    let runtime = formatter_data_declaration("en", &[request(IrFormatterKind::Number, &[])], false);
    for unused in [
        "function scaleBy",
        "SignificantDigits(",
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
                name => "formatTest", plan => plan.clone(), exported => false,
            },
        );
        let helpers = number_helpers(&[plan]);
        for (name, selected) in [
            ("scaleBy100", "scale"),
            ("scientificGeneratedDecimal", "scientific"),
            ("roundTo3SignificantDigits", "significant"),
            ("padTo4SignificantDigits", "significant_padding"),
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
            name => "formatCash", plan => plan.clone(), exported => false,
        },
    );
    let helpers = number_helpers(&[plan]);
    assert!(formatter.contains("const fraction = rounded.fraction;"));
    assert!(!helpers.contains("trimOptionalFractionDigits"));
    assert_eq!(execute(&format!("{formatter}{helpers}console.log(JSON.stringify(['1.23','1.22','-1.23'].map(formatCash)));")), r#"["CHF1.25","CHF1.20","-CHF1.25"]"#);
}

#[test]
fn known_fraction_presence_removes_branches_without_changing_optional_or_significant_output() {
    for (code, expected) in [("USD", "$1.23"), ("JPY", "¥1"), ("KWD", "KWD1.235")] {
        let formatter = request(IrFormatterKind::Currency, &[("code", code)]);
        let binding = formatter_binding(&formatter);
        let emitted = formatter_data_declaration("en", &[formatter], false);
        assert!(!emitted.contains("fraction ? integer"), "{code}");
        if code == "JPY" {
            assert!(!emitted.contains("const fraction = rounded.fraction;"));
            assert!(!emitted.contains("const formatted ="));
        }
        assert_eq!(
            execute(&format!("{emitted}console.log({binding}('1.2345'));")),
            expected,
            "{code}"
        );
    }

    let mut numbers = compiled_number_formatting("en").unwrap();
    let mut pattern = numbers.decimal_pattern;
    for (min, max, significant, expected) in [
        (0, 0, None, r#"["1","-0","-0","1,235"]"#),
        (1, 3, None, r#"["1.0","-0.0","-0.0","1,234.5"]"#),
        (0, 3, None, r#"["1","-0","-0","1,234.5"]"#),
        (1, 3, Some(3), r#"["1.0","-0","-0.0001","1,230"]"#),
    ] {
        pattern.positive.min_fraction_digits = min;
        pattern.positive.max_fraction_digits = max;
        pattern.positive.max_significant_digits = significant;
        let plan = number_plan(&pattern, None, &numbers, None);
        let source = crate::templates::render(
            "formatters/number",
            minijinja::context! {name => "formatValue", plan => plan.clone(), exported => false},
        );
        assert_eq!(
            source.contains("fraction ? integer"),
            min == 0 && max != 0 || significant.is_some()
        );
        assert_eq!(execute(&format!("{source}{}console.log(JSON.stringify(['1', '-0', '-0.0001', '1234.5'].map(formatValue)));", number_helpers(&[plan]))), expected);
    }

    pattern.positive.min_fraction_digits = 0;
    pattern.positive.max_fraction_digits = 0;
    pattern.positive.max_significant_digits = None;
    numbers.digits = "٠١٢٣٤٥٦٧٨٩";
    let plan = number_plan(&pattern, None, &numbers, None);
    let source = crate::templates::render(
        "formatters/number",
        minijinja::context! {name => "formatInteger", plan => plan.clone(), exported => false},
    );
    assert!(source.contains("const localized = integer.replace"));
    assert!(!source.contains("const fraction ="));
    assert_eq!(
        execute(&format!(
            "{source}{}console.log(formatInteger('-1234.5'));",
            number_helpers(&[plan])
        )),
        "-١,٢٣٥"
    );

    pattern.positive.prefix = "prefix";
    pattern.positive.suffix = "tail";
    pattern.negative = Some(pattern.positive);
    let plan = number_plan(&pattern, None, &numbers, None);
    let source = crate::templates::render(
        "formatters/number",
        minijinja::context! {name => "formatInteger", plan => plan.clone(), exported => false},
    );
    assert!(source.contains("const prefix = \"prefix\";"));
    assert!(!source.contains("decimal.negative ?"));
    assert_eq!(
        execute(&format!(
            "{source}{}console.log(formatInteger('-1234.5'));",
            number_helpers(&[plan])
        )),
        "prefix١,٢٣٥tail"
    );
}

#[test]
fn compiled_rounding_matches_exact_rational_oracle_without_static_runtime_parsing() {
    let numbers = compiled_number_formatting("en").unwrap();
    let mut plans = Vec::new();
    let mut cases = String::new();
    for precision in 0..=4 {
        for increment in [
            "1",
            "2",
            "3",
            "5",
            "10",
            "25",
            "100",
            "125",
            "18446744073709551617",
        ] {
            let mut pattern = numbers.decimal_pattern;
            pattern.positive.max_fraction_digits = precision;
            let plan = number_plan(&pattern, None, &numbers, Some(increment));
            let name = plan.get_attr("rounding_helper").unwrap();
            cases.push_str(&format!("[{name}, {precision}, {increment:?}],"));
            plans.push(plan);
        }
    }
    // Identical formatter requests must share the emitted rounding implementation.
    plans.push(plans[0].clone());
    let helpers = number_helpers(&plans);
    assert_eq!(
        helpers.matches("function roundTo0FractionDigits(").count(),
        1
    );
    for retired in [
        "roundingIncrement",
        "quantumSource",
        "roundGeneratedDecimal",
        "10n **",
        "trimOptionalFractionDigits",
    ] {
        assert!(
            !helpers.contains(retired),
            "static runtime operation: {retired}"
        );
    }
    let output = execute(&format!(
        r#"{helpers}
function expected(decimal, precision, increment) {{
  const denominator = 10n ** BigInt(decimal.fraction.length);
  const numerator = BigInt(decimal.integer + decimal.fraction) * 10n ** BigInt(precision);
  const quantum = BigInt(increment);
  const units = (2n * numerator + quantum * denominator) / (2n * quantum * denominator);
  const digits = (units * quantum).toString().padStart(precision + 1, "0");
  return precision === 0 ? {{ integer: digits, fraction: "" }} : {{ integer: digits.slice(0, -precision), fraction: digits.slice(-precision) }};
}}
const samples = ["0", "0.00001", "0.00499", "0.005", "0.00501", "0.00999", "1.225", "9.99999", "999.99999", "123456789012345678901234567890.55555", "1." + "9".repeat(8000)];
for (let value = 0; value <= 1000; value++) samples.push((value / 100).toFixed(2));
for (const [round, precision, increment] of [{cases}]) {{
  for (const sample of samples) {{
    const [integer, fraction = ""] = sample.split(".");
    const decimal = {{ negative: false, integer, fraction }};
    const actual = round(decimal);
    const wanted = expected(decimal, precision, increment);
    if (JSON.stringify(actual) !== JSON.stringify(wanted)) throw new Error(JSON.stringify([round.name, sample, actual, wanted]));
  }}
}}
const originalBigInt = globalThis.BigInt;
globalThis.BigInt = () => {{ throw new Error("exact/non-incrementing unit rounding must skip bigint construction"); }};
try {{
  const actual = roundTo3FractionDigits({{ negative: false, integer: "12345678901234567890", fraction: "1234" }});
  if (actual.integer !== "12345678901234567890" || actual.fraction !== "123") throw new Error("unit rounding changed exact digits");
}} finally {{ globalThis.BigInt = originalBigInt; }}
console.log("ok");
"#
    ));
    assert_eq!(output, "ok");
}

#[test]
fn compiled_grouping_covers_width_boundaries_padding_and_long_exact_values() {
    let mut numbers = compiled_number_formatting("en").unwrap();
    let mut plans = Vec::new();
    let mut generated = String::new();
    let mut cases = String::new();
    for primary in 1..=4 {
        for secondary in 1..=4 {
            for minimum in [1, 6] {
                let mut pattern = numbers.decimal_pattern;
                pattern.positive.primary_group_size = Some(primary);
                pattern.positive.secondary_group_size = Some(secondary);
                pattern.positive.min_integer_digits = minimum;
                pattern.positive.min_fraction_digits = 0;
                pattern.positive.max_fraction_digits = 0;
                numbers.group_symbol = if minimum == 1 { "," } else { "🍎" };
                let plan = number_plan(&pattern, None, &numbers, None);
                let name = format!("group{primary}_{secondary}_{minimum}");
                generated.push_str(&crate::templates::render(
                    "formatters/number",
                    minijinja::context! { name => &name, plan => plan.clone(), exported => false },
                ));
                cases.push_str(&format!(
                    "[{name}, {primary}, {secondary}, {minimum}, {:?}],",
                    numbers.group_symbol
                ));
                plans.push(plan);
            }
        }
    }
    for retired in ["groups.push", "groups.reverse", ".join(", "Math.max(0, end"] {
        assert!(!generated.contains(retired), "grouping retained {retired}");
    }
    assert_eq!(
        execute(&format!(
            r#"{generated}{helpers}
function expected(value, primary, secondary, minimum, separator) {{
  const negative = value.startsWith("-");
  let digits = (negative ? value.slice(1) : value).padStart(minimum, "0");
  const groups = [];
  let width = primary;
  while (digits.length > width) {{
    groups.unshift(digits.slice(-width));
    digits = digits.slice(0, -width);
    width = secondary;
  }}
  groups.unshift(digits);
  return (negative ? "-" : "") + groups.join(separator);
}}
const values = ["0", "-0", "12345678", "9".repeat(8192)];
for (let length = 1; length <= 36; length++) values.push("123456789".repeat(4).slice(0, length));
for (const [format, primary, secondary, minimum, separator] of [{cases}]) {{
  for (const digits of values) {{
    for (const value of [digits, digits === "0" || digits === "-0" ? digits : "-" + digits]) {{
      const actual = format(value);
      const wanted = expected(value, primary, secondary, minimum, separator);
      if (actual !== wanted) throw new Error(JSON.stringify([format.name, value, actual, wanted]));
    }}
  }}
}}
if (group3_3_1("12345678") !== "12,345,678" || group3_2_1("12345678") !== "1,23,45,678") throw new Error("standard and Indian grouping drifted");
console.log("ok");
"#,
            helpers = number_helpers(&plans),
        )),
        "ok"
    );
}

#[test]
fn compiled_date_fields_select_padding_helpers() {
    let medium = request(IrFormatterKind::Date, &[]);
    let short = request(IrFormatterKind::Date, &[("style", "short")]);
    let unpadded = formatter_data_declaration("ru", std::slice::from_ref(&medium), false);
    assert!(!unpadded.contains("padNumber("));
    assert_eq!(
        execute(&format!("{unpadded}console.log(formatDate('2024-01-02'));")),
        "2 янв. 2024 г."
    );
    let padded = formatter_data_declaration("ru", &[medium, short], false);
    assert_eq!(padded.matches("function padNumber(").count(), 1);
    assert_eq!(
        execute(&format!(
            "{padded}console.log(formatDate_style_short('2024-01-02'));"
        )),
        "02.01.2024"
    );
}

#[test]
fn significant_rounding_checks_input_derived_expansion_before_padding() {
    let numbers = compiled_number_formatting("en").unwrap();
    let mut pattern = numbers.decimal_pattern;
    pattern.positive.max_significant_digits = Some(3);
    let helpers = number_helpers(&[number_plan(&pattern, None, &numbers, None)]);
    assert_eq!(
        execute(&format!(
            r#"{helpers}
const admitted = parseGeneratedDecimal("0." + "0".repeat(8189) + "1");
const rounded = roundTo3SignificantDigits(admitted);
if (rounded.fraction.length !== 8192) throw new Error("valid significant expansion was rejected");
const exceeds = parseGeneratedDecimal("0." + "0".repeat(8190) + "1");
if (!exceeds) throw new Error("the input itself is valid");
let rejected = false;
try {{ roundTo3SignificantDigits(exceeds); }} catch(error) {{ rejected = error instanceof RangeError; }}
if (!rejected) throw new Error("input-derived significant expansion exceeded the output limit");
console.log("ok");
"#
        )),
        "ok"
    );
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
            "[\"1,234,567.89\",\"\u{200f}1,234,567.89\u{a0}ZZZ\",\"2\u{200f}/1\u{200f}/2024\u{60c} 1:04 م\"]",
        ),
        (
            "hi",
            "[\"12,34,567.89\",\"ZZZ12,34,567.89\",\"2/1/24, 1:04 pm\"]",
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
    let requests = ["full", "long", "medium", "short"]
        .map(|style| request(IrFormatterKind::Date, &[("style", style)]));
    let emitted = formatter_data_declaration("en", &requests, false);

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
fn generated_date_symbols_preserve_cldr_context_width_and_locale() {
    for (locale, pattern, value, expected) in [
        ("ru", "MMMM LLLL", "2024-01-02T01:00:00Z", "января январь"),
        ("en", "MMMM LLLL", "2024-01-02T01:00:00Z", "January January"),
        ("en", "MMMMM EEEEE EEEEEE", "2024-01-02T01:00:00Z", "J T Tu"),
        (
            "fi",
            "EEEE cccc eeee",
            "2026-05-19T01:00:00Z",
            "tiistaina tiistai tiistaina",
        ),
        ("fi", "ccccc cccccc", "2026-05-19T01:00:00Z", "T ti"),
        (
            "smn",
            "EEEE cccc eeee",
            "2026-05-19T01:00:00Z",
            "majebaargâ majebargâ majebaargâ",
        ),
        ("smn", "ccccc cccccc", "2026-05-19T01:00:00Z", "T ma"),
        ("en", "aaa aaaa aaaaa", "2024-01-02T13:00:00Z", "PM PM p"),
        ("ar", "a", "2024-01-02T13:00:00Z", "م"),
        ("zh", "a", "2024-01-02T01:00:00Z", "上午"),
        ("en", "B", "2024-01-02T09:30:00Z", "in the morning"),
        ("en", "b", "2024-01-02T12:00:00Z", "noon"),
        ("en", "b", "2024-01-02T00:00:00Z", "midnight"),
        (
            "en",
            "y yy yyy yyyy yyyyy G",
            "0001-01-02T01:00:00Z",
            "1 01 001 0001 00001 AD",
        ),
        (
            "en",
            "y yy yyy yyyy yyyyy G",
            "0000-01-02T01:00:00Z",
            "1 01 001 0001 00001 BC",
        ),
        (
            "ru",
            "G GGGG GGGGG",
            "0000-01-02T01:00:00Z",
            "до н. э. до Рождества Христова до н.э.",
        ),
    ] {
        let dates = linguini_cldr::compiled_date_formatting(locale).unwrap();
        let compiled = date_parts(pattern, &dates, "formatTest", locale);
        let formatter = crate::templates::render(
            "formatters/date",
            minijinja::context! { name => "formatTest", exported => false, parts => compiled.parts, tables => compiled.tables, periods => compiled.periods, clock => compiled.clock, digits => dates.digits },
        );
        if locale == "en" && pattern == "MMMM LLLL" {
            assert_eq!(formatter.matches("const formatTest_").count(), 1);
        }
        let emitted = format!(
            "{formatter}{}",
            date_formatter_helpers(compiled.helpers.padding)
        );
        assert!(!emitted.contains("dayPeriods"));
        assert!(!emitted.contains("stand-alone"));
        assert!(!emitted.contains("Intl.DateTimeFormat"));
        assert_eq!(
            execute(&format!("{emitted}console.log(formatTest({value:?}));")),
            expected,
            "{locale}: {pattern}"
        );
    }
}

#[test]
fn gregorian_years_preserve_era_boundaries_and_minimum_widths() {
    let dates = linguini_cldr::compiled_date_formatting("en").unwrap();
    let pattern = "y yy yyy yyyy yyyyy yyyyyy yyyyyyy yyyyyyyy G";
    let compiled = date_parts(pattern, &dates, "formatYear", dates.locale);
    assert!(compiled.helpers.padding);
    let formatter = crate::templates::render(
        "formatters/date",
        minijinja::context! { name => "formatYear", exported => false, parts => compiled.parts, tables => compiled.tables, periods => compiled.periods, clock => compiled.clock, digits => dates.digits },
    );
    assert_eq!(formatter.matches("date.getUTCFullYear()").count(), 1);
    assert!(!formatter.contains("Intl."));
    let mut script = format!("{formatter}{}", date_formatter_helpers(true));
    for extended in [
        -271820, -10000, -1000, -100, -99, -2, -1, 0, 1, 2, 9, 10, 99, 100, 101, 9999, 10000,
        275000,
    ] {
        let year = if extended <= 0 {
            1 - extended
        } else {
            extended
        };
        let mut expected = (1..=8)
            .map(|width| format!("{:0width$}", if width == 2 { year % 100 } else { year }))
            .collect::<Vec<_>>()
            .join(" ");
        expected.push_str(if extended <= 0 { " BC" } else { " AD" });
        script.push_str(&format!("{{ const date = new Date(0); date.setUTCFullYear({extended}, 0, 2); if (formatYear(date) !== {expected:?}) throw new Error('year {extended}'); }}\n"));
    }
    script.push_str("console.log('ok');");
    assert_eq!(execute(&script), "ok");
    let mut dates = dates;
    dates.date_formats.medium = "yyyy";
    let (padded, requirements) = generated_date_request(
        dates.locale,
        &dates,
        &request(IrFormatterKind::Date, &[]),
        "formatPaddedYear",
        false,
    );
    assert!(requirements.padding);
    assert!(!padded.contains("extendedYear"));
    assert_eq!(
        execute(&format!(
            "{padded}{}console.log(formatPaddedYear('0001-01-02'));",
            date_formatter_helpers(requirements.padding)
        )),
        "0001"
    );
}

#[test]
fn compiled_day_periods_match_pinned_rules_across_every_minute_and_precision() {
    let mut script = String::from("import assert from 'node:assert/strict';\n");
    script.push_str(&date_formatter_helpers(true));
    script.push_str(&digit_formatter_helper());
    let mut count = 0;
    for locale in [
        "en", "en-GB", "ru", "zh", "ar", "fr", "de", "es", "hi", "ja", "ko", "pt", "fi", "bn",
    ] {
        let dates = linguini_cldr::compiled_date_formatting(locale).unwrap();
        let calendar = linguini_cldr::compiled_calendar_formatting(locale, "gregory").unwrap();
        let rules = linguini_cldr::compiled_day_period_rules(locale).unwrap_or_default();
        for field in ['b', 'B'] {
            for width in [1, 4, 5] {
                let name = format!("period{count}");
                count += 1;
                let pattern = format!("{}'|'m'|'s'|'SSS", field.to_string().repeat(width));
                let compiled = date_parts(&pattern, &dates, &name, dates.locale);
                let formatter = crate::templates::render(
                    "formatters/date",
                    minijinja::context! { name, exported => false, parts => compiled.parts, tables => compiled.tables, periods => compiled.periods, clock => compiled.clock, digits => dates.digits },
                );
                for forbidden in [
                    "dayPeriods",
                    "dayPeriodRule",
                    "Intl.",
                    "resolveDayPeriod",
                    "totalMinutes",
                ] {
                    assert!(!formatter.contains(forbidden), "{locale}: {forbidden}");
                }
                for getter in [
                    "getUTCHours",
                    "getUTCMinutes",
                    "getUTCSeconds",
                    "getUTCMilliseconds",
                ] {
                    assert!(formatter.matches(getter).count() <= 1, "{locale}: {getter}");
                }
                script.push_str(&formatter);
                let symbol_width = match width {
                    4 => "wide",
                    5 => "narrow",
                    _ => "abbreviated",
                };
                let symbol =
                    |period| calendar.symbol(&format!("dayPeriods.format.{symbol_width}.{period}"));
                let mut general = Vec::new();
                let mut exact = Vec::new();
                for minute in 0..1440 {
                    let fallback = symbol(if minute < 720 { "am" } else { "pm" }).unwrap();
                    // Independent reference evaluation of the raw pinned rules, not the compiled plan.
                    let range = rules.iter().find(|rule| match (rule.from, rule.before) {
                        (Some(from), Some(before)) if from < before => {
                            from <= minute && minute < before
                        }
                        (Some(from), Some(before)) => minute >= from || minute < before,
                        _ => false,
                    });
                    let regular = if field == 'B' {
                        range.and_then(|rule| symbol(rule.name)).unwrap_or(fallback)
                    } else {
                        fallback
                    };
                    general.push(regular);
                    exact.push(
                        rules
                            .iter()
                            .find(|rule| {
                                rule.at == Some(minute) && matches!(rule.name, "noon" | "midnight")
                            })
                            .and_then(|rule| symbol(rule.name))
                            .unwrap_or(regular),
                    );
                }
                script.push_str(&format!(r#"
{{
  const general = {general:?}, exact = {exact:?};
  for (let minute = 0; minute < 1440; minute++) {{
    for (const offset of [0, 1, 1000, 59000]) {{
      const value = new Date(Date.UTC(2024, 0, 2) + minute * 60000 + offset);
      assert.equal({name}(value).split('|')[0], (offset === 0 ? exact : general)[minute], `{locale}/{field}/{width}/${{minute}}/${{offset}}`);
    }}
  }}
}}
"#));
            }
        }
    }
    script.push_str("console.log('ok');\n");
    assert_eq!(execute(&script), "ok");
}

#[test]
fn exact_periods_follow_displayed_fields_and_omit_unused_clock_reads() {
    for (pattern, value, expected) in [
        ("b", "2024-01-02T12:30:45.123Z", "noon"),
        ("b'|'m", "2024-01-02T12:30:45.123Z", "PM|30"),
        ("b'|'m", "2024-01-02T12:00:45.123Z", "noon|0"),
        ("b'|'m'|'s", "2024-01-02T12:00:01.123Z", "PM|0|1"),
        ("b'|'m'|'s", "2024-01-02T12:00:00.123Z", "noon|0|0"),
        ("b'|'m'|'s'|'SSS", "2024-01-02T12:00:00.001Z", "PM|0|0|001"),
        ("'m s S' B", "2024-01-02T12:30:45.123Z", "m s S noon"),
        ("B BBB'|'m", "2024-01-02T12:00:00Z", "noon noon|0"),
        (
            "b B b'|'m'|'s",
            "2024-01-02T12:00:01Z",
            "PM in the afternoon PM|0|1",
        ),
    ] {
        let dates = linguini_cldr::compiled_date_formatting("en").unwrap();
        let compiled = date_parts(pattern, &dates, "formatPeriod", dates.locale);
        let formatter = crate::templates::render(
            "formatters/date",
            minijinja::context! { name => "formatPeriod", exported => false, parts => compiled.parts, tables => compiled.tables, periods => compiled.periods, clock => compiled.clock, digits => dates.digits },
        );
        if pattern == "B BBB'|'m" {
            assert_eq!(formatter.matches("let dayPeriod").count(), 1);
        }
        if pattern == "b B b'|'m'|'s" {
            assert_eq!(formatter.matches("let dayPeriod").count(), 2);
            assert_eq!(formatter.matches("const exactTime").count(), 1);
        }
        if !pattern.contains('|') {
            for getter in ["getUTCMinutes", "getUTCSeconds", "getUTCMilliseconds"] {
                assert!(!formatter.contains(getter));
            }
        }
        assert_eq!(
            execute(&format!(
                "{formatter}{}console.log(formatPeriod({value:?}));",
                date_formatter_helpers(true)
            )),
            expected
        );
    }
}

#[test]
fn compiled_day_period_fixture_preserves_readable_checked_javascript() {
    let mut output = String::new();
    for (name, locale, pattern) in [
        ("formatFlexible", "en", "h:mm:ss.SSS B"),
        ("formatFixed", "ru", "HH:mm bbbb"),
        ("formatNarrow", "ar", "BBBBB"),
        ("formatYear", "en", "y yy yyyy G"),
    ] {
        let dates = linguini_cldr::compiled_date_formatting(locale).unwrap();
        let compiled = date_parts(pattern, &dates, name, dates.locale);
        output.push_str(&crate::templates::render(
            "formatters/date",
            minijinja::context! { name, exported => true, parts => compiled.parts, tables => compiled.tables, periods => compiled.periods, clock => compiled.clock, digits => dates.digits },
        ));
    }
    output.push_str(&date_formatter_helpers(true));
    let snapshot = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/golden/snapshots/js-date/date.js");
    if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&snapshot, &output).unwrap();
    }
    assert_eq!(output, std::fs::read_to_string(snapshot).unwrap());
}

#[test]
fn date_validation_allocates_only_returned_dates_and_preserves_calendar_and_zone_semantics() {
    let helpers = date_formatter_helpers(false);
    let script = r#"
import assert from 'node:assert/strict';
const NativeDate = Date;
let constructions = 0;
globalThis.Date = new Proxy(NativeDate, {
  construct(target, args, receiver) {
    constructions++;
    return Reflect.construct(target, args, receiver);
  },
});
const invalid = {name: 'RangeError', message: 'Linguini: invalid date value'};
function check(input, expected, count = 1) {
  constructions = 0;
  assert.equal(coerceDate(input).toISOString(), expected, String(input));
  assert.equal(constructions, count, String(input));
}
for (const zone of ['UTC', 'America/Los_Angeles', 'Asia/Kolkata']) {
  process.env.TZ = zone;
  for (const [input, expected] of [
    ['0000-02-29', '0000-02-29T00:00:00.000Z'],
    ['0099-01-02T13:04:05', '0099-01-02T13:04:05.000Z'],
    ['1900-02-28', '1900-02-28T00:00:00.000Z'],
    ['2000-02-29T24:00:00', '2000-03-01T00:00:00.000Z'],
    ['2024-02-29T13:04:05.6789Z', '2024-02-29T13:04:05.678Z'],
    ['2024-01-01T00:30+01:00', '2023-12-31T23:30:00.000Z'],
    ['2024-12-31T23:30-01:00', '2025-01-01T00:30:00.000Z'],
    ['9999-12-31', '9999-12-31T00:00:00.000Z'],
  ]) check(input, expected);
  for (const date of ['0001-02-29', '1900-02-29', '2100-02-29', '2024-02-30', '2024-04-31', '2024-00-01', '2024-13-01', '2024-01-00', '2024-01-32']) {
    for (const suffix of ['', 'T12:00:00Z', 'T12:00+01:00', 'T12:00']) {
      constructions = 0;
      assert.throws(() => coerceDate(date + suffix), invalid, date + suffix);
      assert.equal(constructions, 0, 'invalid calendar fields need no Date');
    }
  }
}
for (let year = 0; year <= 9999; year++) {
  const text = String(year).padStart(4, '0') + '-02-29';
  const oracle = new NativeDate(0);
  oracle.setUTCFullYear(year, 1, 29);
  constructions = 0;
  if (oracle.getUTCMonth() === 1) {
    assert.equal(coerceDate(text).getTime(), oracle.getTime(), text);
    assert.equal(constructions, 1);
  } else {
    assert.throws(() => coerceDate(text), invalid, text);
    assert.equal(constructions, 0);
  }
}
for (const input of ['2024-01-01T25:00Z', '2024-01-01T12:60Z', '2024-01-01T12:00:60Z', '2024-01-01T24:01Z', '2024-01-01T12:00+24:00']) {
  constructions = 0;
  assert.throws(() => coerceDate(input), invalid, input);
  assert.equal(constructions, 1);
}
for (const input of ['', 'invalid', '2024-1-01', '2024-01-01junk', ' 2024-01-01', '2024-01-01 ']) {
  constructions = 0;
  assert.throws(() => coerceDate(input), invalid, input);
  assert.equal(constructions, 0);
}
check(0, '1970-01-01T00:00:00.000Z');
check(-1, '1969-12-31T23:59:59.999Z');
for (const input of [NaN, Infinity, -Infinity]) {
  constructions = 0;
  assert.throws(() => coerceDate(input), invalid);
  assert.equal(constructions, 1);
}
const supplied = new NativeDate('2024-01-02T03:04:05Z');
const observations = [];
supplied.getTime = () => { observations.push('getTime'); return NativeDate.prototype.getTime.call(supplied); };
for (const getter of ['getUTCFullYear', 'getUTCMonth', 'getUTCDate']) {
  supplied[getter] = () => { throw Error('supplied Date must not be calendar-validated'); };
}
constructions = 0;
assert.equal(coerceDate(supplied), supplied);
assert.deepEqual(observations, ['getTime']);
assert.equal(constructions, 0);
const failure = new Error('custom date failure');
supplied.getTime = () => { throw failure; };
assert.throws(() => coerceDate(supplied), error => error === failure);
console.log('ok');
"#;
    assert_eq!(execute(&format!("{helpers}{script}")), "ok");
}

#[test]
fn generated_date_runtime_rejects_invalid_values() {
    let emitted = formatter_data_declaration("en", &[request(IrFormatterKind::Date, &[])], false);

    assert!(emitted.contains("function coerceDate(value)"));
    assert!(emitted.contains("date = createUTCDate(year, month, day);"));
    assert!(emitted.contains("date = new Date(hasTimeZone ? value : `${value}Z`);"));
    assert!(emitted.contains("function createUTCDate(year, month, day)"));
    assert!(emitted.contains("date.setUTCFullYear(year, month - 1, day);"));
    assert!(emitted.contains("function validateDateFields(year, month, day)"));
    assert!(!emitted.contains("date.getUTCFullYear() !== year"));
    assert!(emitted.contains("if (!Number.isFinite(date.getTime()))"));
    assert_eq!(
        emitted
            .matches("throw new RangeError(\"Linguini: invalid date value\");")
            .count(),
        1
    );
}

#[test]
fn public_full_date_requests_render_standalone_weekdays_instead_of_a_zone_placeholder() {
    for (locale, expected) in [
        ("fi", "tiistai 19. toukokuuta 2026"),
        ("smn", "majebargâ, vyesimáánu 19. 2026"),
    ] {
        let formatter = request(IrFormatterKind::Date, &[("style", "full")]);
        let binding = formatter_binding(&formatter);
        let emitted = formatter_data_declaration(locale, &[formatter], false);
        assert!(
            !emitted.contains("\"UTC\""),
            "weekday compiled as UTC: {emitted}"
        );
        assert!(!emitted.contains("Intl."));
        assert_eq!(
            execute(&format!(
                "{emitted}console.log({binding}('2026-05-19T01:00:00Z'));"
            )),
            expected
        );
    }
}

#[test]
fn utc_zone_names_are_selected_from_pinned_cldr_and_folded_into_generated_literals() {
    for (locale, long) in [
        ("en", "Coordinated Universal Time"),
        ("fr", "temps universel coordonné"),
        ("ru", "Всемирное координированное время"),
        ("ar", "التوقيت العالمي المنسق"),
        ("hi", "समन्वित वैश्विक समय"),
        ("ja", "協定世界時"),
        ("zh", "协调世界时"),
        ("aa", "GMT"),
    ] {
        let dates = linguini_cldr::compiled_date_formatting(locale).unwrap();
        let compiled = date_parts("z zz zzz zzzz", &dates, "formatZone", locale);
        assert!(compiled.tables.is_empty());
        assert!(!compiled.helpers.padding);
        assert!(!compiled.helpers.digits);
        let formatter = crate::templates::render(
            "formatters/date",
            minijinja::context! {
                name => "formatZone", exported => false, parts => compiled.parts,
                tables => compiled.tables, periods => compiled.periods, clock => compiled.clock,
                digits => dates.digits,
            },
        );
        assert!(
            !formatter.contains("getUTC"),
            "a constant UTC zone must not read date fields"
        );
        for unused in ["zzzz", "gmtFormat", "hourFormat", "timeZoneNames", "Intl."] {
            assert!(!formatter.contains(unused), "{locale}: {unused}");
        }
        let source = format!("{formatter}{}", date_formatter_helpers(false));
        assert_eq!(execute(&format!("{source}Intl.DateTimeFormat = () => {{ throw Error('host formatting used'); }}; console.log(formatZone('2026-05-19T01:00:00Z'));")), format!("UTC UTC UTC {long}"));
        let requests = [
            request(IrFormatterKind::Date, &[("time_style", "full")]),
            request(IrFormatterKind::Date, &[("time_style", "long")]),
        ];
        let full = formatter_binding(&requests[0]);
        let short = formatter_binding(&requests[1]);
        let source = formatter_data_declaration(locale, &requests, false);
        let position = if locale == "zh" {
            "startsWith"
        } else {
            "endsWith"
        };
        assert_eq!(execute(&format!("{source}console.log(JSON.stringify([{full}('2026-05-19T01:00:00Z').{position}({}), {short}('2026-05-19T01:00:00Z').{position}('UTC')]));", crate::ecmascript::string_literal(long))), "[true,true]", "{locale}");
    }
}
