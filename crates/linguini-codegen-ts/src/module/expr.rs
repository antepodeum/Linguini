use std::collections::BTreeMap;

use linguini_cldr::{
    compiled_currency_formatting, compiled_date_formatting, compiled_number_formatting,
    NumberPattern,
};
use linguini_ir::{
    IrBranch, IrExpression, IrFormEntry, IrFormatter, IrFormatterKind, IrText, IrTextPart, IrValue,
};

use super::names::{escape_string, property_key, string_literal};
use super::TypeScriptOptions;

pub fn form_object(entries: &[IrFormEntry], options: &TypeScriptOptions) -> String {
    let fields = entries
        .iter()
        .filter_map(|entry| match entry {
            IrFormEntry::Attribute { name, value } => Some(format!(
                "{}: {}",
                property_key(name),
                value_expression(value, options)
            )),
            IrFormEntry::Branch(_) => None,
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {fields} }}")
}

pub fn value_expression(value: &IrValue, options: &TypeScriptOptions) -> String {
    match value {
        IrValue::Text(text) => text_expression(text, options),
        IrValue::Map(branches) => map_expression(branches, options),
        IrValue::Object(entries) => form_object(entries, options),
    }
}

pub fn map_expression(branches: &[IrBranch], options: &TypeScriptOptions) -> String {
    let items = branch_items(branches, options);
    format!(
        "(value: number | string) => selectBranch({}(value), {{ {items} }})",
        options.plural_function
    )
}

pub fn text_expression(text: &IrText, options: &TypeScriptOptions) -> String {
    text_expression_with_context(text, &BTreeMap::new(), &BTreeMap::new(), options)
}

pub fn text_expression_with_context(
    text: &IrText,
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
) -> String {
    let parts = text
        .parts
        .iter()
        .map(|part| match part {
            IrTextPart::Text(raw) => string_literal(raw),
            IrTextPart::Placeholder(expression) => {
                expression_string(expression, context, default_formatters, options)
            }
        })
        .collect::<Vec<_>>();

    if parts.is_empty() {
        "\"\"".to_owned()
    } else {
        parts.join(" + ")
    }
}

pub fn is_static_text(text: &IrText) -> bool {
    matches!(text.parts.as_slice(), [IrTextPart::Text(_)])
}

fn branch_items(branches: &[IrBranch], options: &TypeScriptOptions) -> String {
    branches
        .iter()
        .map(|branch| {
            let key = branch.keys.first().map(String::as_str).unwrap_or("_");
            format!(
                "{}: {}",
                property_key(key),
                text_expression(&branch.value, options)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn expression_string(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
) -> String {
    let value = expression_value(expression, context, options);
    let formatters = if expression.formatters.is_empty() {
        expression
            .path
            .first()
            .and_then(|name| default_formatters.get(name))
            .map_or(expression.formatters.as_slice(), Vec::as_slice)
    } else {
        expression.formatters.as_slice()
    };
    let formatted = apply_formatters(value, formatters);
    format!("String({formatted})")
}

fn expression_value(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
    options: &TypeScriptOptions,
) -> String {
    if expression.path.is_empty() {
        return "\"\"".to_owned();
    }

    if !expression.arguments.is_empty() {
        if let [root] = expression.path.as_slice() {
            if root == "plural" {
                return format!(
                    "{}({})",
                    options.plural_function,
                    expression
                        .arguments
                        .iter()
                        .map(|argument| expression_value(argument, context, options))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        if let [root] = expression.path.as_slice() {
            if let Some(ty) = context.get(root) {
                return format!(
                    "{ty}Forms[{root}]({})",
                    expression
                        .arguments
                        .iter()
                        .map(|argument| expression_value(argument, context, options))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        if let [root, property] = expression.path.as_slice() {
            if let Some(ty) = context.get(root) {
                return format!(
                    "{ty}Forms[{root}].{property}({})",
                    expression
                        .arguments
                        .iter()
                        .map(|argument| expression_value(argument, context, options))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        return format!(
            "{}({})",
            expression.path.join("."),
            expression
                .arguments
                .iter()
                .map(|argument| expression_value(argument, context, options))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    match expression.path.as_slice() {
        [root, property] => context.get(root).map_or_else(
            || expression.path.join("."),
            |ty| format!("{ty}Forms[{root}].{property}"),
        ),
        [root, property, rest @ ..] => {
            let suffix = rest
                .iter()
                .map(|part| format!(".{part}"))
                .collect::<String>();
            context.get(root).map_or_else(
                || expression.path.join("."),
                |ty| format!("{ty}Forms[{root}].{property}{suffix}"),
            )
        }
        _ => expression.path.join("."),
    }
}

fn apply_formatters(value: String, formatters: &[IrFormatter]) -> String {
    formatters.iter().fold(value, |current, formatter| {
        let formatter_options = formatter_options(formatter);
        match formatter.kind {
            IrFormatterKind::Number => format!("formatNumber({current})"),
            IrFormatterKind::Currency => format!("formatCurrency({current}, {formatter_options})"),
            IrFormatterKind::Date => format!("formatDate({current}, {formatter_options})"),
            IrFormatterKind::Unknown => current,
        }
    })
}

fn formatter_options(formatter: &IrFormatter) -> String {
    let items = formatter
        .arguments
        .iter()
        .map(|argument| {
            format!(
                "{}: \"{}\"",
                property_key(&argument.name),
                escape_string(&argument.value)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {items} }}")
}

pub fn formatter_data_declaration(locale: &str) -> String {
    let numbers = compiled_number_formatting(locale);
    let currency = compiled_currency_formatting(locale);
    let dates = compiled_date_formatting(locale);

    format!(
        "type GeneratedCurrencyFormatterOptions = {{ code?: string; accounting?: \"true\" | \"false\" }};\n\
type GeneratedDateFormatterOptions = {{ style?: \"full\" | \"long\" | \"medium\" | \"short\" }};\n\n\
{}{}{}{}",
        generated_number_function(numbers.as_ref()),
        generated_currency_function(locale, numbers.as_ref(), currency.as_ref()),
        generated_date_function(dates.as_ref()),
        formatter_helpers()
    )
}

fn generated_number_function(numbers: Option<&linguini_cldr::NumberFormatData>) -> String {
    let Some(numbers) = numbers else {
        return "function formatNumber(value: number | string): string {\n  return String(value);\n}\n\n"
            .to_owned();
    };
    format!(
        "function formatNumber(value: number | string): string {{\n  return formatGeneratedNumber(Number(value), {});\n}}\n\n",
        number_pattern_args(&numbers.decimal_pattern, None, numbers)
    )
}

fn generated_currency_function(
    locale: &str,
    numbers: Option<&linguini_cldr::NumberFormatData>,
    currency: Option<&linguini_cldr::CurrencyFormatData>,
) -> String {
    let (Some(numbers), Some(currency)) = (numbers, currency) else {
        return "function formatCurrency(value: number | string, options: GeneratedCurrencyFormatterOptions = {}): string {\n  return `${options.code ?? \"USD\"} ${value}`;\n}\n\n".to_owned();
    };
    let standard = number_pattern_args(&currency.standard_pattern, Some("symbol"), numbers);
    let accounting = number_pattern_args(
        currency
            .accounting_pattern
            .as_ref()
            .unwrap_or(&currency.standard_pattern),
        Some("symbol"),
        numbers,
    );
    format!(
        "\
function formatCurrency(
  value: number | string,
  options: GeneratedCurrencyFormatterOptions = {{}},
): string {{
  const symbol = currencySymbol(options.code ?? \"USD\");
  if (options.accounting === \"true\") {{
    return formatGeneratedNumber(Number(value), {});
  }}
  return formatGeneratedNumber(Number(value), {});
}}

function currencySymbol(currency: string): string {{
  return new Intl.NumberFormat({}, {{ style: \"currency\", currency }})
    .formatToParts(0)
    .find((part) => part.type === \"currency\")?.value ?? currency;
}}

",
        accounting,
        standard,
        string_literal(locale)
    )
}

fn generated_date_function(dates: Option<&linguini_cldr::DateFormatData>) -> String {
    let Some(dates) = dates else {
        return "function formatDate(value: Date | number | string, options: GeneratedDateFormatterOptions = {}): string {\n  return String(value);\n}\n\n".to_owned();
    };
    format!(
        "\
function formatDate(
  value: Date | number | string,
  options: GeneratedDateFormatterOptions = {{}},
): string {{
  if (typeof value === \"string\") return value;
  const date = value instanceof Date ? value : new Date(value);
  switch (options.style ?? \"medium\") {{
    case \"full\":
      return {};
    case \"long\":
      return {};
    case \"short\":
      return {};
    default:
      return {};
  }}
}}

",
        date_pattern_expression(dates.date_formats.full, dates),
        date_pattern_expression(dates.date_formats.long, dates),
        date_pattern_expression(dates.date_formats.short, dates),
        date_pattern_expression(dates.date_formats.medium, dates)
    )
}

fn formatter_helpers() -> &'static str {
    "\
function formatGeneratedNumber(
  value: number,
  prefix: string,
  suffix: string,
  negativePrefix: string | undefined,
  negativeSuffix: string | undefined,
  minIntegerDigits: number,
  minFractionDigits: number,
  maxFractionDigits: number,
  primaryGroupSize: number | undefined,
  secondaryGroupSize: number | undefined,
  decimalSymbol: string,
  groupSymbol: string,
): string {
  if (!Number.isFinite(value)) return String(value);
  const negative = value < 0 || Object.is(value, -0);
  const rounded = roundToFractionDigits(Math.abs(value), maxFractionDigits);
  let [integer, fraction = \"\"] = rounded.toFixed(maxFractionDigits).split(\".\");
  integer = integer.padStart(minIntegerDigits, \"0\");
  fraction = trimOptionalFractionDigits(fraction, minFractionDigits);

  const grouped = groupIntegerDigits(integer, primaryGroupSize, secondaryGroupSize, groupSymbol);
  const formatted = fraction ? `${grouped}${decimalSymbol}${fraction}` : grouped;
  if (negative) return `${negativePrefix ?? `-${prefix}`}${formatted}${negativeSuffix ?? suffix}`;
  return `${prefix}${formatted}${suffix}`;
}

function roundToFractionDigits(value: number, digits: number): number {
  if (digits <= 0) return Math.round(value);
  const factor = 10 ** digits;
  return Math.round(value * factor) / factor;
}

function trimOptionalFractionDigits(fraction: string, minDigits: number): string {
  while (fraction.length > minDigits && fraction.endsWith(\"0\")) {
    fraction = fraction.slice(0, -1);
  }
  return fraction;
}

function groupIntegerDigits(
  integer: string,
  primaryGroupSize: number | undefined,
  secondaryGroupSize: number | undefined,
  groupSymbol: string,
): string {
  if (!primaryGroupSize || integer.length <= primaryGroupSize) return integer;
  const groups: string[] = [];
  let end = integer.length;
  let groupSize = primaryGroupSize;
  while (end > 0) {
    const start = Math.max(0, end - groupSize);
    groups.unshift(integer.slice(start, end));
    end = start;
    groupSize = secondaryGroupSize ?? primaryGroupSize;
  }
  return groups.join(groupSymbol);
}

function padNumber(value: number, length: number): string {
  return String(value).padStart(length, \"0\");
}

"
}

fn number_pattern_args(
    pattern: &NumberPattern,
    currency_symbol: Option<&str>,
    numbers: &linguini_cldr::NumberFormatData,
) -> String {
    let negative = pattern.negative.as_ref();
    format!(
        "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
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
        string_literal(numbers.group_symbol)
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
        if matches!(character, 'y' | 'M' | 'L' | 'd' | 'E') {
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
            if next == '\'' || matches!(next, 'y' | 'M' | 'L' | 'd' | 'E') {
                break;
            }
            chars.next();
            literal.push(next);
        }
        parts.push(string_literal(&literal));
    }
    parts.join(" + ")
}

fn date_field_expression(
    field: char,
    width: usize,
    dates: &linguini_cldr::DateFormatData,
) -> String {
    match field {
        'y' if width == 2 => "padNumber(date.getFullYear() % 100, 2)".to_owned(),
        'y' => "String(date.getFullYear())".to_owned(),
        'M' | 'L' if width >= 4 => indexed_string_literal(&dates.months.wide, "date.getMonth()"),
        'M' | 'L' if width == 3 => {
            indexed_string_literal(&dates.months.abbreviated, "date.getMonth()")
        }
        'M' | 'L' if width == 2 => "padNumber(date.getMonth() + 1, 2)".to_owned(),
        'M' | 'L' => "String(date.getMonth() + 1)".to_owned(),
        'd' if width == 2 => "padNumber(date.getDate(), 2)".to_owned(),
        'd' => "String(date.getDate())".to_owned(),
        'E' if width >= 4 => indexed_string_literal(&dates.weekdays.wide, "date.getDay()"),
        'E' => indexed_string_literal(&dates.weekdays.abbreviated, "date.getDay()"),
        _ => "\"\"".to_owned(),
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
