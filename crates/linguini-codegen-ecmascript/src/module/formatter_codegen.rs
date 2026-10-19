//! Locale formatter lowering and demand-selected runtime assembly.

use linguini_cldr::{
    compiled_currency_formatting, compiled_date_formatting, compiled_number_formatting,
    NumberPaddingPosition, NumberPattern,
};

use super::formatter_helpers::{date_formatter_helpers, digit_formatter_helper};

pub(super) fn formatter_data_declaration(
    locale: &str,
    requests: &[super::formatters::FormatterRequest],
    export_helpers: bool,
) -> String {
    use minijinja::context;
    let needs_number = requests.iter().any(|request| {
        matches!(
            request.formatter.kind,
            linguini_ir::IrFormatterKind::Number | linguini_ir::IrFormatterKind::Currency
        )
    });
    let needs_date = requests
        .iter()
        .any(|request| request.formatter.kind == linguini_ir::IrFormatterKind::Date);
    let mut output = crate::templates::render("formatters/types", context! { needs_number });
    let mut plans = Vec::new();
    for request in requests {
        let name = super::formatters::formatter_binding(request);
        match request.formatter.kind {
            linguini_ir::IrFormatterKind::Number | linguini_ir::IrFormatterKind::Currency => {
                let numbers = compiled_number_formatting(locale).expect("validated number locale");
                let mut pattern = numbers.decimal_pattern;
                if request.formatter.kind == linguini_ir::IrFormatterKind::Currency {
                    let currency =
                        compiled_currency_formatting(locale).expect("validated currency locale");
                    pattern = if request.argument("accounting") == Some("true") {
                        currency
                            .accounting_pattern
                            .unwrap_or(currency.standard_pattern)
                    } else {
                        currency.standard_pattern
                    };
                    let code = request.argument("code").unwrap_or("USD");
                    let symbol =
                        Some(linguini_cldr::compiled_currency_symbol(locale, code).unwrap_or(code));
                    let fraction = linguini_cldr::compiled_currency_fraction(code)
                        .expect("validated currency code");
                    pattern.positive.min_fraction_digits = fraction.digits;
                    pattern.positive.max_fraction_digits = fraction.digits;
                    // Supplemental increments are expressed at the currency fraction scale.
                    let rounding = fraction.rounding.to_string();
                    let plan = number_plan(&pattern, symbol, &numbers, Some(&rounding));
                    output.push_str(&crate::templates::render(
                        "formatters/number",
                        context! { name, plan, exported => export_helpers },
                    ));
                } else {
                    let plan = number_plan(&pattern, None, &numbers, None);
                    output.push_str(&crate::templates::render(
                        "formatters/number",
                        context! { name, plan, exported => export_helpers },
                    ));
                }
                plans.push(pattern.positive);
            }
            linguini_ir::IrFormatterKind::Date => {
                let dates = compiled_date_formatting(locale).expect("validated date locale");
                output.push_str(&generated_date_request(
                    &dates,
                    request,
                    &name,
                    export_helpers,
                ));
            }
            linguini_ir::IrFormatterKind::Unknown(_) => unreachable!("validated formatter request"),
        }
    }
    if needs_number {
        output.push_str(&crate::templates::render("number_helpers", context! {
            scale => plans.iter().any(|plan| plan.scale != 1),
            scientific => plans.iter().any(|plan| plan.exponent_digits.is_some()),
            significant => plans.iter().any(|plan| plan.max_significant_digits.is_some()),
            significant_padding => plans.iter().any(|plan| plan.min_significant_digits.is_some()),
        }));
    }
    if needs_date {
        output.push_str(&date_formatter_helpers());
    }
    if needs_date
        && compiled_date_formatting(locale)
            .expect("validated date locale")
            .digits
            != "0123456789"
    {
        output.push_str(&digit_formatter_helper());
    }
    output
}

fn number_plan(
    pattern: &NumberPattern,
    symbol: Option<&str>,
    numbers: &linguini_cldr::NumberFormatData,
    rounding: Option<&str>,
) -> minijinja::Value {
    use minijinja::context;
    let positive = &pattern.positive;
    let affix = |text: &str| {
        symbol.map_or_else(|| text.to_owned(), |symbol| text.replace('\u{a4}', symbol))
    };
    let prefix = affix(positive.prefix);
    let suffix = affix(positive.suffix);
    let negative_prefix = pattern
        .negative
        .as_ref()
        .map_or_else(|| format!("-{prefix}"), |negative| affix(negative.prefix));
    let negative_suffix = pattern
        .negative
        .as_ref()
        .map_or_else(|| suffix.clone(), |negative| affix(negative.suffix));
    let padding = positive.padding.as_ref().map(|padding| context! {
        character => padding.character.to_string(), width => padding.width,
        position => match padding.position { NumberPaddingPosition::BeforePrefix => 0, NumberPaddingPosition::AfterPrefix => 1, NumberPaddingPosition::BeforeSuffix => 2, NumberPaddingPosition::AfterSuffix => 3 },
    });
    context! {
        prefix, suffix, negative_prefix, negative_suffix,
        min_integer => positive.min_integer_digits, min_fraction => positive.min_fraction_digits,
        max_fraction => positive.max_fraction_digits, primary_group => positive.primary_group_size,
        secondary_group => positive.secondary_group_size.or(positive.primary_group_size),
        decimal_symbol => numbers.decimal_symbol, group_symbol => numbers.group_symbol, digits => numbers.digits,
        scale => positive.scale, min_significant => positive.min_significant_digits, max_significant => positive.max_significant_digits,
        rounding => rounding.or(positive.rounding_increment).filter(|value| value.bytes().any(|byte| byte != b'0')).unwrap_or("1"), exponent_digits => positive.exponent_digits, exponent_sign => positive.exponent_sign_always, padding,
    }
}

fn generated_date_request(
    dates: &linguini_cldr::DateFormatData,
    request: &super::formatters::FormatterRequest,
    name: &str,
    exported: bool,
) -> String {
    let select = |widths: linguini_cldr::FormatWidths, style: &str| match style {
        "full" => widths.full,
        "long" => widths.long,
        "short" => widths.short,
        _ => widths.medium,
    };
    let style = request.argument("style");
    let time_style = request.argument("time_style");
    let date = (style.is_some() || time_style.is_none())
        .then(|| select(dates.date_formats, style.unwrap_or("medium")));
    let time = time_style.map(|style| select(dates.time_formats, style));
    let pattern = match (date, time) {
        (Some(date), Some(time)) => select(dates.date_time_formats, style.unwrap_or("medium"))
            .replace("{1}", date)
            .replace("{0}", time),
        (Some(part), None) | (None, Some(part)) => part.to_owned(),
        (None, None) => unreachable!("at least a date or time pattern"),
    };
    let (parts, tables) = date_parts(&pattern, dates, name);
    crate::templates::render(
        "formatters/date",
        minijinja::context! { name, exported, parts, tables, digits => dates.digits },
    )
}

fn date_parts(
    pattern: &str,
    dates: &linguini_cldr::DateFormatData,
    name: &str,
) -> (Vec<minijinja::Value>, Vec<minijinja::Value>) {
    use minijinja::context;
    let mut parts = Vec::new();
    let mut tables = std::collections::BTreeMap::new();
    let mut chars = pattern.chars().peekable();
    let mut literal = String::new();
    let mut quoted = false;
    while let Some(character) = chars.next() {
        if character == '\'' {
            if chars.peek() == Some(&'\'') {
                chars.next();
                literal.push('\'');
            } else {
                quoted = !quoted;
            }
            continue;
        }
        if quoted || !character.is_ascii_alphabetic() {
            literal.push(character);
            continue;
        }
        if !literal.is_empty() {
            parts.push(context! { kind => "literal", value => std::mem::take(&mut literal) });
        }
        let mut width = 1;
        while chars.peek() == Some(&character) {
            chars.next();
            width += 1;
        }
        let mut table = None;
        let symbols: Option<&[&str]> = match character {
            'M' | 'L' if width >= 4 => Some(&dates.months.wide),
            'M' | 'L' if width == 3 => Some(&dates.months.abbreviated),
            'E' if width >= 4 => Some(&dates.weekdays.wide),
            'E' => Some(&dates.weekdays.abbreviated),
            _ => None,
        };
        if let Some(values) = symbols {
            let binding = format!(
                "{name}_{}_{width}",
                if matches!(character, 'M' | 'L') {
                    "months"
                } else {
                    "weekdays"
                }
            );
            tables
                .entry(binding.clone())
                .or_insert_with(|| context! { name => &binding, values });
            table = Some(binding);
        }
        let numeric = matches!(
            character,
            'y' | 'd' | 'H' | 'h' | 'K' | 'k' | 'm' | 's' | 'S'
        ) || matches!(character, 'M' | 'L') && width <= 2;
        parts.push(context! { kind => "field", field => character.to_string(), width, table, numeric, am => "AM", pm => "PM", ad => "AD", bc => "BC" });
    }
    assert!(!quoted, "validated CLDR pattern has balanced quoting");
    if !literal.is_empty() {
        parts.push(context! { kind => "literal", value => literal });
    }
    (parts, tables.into_values().collect())
}

#[cfg(test)]
fn date_pattern_expression(pattern: &str, dates: &linguini_cldr::DateFormatData) -> String {
    let (parts, _) = date_parts(pattern, dates, "test");
    crate::templates::render(
        "formatters/date_expression",
        minijinja::context! { parts, digits => dates.digits },
    )
    .trim()
    .to_owned()
}

#[cfg(test)]
mod tests {
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

        let runtime =
            formatter_data_declaration("en", &[request(IrFormatterKind::Number, &[])], false);
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
        let emitted =
            formatter_data_declaration("en", &[request(IrFormatterKind::Date, &[])], false);

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
}
