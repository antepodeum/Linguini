use std::collections::BTreeMap;

use linguini_cldr::{
    compiled_currency_formatting, compiled_currency_fraction, compiled_date_formatting,
    compiled_number_formatting, NumberPattern,
};
use linguini_ir::{
    is_plural_intrinsic, IrBranch, IrExpression, IrExpressionKind, IrFormEntry, IrFormatter,
    IrFormatterKind, IrFunctionBranch, IrFunctionBranchValue, IrFunctionParameter,
    IrInlineFunctionInput, IrText, IrTextPart, IrValue,
};

use super::formatters::FormatterRequirements;
use super::names::{
    escape_string, form_binding_name, path_expression, property_access, property_key,
    safe_identifier, string_literal, ts_type,
};
use super::TypeScriptOptions;

pub fn form_object(entries: &[IrFormEntry], options: &TypeScriptOptions) -> String {
    let fields = entries
        .iter()
        .filter_map(|entry| match entry {
            IrFormEntry::Attribute {
                name,
                parameters,
                value,
            } => Some(format!(
                "{}: {}",
                property_key(name),
                value_expression_with_parameters(value, parameters, options)
            )),
            IrFormEntry::Branch(_) => None,
        })
        .collect::<Vec<_>>()
        .join(", ");
    let object = format!("{{ {fields} }}");
    let branches = entries
        .iter()
        .filter_map(|entry| match entry {
            IrFormEntry::Attribute { .. } => None,
            IrFormEntry::Branch(branch) => Some(branch.clone()),
        })
        .collect::<Vec<_>>();

    if branches.is_empty() {
        object
    } else {
        let dispatcher = map_expression(&branches, &[], options);
        if fields.is_empty() {
            dispatcher
        } else {
            format!("Object.assign({dispatcher}, {object})")
        }
    }
}

fn value_expression_with_parameters(
    value: &IrValue,
    parameters: &[IrFunctionParameter],
    options: &TypeScriptOptions,
) -> String {
    match value {
        IrValue::Text(text) => text_expression(text, options),
        IrValue::Map(branches) => map_expression(branches, parameters, options),
        IrValue::Object(entries) => form_object(entries, options),
    }
}

pub fn map_expression(
    branches: &[IrBranch],
    parameters: &[IrFunctionParameter],
    options: &TypeScriptOptions,
) -> String {
    let context = parameters
        .iter()
        .filter_map(|parameter| {
            parameter
                .name
                .as_ref()
                .map(|name| (name.clone(), parameter.ty.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let items = branch_items(branches, &context, options);
    let parameter = parameters
        .first()
        .and_then(|parameter| parameter.name.as_deref())
        .map(safe_identifier)
        .unwrap_or_else(|| "value".to_owned());
    let selector = parameters
        .first()
        .filter(|parameter| parameter.ty != "Plural")
        .map_or_else(
            || format!("{}({parameter})", options.plural_function),
            |_| format!("String({parameter})"),
        );
    let parameter_type = parameters.first().map_or_else(
        || "number | bigint | string".to_owned(),
        |parameter| {
            if parameter.ty == "Plural" {
                "number | bigint | string".to_owned()
            } else {
                ts_type(&parameter.ty)
            }
        },
    );
    format!("({parameter}: {parameter_type}) => selectBranch({selector}, {{ {items} }})")
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

fn branch_items(
    branches: &[IrBranch],
    context: &BTreeMap<String, String>,
    options: &TypeScriptOptions,
) -> String {
    branches
        .iter()
        .flat_map(|branch| {
            let value =
                text_expression_with_context(&branch.value, context, &BTreeMap::new(), options);
            if branch.keys.is_empty() {
                return vec![format!("{}: {value}", property_key("_"))];
            }
            branch
                .keys
                .iter()
                .map(|key| format!("{}: {value}", property_key(key)))
                .collect::<Vec<_>>()
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
    let value = expression_value(expression, context, default_formatters, options);
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
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
) -> String {
    if let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind {
        return inline_function_expression(inputs, branches, context, default_formatters, options);
    }
    if expression.path.is_empty() {
        return "\"\"".to_owned();
    }

    if expression.kind == IrExpressionKind::Call {
        if let [root] = expression.path.as_slice() {
            if is_plural_intrinsic(root) {
                return format!(
                    "{}({})",
                    options.plural_function,
                    expression
                        .arguments
                        .iter()
                        .map(|argument| {
                            expression_value(argument, context, default_formatters, options)
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        if let [root] = expression.path.as_slice() {
            if let Some(ty) = context.get(root) {
                return format!(
                    "{}[{}]({})",
                    form_binding_name(ty),
                    safe_identifier(root),
                    expression
                        .arguments
                        .iter()
                        .map(|argument| {
                            expression_value(argument, context, default_formatters, options)
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        if let [root, property] = expression.path.as_slice() {
            if let Some(ty) = context.get(root) {
                return format!(
                    "{}[{}]{}({})",
                    form_binding_name(ty),
                    safe_identifier(root),
                    property_access(property),
                    expression
                        .arguments
                        .iter()
                        .map(|argument| {
                            expression_value(argument, context, default_formatters, options)
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        return format!(
            "{}({})",
            path_expression(&expression.path),
            expression
                .arguments
                .iter()
                .map(|argument| {
                    expression_value(argument, context, default_formatters, options)
                })
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    match expression.path.as_slice() {
        [root, property] => context.get(root).map_or_else(
            || path_expression(&expression.path),
            |ty| {
                format!(
                    "{}[{}]{}",
                    form_binding_name(ty),
                    safe_identifier(root),
                    property_access(property)
                )
            },
        ),
        [root, property, rest @ ..] => {
            let suffix = rest
                .iter()
                .map(|part| property_access(part))
                .collect::<String>();
            context.get(root).map_or_else(
                || path_expression(&expression.path),
                |ty| {
                    format!(
                        "{}[{}]{}{suffix}",
                        form_binding_name(ty),
                        safe_identifier(root),
                        property_access(property)
                    )
                },
            )
        }
        _ => path_expression(&expression.path),
    }
}

pub(super) fn function_dispatch_expression(
    parameters: &[IrFunctionParameter],
    branches: &[IrFunctionBranch],
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
) -> String {
    let selectors = parameters
        .iter()
        .enumerate()
        .filter(|(_, parameter)| parameter.name.is_none())
        .map(|(index, parameter)| DispatchSelector {
            value: format!("__lgl_p{index}"),
            normalize_plural: parameter.ty == "Plural",
        })
        .collect::<Vec<_>>();
    dispatch_expression_level(
        &selectors,
        branches,
        0,
        context,
        default_formatters,
        options,
    )
}

#[derive(Debug)]
struct DispatchSelector {
    value: String,
    normalize_plural: bool,
}

fn inline_function_expression(
    inputs: &[IrInlineFunctionInput],
    branches: &[IrFunctionBranch],
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
) -> String {
    let mut parameters = Vec::with_capacity(inputs.len());
    let mut arguments = Vec::with_capacity(inputs.len());
    let mut selectors = Vec::new();
    let mut branch_context = context.clone();
    let mut branch_formatters = default_formatters.clone();

    for (index, input) in inputs.iter().enumerate() {
        match input {
            IrInlineFunctionInput::Selector { value, .. } => {
                let parameter = format!("__lgl_inline_selector_{index}");
                parameters.push(parameter.clone());
                arguments.push(expression_value(
                    value,
                    context,
                    default_formatters,
                    options,
                ));
                // Explicit `Plural(value)` has already produced its category.
                // A directly referenced `Plural` value still accepts either a
                // numeric operand or a pre-classified category, matching named
                // function dispatch.
                selectors.push(DispatchSelector {
                    value: parameter,
                    normalize_plural: inferred_expression_type(value, context).as_deref()
                        == Some("Plural")
                        && !is_plural_intrinsic_call(value),
                });
            }
            IrInlineFunctionInput::Binding { name, value, .. } => {
                parameters.push(safe_identifier(name));
                arguments.push(expression_value(
                    value,
                    context,
                    default_formatters,
                    options,
                ));
                branch_context.insert(
                    name.clone(),
                    inferred_expression_type(value, context).unwrap_or_else(|| "String".to_owned()),
                );
                if value.formatters.is_empty()
                    && value.kind == IrExpressionKind::Reference
                    && value.path.len() == 1
                {
                    if let Some(formatters) = default_formatters.get(&value.path[0]) {
                        branch_formatters.insert(name.clone(), formatters.clone());
                    }
                }
            }
        }
    }

    let body = dispatch_expression_level(
        &selectors,
        branches,
        0,
        &branch_context,
        &branch_formatters,
        options,
    );
    format!(
        "(({}) => {body})({})",
        parameters.join(", "),
        arguments.join(", ")
    )
}

fn is_plural_intrinsic_call(expression: &IrExpression) -> bool {
    expression.kind == IrExpressionKind::Call
        && expression.path.len() == 1
        && is_plural_intrinsic(&expression.path[0])
}

fn inferred_expression_type(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
) -> Option<String> {
    match &expression.kind {
        IrExpressionKind::InlineFunction { .. } => Some("String".to_owned()),
        IrExpressionKind::Call => expression
            .path
            .first()
            .filter(|_| expression.path.len() == 1)
            .filter(|name| is_plural_intrinsic(name))
            .map(|_| "Plural".to_owned())
            .or_else(|| Some("String".to_owned())),
        IrExpressionKind::Reference if expression.path.len() == 1 => {
            context.get(&expression.path[0]).cloned()
        }
        IrExpressionKind::Reference => None,
    }
}

fn dispatch_expression_level(
    selectors: &[DispatchSelector],
    branches: &[IrFunctionBranch],
    depth: usize,
    context: &BTreeMap<String, String>,
    default_formatters: &BTreeMap<String, Vec<IrFormatter>>,
    options: &TypeScriptOptions,
) -> String {
    let selector = selectors.get(depth).map_or_else(
        || "undefined".to_owned(),
        |selector| {
            if selector.normalize_plural {
                format!("{}({})", options.plural_function, selector.value)
            } else {
                format!("String({})", selector.value)
            }
        },
    );
    let items = branches
        .iter()
        .map(|branch| {
            let value = match &branch.value {
                IrFunctionBranchValue::Text(text) => {
                    text_expression_with_context(text, context, default_formatters, options)
                }
                IrFunctionBranchValue::Dispatch(children) => dispatch_expression_level(
                    selectors,
                    children,
                    depth + 1,
                    context,
                    default_formatters,
                    options,
                ),
            };
            format!("{}: (): string => {value}", property_key(&branch.key))
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("selectBranch({selector}, {{ {items} }})()")
}

fn apply_formatters(value: String, formatters: &[IrFormatter]) -> String {
    formatters.iter().fold(value, |current, formatter| {
        let formatter_options = formatter_options(formatter);
        match &formatter.kind {
            IrFormatterKind::Number => format!("formatNumber({current})"),
            IrFormatterKind::Currency => {
                let currency = formatter
                    .arguments
                    .iter()
                    .find(|argument| argument.name == "code")
                    .map_or("USD", |argument| argument.value.as_str());
                let fraction = compiled_currency_fraction(currency)
                    .expect("validated currency formatter must have CLDR fraction rules");
                format!(
                    "formatCurrency({current}, {}, {}, {formatter_options})",
                    fraction.digits, fraction.rounding
                )
            }
            IrFormatterKind::Date => format!("formatDate({current}, {formatter_options})"),
            IrFormatterKind::Unknown(_) => current,
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

pub fn formatter_data_declaration(locale: &str, requirements: FormatterRequirements) -> String {
    let mut output = "type GeneratedNumeric = number | bigint | string;\n".to_owned();
    if requirements.currency {
        output.push_str(
            "type GeneratedCurrencyFormatterOptions = { code?: string; accounting?: \"true\" | \"false\" };\n",
        );
    }
    if requirements.date {
        output.push_str(
            "type GeneratedDateFormatterOptions = { style?: \"full\" | \"long\" | \"medium\" | \"short\" };\n",
        );
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
        ));
    }
    if requirements.date {
        let dates = compiled_date_formatting(locale)
            .expect("validated locale must have required CLDR date formatting data");
        output.push_str(&generated_date_function(&dates));
    }
    if requirements.needs_number_data() {
        output.push_str(number_formatter_helpers());
    }
    if requirements.date {
        output.push_str(date_formatter_helpers());
    }
    output
}

fn generated_number_function(numbers: &linguini_cldr::NumberFormatData) -> String {
    format!(
        "function formatNumber(value: GeneratedNumeric): string {{\n  return formatGeneratedNumber(value, {});\n}}\n\n",
        number_pattern_args(&numbers.decimal_pattern, None, numbers)
    )
}

fn generated_currency_function(
    locale: &str,
    numbers: &linguini_cldr::NumberFormatData,
    currency: &linguini_cldr::CurrencyFormatData,
) -> String {
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
  value: GeneratedNumeric,
  fractionDigits: number,
  roundingIncrement: number,
  options: GeneratedCurrencyFormatterOptions = {{}},
): string {{
  const symbol = currencySymbol(options.code ?? \"USD\");
  if (options.accounting === \"true\") {{
    return formatGeneratedNumber(value, {}, fractionDigits, fractionDigits, roundingIncrement);
  }}
  return formatGeneratedNumber(value, {}, fractionDigits, fractionDigits, roundingIncrement);
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

fn generated_date_function(dates: &linguini_cldr::DateFormatData) -> String {
    format!(
        "\
function formatDate(
  value: Date | number | string,
  options: GeneratedDateFormatterOptions = {{}},
): string {{
  const date = coerceDate(value);
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

fn number_formatter_helpers() -> &'static str {
    r#"type GeneratedDecimal = { negative: boolean; integer: string; fraction: string };
const MAX_GENERATED_DECIMAL_DIGITS = 8192;

function formatGeneratedNumber(
  value: GeneratedNumeric,
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
  minFractionDigitsOverride?: number,
  maxFractionDigitsOverride?: number,
  roundingIncrement = 0,
): string {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const effectiveMinFractionDigits = minFractionDigitsOverride ?? minFractionDigits;
  const effectiveMaxFractionDigits = maxFractionDigitsOverride ?? maxFractionDigits;
  const rounded = roundGeneratedDecimal(decimal, effectiveMaxFractionDigits, roundingIncrement);
  let integer = rounded.integer.padStart(minIntegerDigits, "0");
  const fraction = trimOptionalFractionDigits(
    rounded.fraction,
    effectiveMinFractionDigits,
  );

  integer = groupIntegerDigits(integer, primaryGroupSize, secondaryGroupSize, groupSymbol);
  const formatted = fraction ? `${integer}${decimalSymbol}${fraction}` : integer;
  if (decimal.negative) {
    return `${negativePrefix ?? `-${prefix}`}${formatted}${negativeSuffix ?? suffix}`;
  }
  return `${prefix}${formatted}${suffix}`;
}

function parseGeneratedDecimal(value: GeneratedNumeric): GeneratedDecimal | undefined {
  if (typeof value === "number" && !Number.isFinite(value)) return undefined;
  const negativeZero = typeof value === "number" && Object.is(value, -0);
  const source = String(value);
  const match = /^([+-]?)(\d*)(?:\.(\d*))?(?:[eE]([+-]?\d+))?$/.exec(source);
  if (!match || (match[2] === "" && (match[3] ?? "") === "")) throwInvalidNumber();

  const whole = match[2];
  const fractional = match[3] ?? "";
  const exponent = Number(match[4] ?? "0");
  if (
    !Number.isSafeInteger(exponent) ||
    Math.abs(exponent) > MAX_GENERATED_DECIMAL_DIGITS ||
    (match[4]?.length ?? 0) > MAX_GENERATED_DECIMAL_DIGITS ||
    whole.length + fractional.length > MAX_GENERATED_DECIMAL_DIGITS
  ) {
    throwInvalidNumber();
  }

  const digits = `${whole}${fractional}` || "0";
  const decimalPosition = whole.length + exponent;
  const expandedLength = decimalPosition <= 0
    ? -decimalPosition + digits.length
    : Math.max(decimalPosition, digits.length);
  if (expandedLength > MAX_GENERATED_DECIMAL_DIGITS) throwInvalidNumber();

  let integer: string;
  let fraction: string;
  if (decimalPosition <= 0) {
    integer = "0";
    fraction = `${"0".repeat(-decimalPosition)}${digits}`;
  } else if (decimalPosition >= digits.length) {
    integer = `${digits}${"0".repeat(decimalPosition - digits.length)}`;
    fraction = "";
  } else {
    integer = digits.slice(0, decimalPosition);
    fraction = digits.slice(decimalPosition);
  }
  integer = integer.replace(/^0+(?=\d)/, "");
  fraction = fraction.replace(/0+$/, "");
  return {
    negative: match[1] === "-" || negativeZero,
    integer,
    fraction,
  };
}

function roundGeneratedDecimal(
  decimal: GeneratedDecimal,
  fractionDigits: number,
  roundingIncrement: number,
): { integer: string; fraction: string } {
  if (
    !Number.isSafeInteger(fractionDigits) ||
    fractionDigits < 0 ||
    fractionDigits > MAX_GENERATED_DECIMAL_DIGITS ||
    !Number.isSafeInteger(roundingIncrement) ||
    roundingIncrement < 0
  ) {
    throwInvalidNumber();
  }

  const keptFraction = decimal.fraction.slice(0, fractionDigits).padEnd(fractionDigits, "0");
  const discarded = decimal.fraction.slice(fractionDigits);
  const scaled = BigInt(`${decimal.integer}${keptFraction}` || "0");
  const quantum = BigInt(roundingIncrement || 1);
  const remainder = scaled % quantum;
  let roundUp: boolean;
  if (discarded === "") {
    roundUp = remainder * 2n >= quantum;
  } else {
    const denominator = 10n ** BigInt(discarded.length);
    const exactRemainder = remainder * denominator + BigInt(discarded);
    roundUp = exactRemainder * 2n >= quantum * denominator;
  }
  const rounded = (scaled / quantum + (roundUp ? 1n : 0n)) * quantum;
  const digits = rounded.toString().padStart(fractionDigits + 1, "0");
  return fractionDigits === 0
    ? { integer: digits, fraction: "" }
    : {
        integer: digits.slice(0, -fractionDigits),
        fraction: digits.slice(-fractionDigits),
      };
}

function trimOptionalFractionDigits(fraction: string, minDigits: number): string {
  while (fraction.length > minDigits && fraction.endsWith("0")) {
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

function throwInvalidNumber(): never {
  throw new RangeError("Linguini: invalid numeric value");
}

"#
}

fn date_formatter_helpers() -> &'static str {
    r#"function padNumber(value: number, length: number): string {
  return String(value).padStart(length, "0");
}

function coerceDate(value: Date | number | string): Date {
  let date: Date;
  if (value instanceof Date) {
    date = value;
  } else if (typeof value === "string") {
    const dateOnly = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
    if (dateOnly) {
      const year = Number(dateOnly[1]);
      const month = Number(dateOnly[2]);
      const day = Number(dateOnly[3]);
      date = createUTCDate(year, month, day);
    } else {
      const dateTime =
        /^(\d{4})-(\d{2})-(\d{2})T\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?(?:Z|[+-]\d{2}:\d{2})?$/.exec(value);
      if (!dateTime) throwInvalidDate();
      createUTCDate(Number(dateTime[1]), Number(dateTime[2]), Number(dateTime[3]));
      const hasTimeZone = /(?:Z|[+-]\d{2}:\d{2})$/.test(value);
      date = new Date(hasTimeZone ? value : `${value}Z`);
    }
  } else {
    date = new Date(value);
  }
  if (!Number.isFinite(date.getTime())) {
    throwInvalidDate();
  }
  return date;
}

function createUTCDate(year: number, month: number, day: number): Date {
  const date = new Date(0);
  date.setUTCHours(0, 0, 0, 0);
  date.setUTCFullYear(year, month - 1, day);
  if (
    date.getUTCFullYear() !== year ||
    date.getUTCMonth() !== month - 1 ||
    date.getUTCDate() !== day
  ) {
    throwInvalidDate();
  }
  return date;
}

function throwInvalidDate(): never {
  throw new RangeError("Linguini: invalid date value");
}

"#
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

#[cfg(test)]
mod tests {
    use super::{
        date_pattern_expression, expression_value, form_object, formatter_data_declaration,
        FormatterRequirements, TypeScriptOptions,
    };
    use linguini_ir::{lower_locale, IrExpression, IrExpressionKind};
    use linguini_syntax::{parse_locale, Span};
    use std::collections::BTreeMap;

    fn expression(kind: IrExpressionKind, path: &[&str]) -> IrExpression {
        IrExpression {
            kind,
            path: path.iter().map(|part| (*part).to_owned()).collect(),
            arguments: Vec::new(),
            formatters: Vec::new(),
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn zero_argument_global_call_differs_from_reference() {
        let context = BTreeMap::new();
        let options = TypeScriptOptions::default();

        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["ready"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "ready()"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Reference, &["ready"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "ready"
        );
    }

    #[test]
    fn canonical_plural_call_uses_configured_function() {
        let context = BTreeMap::new();
        let options = TypeScriptOptions {
            plural_function: "selectPlural".to_owned(),
            ..TypeScriptOptions::default()
        };

        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["Plural"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "selectPlural()"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["plural"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "plural()"
        );
    }

    #[test]
    fn zero_argument_form_calls_differ_from_form_references() {
        let context = BTreeMap::from([("item".to_owned(), "Item".to_owned())]);
        let options = TypeScriptOptions::default();

        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["item"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "__lgl_form_4974656D[item]()"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Reference, &["item"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "item"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Call, &["item", "label"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "__lgl_form_4974656D[item].label()"
        );
        assert_eq!(
            expression_value(
                &expression(IrExpressionKind::Reference, &["item", "label"]),
                &context,
                &BTreeMap::new(),
                &options
            ),
            "__lgl_form_4974656D[item].label"
        );
    }

    #[test]
    fn explicit_non_plural_form_selector_uses_string_dispatch() {
        let locale = lower_locale(
            &parse_locale(
                "impl Fruit {\n  apple {\n    form label(gender: Gender) {\n      male => He\n      _ => They\n    }\n  }\n}\n",
            )
            .expect("locale"),
        );
        let entries = &locale.forms[0].variants[0].entries;

        let emitted = form_object(entries, &TypeScriptOptions::default());

        assert!(emitted.contains("label: (gender: Gender) => selectBranch(String(gender),"));
        assert!(!emitted.contains("pluralEn(gender)"));
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
    fn generated_date_runtime_rejects_invalid_values() {
        let emitted = formatter_data_declaration(
            "en",
            FormatterRequirements {
                date: true,
                ..FormatterRequirements::default()
            },
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
