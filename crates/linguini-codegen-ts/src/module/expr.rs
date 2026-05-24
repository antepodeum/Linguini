use std::collections::BTreeMap;

use linguini_cldr::{
    compiled_currency_formatting, compiled_date_formatting, compiled_number_formatting,
    NumberPattern, NumberPatternPart,
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
    let formatted = apply_formatters(value, formatters, options);
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

fn apply_formatters(
    value: String,
    formatters: &[IrFormatter],
    _options: &TypeScriptOptions,
) -> String {
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
    format!(
        "{}\n{}",
        formatter_constants(locale),
        formatter_functions(locale)
    )
}

fn formatter_constants(locale: &str) -> String {
    let numbers = compiled_number_formatting(locale);
    let currency = compiled_currency_formatting(locale);
    let dates = compiled_date_formatting(locale);

    format!(
        "type GeneratedNumberPatternPart = {{ prefix: string; suffix: string; minIntegerDigits: number; minFractionDigits: number; maxFractionDigits: number; primaryGroupSize?: number; secondaryGroupSize?: number }};\n\
type GeneratedNumberPattern = {{ positive: GeneratedNumberPatternPart; negative?: GeneratedNumberPatternPart }};\n\
type GeneratedCurrencyFormatterOptions = {{ code?: string; accounting?: \"true\" | \"false\" }};\n\
type GeneratedDateFormatterOptions = {{ style?: \"full\" | \"long\" | \"medium\" | \"short\" }};\n\n\
const FORMATTER_LOCALE = {};\n\
const NUMBER_DECIMAL_SYMBOL = {};\n\
const NUMBER_GROUP_SYMBOL = {};\n\
const NUMBER_DECIMAL_PATTERN: GeneratedNumberPattern | undefined = {};\n\
const CURRENCY_STANDARD_PATTERN: GeneratedNumberPattern | undefined = {};\n\
const CURRENCY_ACCOUNTING_PATTERN: GeneratedNumberPattern | undefined = {};\n\
const DATE_FORMATS = {};\n",
        string_literal(locale),
        numbers.as_ref().map_or_else(
            || "undefined".to_owned(),
            |data| string_literal(&data.decimal_symbol)
        ),
        numbers.as_ref().map_or_else(
            || "undefined".to_owned(),
            |data| string_literal(&data.group_symbol)
        ),
        numbers.as_ref().map_or_else(
            || "undefined".to_owned(),
            |data| number_pattern_literal(&data.decimal_pattern)
        ),
        currency.as_ref().map_or_else(
            || "undefined".to_owned(),
            |data| number_pattern_literal(&data.standard_pattern)
        ),
        currency
            .as_ref()
            .and_then(|data| data.accounting_pattern.as_ref())
            .map_or_else(|| "undefined".to_owned(), number_pattern_literal),
        dates.as_ref().map_or_else(
            || "undefined".to_owned(),
            |data| widths_literal(&data.date_formats)
        )
    )
}

fn formatter_functions(locale: &str) -> String {
    let locale = string_literal(locale);
    format!(
        "\
function formatNumber(value: number | string): string {{
  return formatGeneratedNumber(Number(value), NUMBER_DECIMAL_PATTERN);
}}

function formatCurrency(
  value: number | string,
  options: GeneratedCurrencyFormatterOptions = {{}},
): string {{
  const currency = options.code ?? \"USD\";
  const pattern = options.accounting === \"true\"
    ? CURRENCY_ACCOUNTING_PATTERN ?? CURRENCY_STANDARD_PATTERN
    : CURRENCY_STANDARD_PATTERN;
  return formatGeneratedNumber(Number(value), pattern, currencySymbol(currency));
}}

function formatDate(
  value: Date | number | string,
  options: GeneratedDateFormatterOptions = {{}},
): string {{
  if (typeof value === \"string\") return value;
  const style = options.style ?? \"medium\";
  if (!DATE_FORMATS?.[style]) return new Intl.DateTimeFormat(FORMATTER_LOCALE).format(value);
  return new Intl.DateTimeFormat(FORMATTER_LOCALE, {{ dateStyle: style }}).format(value);
}}

function formatGeneratedNumber(
  value: number,
  pattern: GeneratedNumberPattern | undefined,
  currency?: string,
): string {{
  if (!Number.isFinite(value)) return String(value);
  const negative = value < 0 || Object.is(value, -0);
  const positive = pattern?.positive;
  const part = negative ? pattern?.negative ?? negativePatternPart(positive) : positive;
  if (!part) return String(value);

  const rounded = roundToFractionDigits(Math.abs(value), part.maxFractionDigits);
  let [integer, fraction = \"\"] = rounded.toFixed(part.maxFractionDigits).split(\".\");
  integer = integer.padStart(part.minIntegerDigits, \"0\");
  fraction = trimOptionalFractionDigits(fraction, part.minFractionDigits);

  const grouped = groupIntegerDigits(integer, part.primaryGroupSize, part.secondaryGroupSize);
  const formatted = fraction ? `${{grouped}}${{NUMBER_DECIMAL_SYMBOL ?? \".\"}}${{fraction}}` : grouped;
  return `${{formatNumberAffix(part.prefix, currency)}}${{formatted}}${{formatNumberAffix(part.suffix, currency)}}`;
}}

function negativePatternPart(part: GeneratedNumberPatternPart | undefined): GeneratedNumberPatternPart | undefined {{
  return part ? {{ ...part, prefix: `-${{part.prefix}}` }} : undefined;
}}

function roundToFractionDigits(value: number, digits: number): number {{
  if (digits <= 0) return Math.round(value);
  const factor = 10 ** digits;
  return Math.round(value * factor) / factor;
}}

function trimOptionalFractionDigits(fraction: string, minDigits: number): string {{
  while (fraction.length > minDigits && fraction.endsWith(\"0\")) {{
    fraction = fraction.slice(0, -1);
  }}
  return fraction;
}}

function groupIntegerDigits(
  integer: string,
  primaryGroupSize: number | undefined,
  secondaryGroupSize: number | undefined,
): string {{
  if (!primaryGroupSize || integer.length <= primaryGroupSize) return integer;
  const groups: string[] = [];
  let end = integer.length;
  let groupSize = primaryGroupSize;
  while (end > 0) {{
    const start = Math.max(0, end - groupSize);
    groups.unshift(integer.slice(start, end));
    end = start;
    groupSize = secondaryGroupSize ?? primaryGroupSize;
  }}
  return groups.join(NUMBER_GROUP_SYMBOL ?? \",\");
}}

function formatNumberAffix(affix: string, currency: string | undefined): string {{
  let output = \"\";
  for (const character of affix) {{
    output += character === \"¤\" ? currency ?? \"\" : character;
  }}
  return output;
}}

function currencySymbol(currency: string): string {{
  return new Intl.NumberFormat({}, {{ style: \"currency\", currency }})
    .formatToParts(0)
    .find((part) => part.type === \"currency\")?.value ?? currency;
}}

",
        locale
    )
}

fn number_pattern_literal(pattern: &NumberPattern) -> String {
    format!(
        "{{ positive: {}, negative: {} }}",
        number_pattern_part_literal(&pattern.positive),
        pattern
            .negative
            .as_ref()
            .map_or_else(|| "undefined".to_owned(), number_pattern_part_literal)
    )
}

fn number_pattern_part_literal(part: &NumberPatternPart) -> String {
    let primary_group_size = option_u8_literal(part.primary_group_size);
    let secondary_group_size = option_u8_literal(part.secondary_group_size);
    format!(
        "{{ prefix: {}, suffix: {}, minIntegerDigits: {}, minFractionDigits: {}, maxFractionDigits: {}, primaryGroupSize: {}, secondaryGroupSize: {} }}",
        string_literal(&part.prefix),
        string_literal(&part.suffix),
        part.min_integer_digits,
        part.min_fraction_digits,
        part.max_fraction_digits,
        primary_group_size,
        secondary_group_size
    )
}

fn option_u8_literal(value: Option<u8>) -> String {
    value.map_or_else(|| "undefined".to_owned(), |value| value.to_string())
}

fn widths_literal(widths: &linguini_cldr::FormatWidths) -> String {
    format!(
        "{{ full: {}, long: {}, medium: {}, short: {} }}",
        string_literal(&widths.full),
        string_literal(&widths.long),
        string_literal(&widths.medium),
        string_literal(&widths.short)
    )
}
