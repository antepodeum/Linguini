//! Locale formatter lowering and demand-selected runtime assembly.

use linguini_cldr::{
    compiled_currency_formatting, compiled_date_formatting, compiled_number_formatting,
    NumberPaddingPosition, NumberPattern,
};

use super::formatter_helpers::{
    date_formatter_helpers, digit_formatter_helper, number_formatter_helpers,
};
use super::formatters::FormatterRequirements;
use super::names::string_literal;
use super::EcmaScriptTarget;

pub(super) fn formatter_data_declaration_for_target(
    locale: &str,
    requirements: FormatterRequirements,
    target: EcmaScriptTarget,
    export_helpers: bool,
) -> String {
    let mut output = match target {
        EcmaScriptTarget::TypeScript => {
            "type GeneratedNumeric = number | bigint | string;\n".to_owned()
        }
        EcmaScriptTarget::JavaScript => {
            "/** @typedef {number | bigint | string} GeneratedNumeric */\n".to_owned()
        }
    };
    if requirements.currency {
        output.push_str(match target {
            EcmaScriptTarget::TypeScript => {
                "type GeneratedCurrencyFormatterOptions = { code?: string; accounting?: \"true\" | \"false\" };\n"
            }
            EcmaScriptTarget::JavaScript => {
                "/** @typedef {{ code?: string, accounting?: \"true\" | \"false\" }} GeneratedCurrencyFormatterOptions */\n"
            }
        });
    }
    if requirements.date {
        output.push_str(match target {
            EcmaScriptTarget::TypeScript => {
                "type GeneratedDateFormatterStyle = \"full\" | \"long\" | \"medium\" | \"short\";\n\
                 type GeneratedDateFormatterOptions = { style?: GeneratedDateFormatterStyle; time_style?: GeneratedDateFormatterStyle };\n"
            }
            EcmaScriptTarget::JavaScript => {
                "/** @typedef {\"full\" | \"long\" | \"medium\" | \"short\"} GeneratedDateFormatterStyle */\n\
                 /** @typedef {{ style?: GeneratedDateFormatterStyle, time_style?: GeneratedDateFormatterStyle }} GeneratedDateFormatterOptions */\n"
            }
        });
    }
    output.push('\n');

    let numbers = requirements.needs_number_data().then(|| {
        compiled_number_formatting(locale)
            .expect("validated locale must have required CLDR number formatting data")
    });
    if requirements.number {
        output.push_str(&generated_number_function(
            numbers
                .as_ref()
                .expect("number formatter requires number data"),
            target,
            export_helpers,
        ));
    }
    if requirements.currency {
        let currency = compiled_currency_formatting(locale)
            .expect("validated locale must have required CLDR currency formatting data");
        output.push_str(&generated_currency_function(
            locale,
            numbers
                .as_ref()
                .expect("currency formatter requires number data"),
            &currency,
            target,
            export_helpers,
        ));
    }
    if requirements.date {
        let dates = compiled_date_formatting(locale)
            .expect("validated locale must have required CLDR date formatting data");
        output.push_str(&generated_date_function(&dates, target, export_helpers));
    }
    if requirements.needs_number_data() {
        output.push_str(&number_formatter_helpers(target));
    }
    if requirements.date {
        output.push_str(&date_formatter_helpers(target));
    }
    if requirements.any() {
        output.push_str(&digit_formatter_helper(target));
    }
    output
}

fn generated_number_function(
    numbers: &linguini_cldr::NumberFormatData,
    target: EcmaScriptTarget,
    exported: bool,
) -> String {
    let export = if exported { "export " } else { "" };
    let arguments = number_pattern_args(&numbers.decimal_pattern, None, numbers);
    let signature = match target {
        EcmaScriptTarget::TypeScript => format!(
            "{export}function formatNumber(value: GeneratedNumeric): string {{\n"
        ),
        EcmaScriptTarget::JavaScript => format!(
            "/**\n * @param {{GeneratedNumeric}} value\n * @returns {{string}}\n */\n{export}function formatNumber(value) {{\n"
        ),
    };
    format!("{signature}  return formatGeneratedNumber(value, {arguments});\n}}\n\n")
}

fn generated_currency_function(
    locale: &str,
    numbers: &linguini_cldr::NumberFormatData,
    currency: &linguini_cldr::CurrencyFormatData,
    target: EcmaScriptTarget,
    exported: bool,
) -> String {
    let export = if exported { "export " } else { "" };
    let standard = number_pattern_args(&currency.standard_pattern, Some("symbol"), numbers);
    let accounting = number_pattern_args(
        currency
            .accounting_pattern
            .as_ref()
            .unwrap_or(&currency.standard_pattern),
        Some("symbol"),
        numbers,
    );
    let signature = match target {
        EcmaScriptTarget::TypeScript => format!(
            "{export}function formatCurrency(\n  value: GeneratedNumeric,\n  fractionDigits: number,\n  roundingIncrement: number,\n  options: GeneratedCurrencyFormatterOptions = {{}},\n): string {{\n"
        ),
        EcmaScriptTarget::JavaScript => format!(
            "/**\n * @param {{GeneratedNumeric}} value\n * @param {{number}} fractionDigits\n * @param {{number}} roundingIncrement\n * @param {{GeneratedCurrencyFormatterOptions}} [options]\n * @returns {{string}}\n */\n{export}function formatCurrency(\n  value,\n  fractionDigits,\n  roundingIncrement,\n  options = {{}},\n) {{\n"
        ),
    };
    let symbol_signature = match target {
        EcmaScriptTarget::TypeScript => "function currencySymbol(currency: string): string {\n",
        EcmaScriptTarget::JavaScript => "/**\n * @param {string} currency\n * @returns {string}\n */\nfunction currencySymbol(currency) {\n",
    };
    format!(
        "\
{signature}  const symbol = currencySymbol(options.code ?? \"USD\");
  if (options.accounting === \"true\") {{
    return formatGeneratedNumber(value, {accounting}, fractionDigits, fractionDigits, roundingIncrement);
  }}
  return formatGeneratedNumber(value, {standard}, fractionDigits, fractionDigits, roundingIncrement);
}}

{symbol_signature}  return new Intl.NumberFormat({}, {{ style: \"currency\", currency }})
    .formatToParts(0)
    .find((part) => part.type === \"currency\")?.value ?? currency;
}}

",
        string_literal(locale)
    )
}

fn generated_date_function(
    dates: &linguini_cldr::DateFormatData,
    target: EcmaScriptTarget,
    exported: bool,
) -> String {
    let export = if exported { "export " } else { "" };
    let date_full = date_pattern_expression(dates.date_formats.full, dates);
    let date_long = date_pattern_expression(dates.date_formats.long, dates);
    let date_medium = date_pattern_expression(dates.date_formats.medium, dates);
    let date_short = date_pattern_expression(dates.date_formats.short, dates);
    let time_full = date_pattern_expression(dates.time_formats.full, dates);
    let time_long = date_pattern_expression(dates.time_formats.long, dates);
    let time_medium = date_pattern_expression(dates.time_formats.medium, dates);
    let time_short = date_pattern_expression(dates.time_formats.short, dates);
    let combined_full = date_time_pattern_expression(dates.date_time_formats.full);
    let combined_long = date_time_pattern_expression(dates.date_time_formats.long);
    let combined_medium = date_time_pattern_expression(dates.date_time_formats.medium);
    let combined_short = date_time_pattern_expression(dates.date_time_formats.short);
    let function_head = match target {
        EcmaScriptTarget::TypeScript => format!(
            "{export}function formatDate(\n  value: Date | number | string,\n  options: GeneratedDateFormatterOptions = {{}},\n): string {{"
        ),
        EcmaScriptTarget::JavaScript => format!(
            "/**\n * @param {{Date | number | string}} value\n * @param {{GeneratedDateFormatterOptions}} [options]\n * @returns {{string}}\n */\n{export}function formatDate(\n  value,\n  options = {{}},\n) {{"
        ),
    };
    let date_part_declaration = match target {
        EcmaScriptTarget::TypeScript => "  let datePart: string | undefined;",
        EcmaScriptTarget::JavaScript => "  /** @type {string | undefined} */\n  let datePart;",
    };
    let time_part_declaration = match target {
        EcmaScriptTarget::TypeScript => "  let timePart: string;",
        EcmaScriptTarget::JavaScript => "  /** @type {string} */\n  let timePart;",
    };
    let combined_declaration = match target {
        EcmaScriptTarget::TypeScript => "  let combined: string;",
        EcmaScriptTarget::JavaScript => "  /** @type {string} */\n  let combined;",
    };
    format!(
        "\
{function_head}
  const date = coerceDate(value);
{date_part_declaration}
  if (options.style !== undefined || options.time_style === undefined) {{
    switch (options.style ?? \"medium\") {{
      case \"full\": datePart = {date_full}; break;
      case \"long\": datePart = {date_long}; break;
      case \"short\": datePart = {date_short}; break;
      default: datePart = {date_medium};
    }}
  }}
  if (options.time_style === undefined) {{
    return localizeGeneratedDigits(datePart ?? \"\", {});
  }}
{time_part_declaration}
  switch (options.time_style) {{
    case \"full\": timePart = {time_full}; break;
    case \"long\": timePart = {time_long}; break;
    case \"short\": timePart = {time_short}; break;
    default: timePart = {time_medium};
  }}
  if (datePart === undefined) {{
    return localizeGeneratedDigits(timePart, {});
  }}
{combined_declaration}
  switch (options.style ?? \"medium\") {{
    case \"full\": combined = {combined_full}; break;
    case \"long\": combined = {combined_long}; break;
    case \"short\": combined = {combined_short}; break;
    default: combined = {combined_medium};
  }}
  return localizeGeneratedDigits(combined, {});
}}

",
        string_literal(dates.digits),
        string_literal(dates.digits),
        string_literal(dates.digits)
    )
}

fn number_pattern_args(
    pattern: &NumberPattern,
    currency_symbol: Option<&str>,
    numbers: &linguini_cldr::NumberFormatData,
) -> String {
    let negative = pattern.negative.as_ref();
    let padding = pattern.positive.padding.as_ref();
    format!(
        "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
        affix_expression(pattern.positive.prefix, currency_symbol),
        affix_expression(pattern.positive.suffix, currency_symbol),
        negative.map_or_else(
            || "undefined".to_owned(),
            |part| affix_expression(part.prefix, currency_symbol)
        ),
        negative.map_or_else(
            || "undefined".to_owned(),
            |part| affix_expression(part.suffix, currency_symbol)
        ),
        pattern.positive.min_integer_digits,
        pattern.positive.min_fraction_digits,
        pattern.positive.max_fraction_digits,
        option_u8_literal(pattern.positive.primary_group_size),
        option_u8_literal(pattern.positive.secondary_group_size),
        string_literal(numbers.decimal_symbol),
        string_literal(numbers.group_symbol),
        string_literal(numbers.digits),
        pattern.positive.scale,
        option_u8_literal(pattern.positive.min_significant_digits),
        option_u8_literal(pattern.positive.max_significant_digits),
        pattern
            .positive
            .rounding_increment
            .map_or_else(|| "undefined".to_owned(), string_literal),
        option_u8_literal(pattern.positive.exponent_digits),
        pattern.positive.exponent_sign_always,
        padding.map_or_else(
            || "undefined".to_owned(),
            |padding| string_literal(&padding.character.to_string())
        ),
        padding.map_or_else(
            || "undefined".to_owned(),
            |padding| padding.width.to_string()
        ),
        padding.map_or_else(
            || "undefined".to_owned(),
            |padding| match padding.position {
                NumberPaddingPosition::BeforePrefix => "0".to_owned(),
                NumberPaddingPosition::AfterPrefix => "1".to_owned(),
                NumberPaddingPosition::BeforeSuffix => "2".to_owned(),
                NumberPaddingPosition::AfterSuffix => "3".to_owned(),
            }
        )
    )
}

fn affix_expression(value: &str, currency_symbol: Option<&str>) -> String {
    let Some(symbol) = currency_symbol else {
        return string_literal(value);
    };
    value
        .split('\u{a4}')
        .map(string_literal)
        .collect::<Vec<_>>()
        .join(&format!(" + {symbol} + "))
}

fn option_u8_literal(value: Option<u8>) -> String {
    value.map_or_else(|| "undefined".to_owned(), |value| value.to_string())
}

fn date_pattern_expression(pattern: &str, dates: &linguini_cldr::DateFormatData) -> String {
    let mut parts = Vec::new();
    let mut chars = pattern.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\'' {
            let mut literal = String::new();
            for next in chars.by_ref() {
                if next == '\'' {
                    break;
                }
                literal.push(next);
            }
            if !literal.is_empty() {
                parts.push(string_literal(&literal));
            }
            continue;
        }
        if is_date_pattern_field(character) {
            let mut width = 1;
            while chars.peek() == Some(&character) {
                chars.next();
                width += 1;
            }
            parts.push(date_field_expression(character, width, dates));
            continue;
        }
        let mut literal = character.to_string();
        while let Some(next) = chars.peek().copied() {
            if next == '\'' || is_date_pattern_field(next) {
                break;
            }
            chars.next();
            literal.push(next);
        }
        parts.push(string_literal(&literal));
    }
    parts.join(" + ")
}

fn is_date_pattern_field(character: char) -> bool {
    matches!(
        character,
        'G' | 'y'
            | 'M'
            | 'L'
            | 'd'
            | 'E'
            | 'H'
            | 'h'
            | 'K'
            | 'k'
            | 'm'
            | 's'
            | 'S'
            | 'a'
            | 'b'
            | 'B'
            | 'z'
            | 'Z'
            | 'O'
            | 'v'
            | 'V'
            | 'X'
            | 'x'
    )
}

fn date_field_expression(
    field: char,
    width: usize,
    dates: &linguini_cldr::DateFormatData,
) -> String {
    match field {
        'y' if width == 2 => "padNumber(date.getUTCFullYear() % 100, 2)".to_owned(),
        'y' => "String(date.getUTCFullYear())".to_owned(),
        'M' | 'L' if width >= 4 => indexed_string_literal(&dates.months.wide, "date.getUTCMonth()"),
        'M' | 'L' if width == 3 => {
            indexed_string_literal(&dates.months.abbreviated, "date.getUTCMonth()")
        }
        'M' | 'L' if width == 2 => "padNumber(date.getUTCMonth() + 1, 2)".to_owned(),
        'M' | 'L' => "String(date.getUTCMonth() + 1)".to_owned(),
        'd' if width == 2 => "padNumber(date.getUTCDate(), 2)".to_owned(),
        'd' => "String(date.getUTCDate())".to_owned(),
        'E' if width >= 4 => indexed_string_literal(&dates.weekdays.wide, "date.getUTCDay()"),
        'E' => indexed_string_literal(&dates.weekdays.abbreviated, "date.getUTCDay()"),
        'H' if width == 2 => "padNumber(date.getUTCHours(), 2)".to_owned(),
        'H' => "String(date.getUTCHours())".to_owned(),
        'h' if width == 2 => "padNumber(date.getUTCHours() % 12 || 12, 2)".to_owned(),
        'h' => "String(date.getUTCHours() % 12 || 12)".to_owned(),
        'K' if width == 2 => "padNumber(date.getUTCHours() % 12, 2)".to_owned(),
        'K' => "String(date.getUTCHours() % 12)".to_owned(),
        'k' if width == 2 => "padNumber(date.getUTCHours() || 24, 2)".to_owned(),
        'k' => "String(date.getUTCHours() || 24)".to_owned(),
        'm' if width == 2 => "padNumber(date.getUTCMinutes(), 2)".to_owned(),
        'm' => "String(date.getUTCMinutes())".to_owned(),
        's' if width == 2 => "padNumber(date.getUTCSeconds(), 2)".to_owned(),
        's' => "String(date.getUTCSeconds())".to_owned(),
        'S' => format!(
            "padNumber(date.getUTCMilliseconds(), 3).padEnd({width}, \"0\").slice(0, {width})"
        ),
        'a' | 'b' | 'B' => "(date.getUTCHours() < 12 ? \"AM\" : \"PM\")".to_owned(),
        'z' | 'Z' | 'O' | 'v' | 'V' | 'X' | 'x' => "\"UTC\"".to_owned(),
        'G' => "date.getUTCFullYear() > 0 ? \"AD\" : \"BC\"".to_owned(),
        _ => "\"\"".to_owned(),
    }
}

fn date_time_pattern_expression(pattern: &str) -> String {
    let mut parts = Vec::new();
    let mut remaining = pattern;
    while !remaining.is_empty() {
        let date = remaining.find("{1}");
        let time = remaining.find("{0}");
        let next = match (date, time) {
            (Some(date), Some(time)) if date <= time => Some((date, "datePart", 3)),
            (Some(_), Some(time)) => Some((time, "timePart", 3)),
            (Some(date), None) => Some((date, "datePart", 3)),
            (None, Some(time)) => Some((time, "timePart", 3)),
            (None, None) => None,
        };
        let Some((index, expression, width)) = next else {
            parts.push(string_literal(remaining));
            break;
        };
        if index > 0 {
            parts.push(string_literal(&remaining[..index]));
        }
        parts.push(expression.to_owned());
        remaining = &remaining[index + width..];
    }
    if parts.is_empty() {
        "datePart + timePart".to_owned()
    } else {
        parts.join(" + ")
    }
}

fn indexed_string_literal(values: &[&str], index: &str) -> String {
    format!(
        "[{}][{index}]",
        values
            .iter()
            .map(|value| string_literal(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use linguini_cldr::{NumberFormatData, NumberPadding, NumberPatternPart};
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

        let arguments = number_pattern_args(&pattern, None, &numbers);
        assert!(arguments.ends_with("100, 2, 4, \"5\", 2, true, \"x\", 12, 2"));

        let runtime = formatter_data_declaration_for_target(
            "en",
            FormatterRequirements {
                number: true,
                ..FormatterRequirements::default()
            },
            EcmaScriptTarget::TypeScript,
            false,
        );
        for semantic in [
            "scaleGeneratedDecimal",
            "roundGeneratedSignificant",
            "scientificGeneratedDecimal",
            "padGeneratedNumber",
        ] {
            assert!(runtime.contains(semantic));
        }
        assert!(runtime.contains("if (exponent !== undefined && exponentDigits !== undefined)"));
    }

    #[test]
    fn generated_javascript_formatter_runtime_snapshot_is_stable() {
        let output = formatter_data_declaration_for_target(
            "ru",
            FormatterRequirements {
                number: true,
                currency: true,
                date: true,
            },
            EcmaScriptTarget::JavaScript,
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
            let generated = formatter_data_declaration_for_target(
                locale,
                FormatterRequirements {
                    number: true,
                    currency: true,
                    date: true,
                },
                EcmaScriptTarget::JavaScript,
                true,
            );
            let script = format!(
                r#"{generated}
console.log(JSON.stringify([
  formatNumber("1234567.89"),
  formatCurrency("1234567.89", 2, 0, {{ code: "ZZZ" }}),
  formatDate("2024-01-02T13:04:05Z", {{ style: "short", time_style: "short" }}),
]));
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
        let emitted = formatter_data_declaration_for_target(
            "en",
            FormatterRequirements {
                date: true,
                ..FormatterRequirements::default()
            },
            EcmaScriptTarget::TypeScript,
            false,
        );

        assert!(emitted.contains("time_style?: GeneratedDateFormatterStyle"));
        assert!(emitted.contains(&date_pattern_expression(dates.time_formats.short, &dates)));
        assert!(emitted.contains("date.getUTCHours() % 12 || 12"));
        assert!(emitted.contains("date.getUTCMinutes()"));
        assert!(emitted.contains("date.getUTCSeconds()"));
        assert!(emitted.contains("if (datePart === undefined)"));
        assert!(emitted.contains("combined = datePart + \", \" + timePart"));
        assert!(!emitted.contains("date.getHours()"));
        assert!(!emitted.contains("date.getMinutes()"));
        assert!(!emitted.contains("date.getSeconds()"));
    }

    #[test]
    fn generated_date_runtime_rejects_invalid_values() {
        let emitted = formatter_data_declaration_for_target(
            "en",
            FormatterRequirements {
                date: true,
                ..FormatterRequirements::default()
            },
            EcmaScriptTarget::TypeScript,
            false,
        );

        assert!(emitted.contains("function coerceDate(value: Date | number | string): Date"));
        assert!(emitted.contains("date = createUTCDate(year, month, day);"));
        assert!(emitted.contains("date = new Date(hasTimeZone ? value : `${value}Z`);"));
        assert!(emitted
            .contains("function createUTCDate(year: number, month: number, day: number): Date"));
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
