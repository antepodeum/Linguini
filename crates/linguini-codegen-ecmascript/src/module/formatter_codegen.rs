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
    let mut date_helpers = DateHelperRequirements::default();
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
                    plans.push(plan.clone());
                    output.push_str(&crate::templates::render(
                        "formatters/number",
                        context! { name, plan, exported => export_helpers },
                    ));
                } else {
                    let plan = number_plan(&pattern, None, &numbers, None);
                    plans.push(plan.clone());
                    output.push_str(&crate::templates::render(
                        "formatters/number",
                        context! { name, plan, exported => export_helpers },
                    ));
                }
            }
            linguini_ir::IrFormatterKind::Date => {
                let dates = compiled_date_formatting(locale).expect("validated date locale");
                let (source, helpers) =
                    generated_date_request(&dates, request, &name, export_helpers);
                output.push_str(&source);
                date_helpers.padding |= helpers.padding;
                date_helpers.digits |= helpers.digits;
            }
            linguini_ir::IrFormatterKind::Unknown(_) => unreachable!("validated formatter request"),
        }
    }
    if needs_number {
        output.push_str(&number_helpers(&plans));
    }
    if needs_date {
        output.push_str(&date_formatter_helpers(date_helpers.padding));
    }
    if date_helpers.digits {
        output.push_str(&digit_formatter_helper());
    }
    output
}

fn number_helpers(plans: &[minijinja::Value]) -> String {
    use minijinja::context;
    use std::collections::BTreeMap;
    let unique = |key: &str, selected: &dyn Fn(&minijinja::Value) -> bool| {
        plans
            .iter()
            .filter(|plan| selected(plan))
            .map(|plan| {
                (
                    plan.get_attr(key)
                        .expect("compiled number-plan field")
                        .to_string(),
                    plan.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect::<Vec<_>>()
    };
    let present = |plan: &minijinja::Value, key: &str| !plan.get_attr(key).unwrap().is_none();
    crate::templates::render(
        "number_helpers",
        context! {
            rounding_plans => unique("rounding_helper", &|plan| !present(plan, "max_significant")),
            scale_plans => unique("scale", &|plan| plan.get_attr("scale").unwrap() != minijinja::Value::from(1)),
            significant_plans => unique("max_significant", &|plan| present(plan, "max_significant")),
            padding_plans => unique("min_significant", &|plan| present(plan, "min_significant")),
            scientific => plans.iter().any(|plan| present(plan, "exponent_digits")),
        },
    )
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
    let rounding = linguini_codegen::NumberRoundingPlan::new(
        positive.max_fraction_digits,
        rounding.or(positive.rounding_increment).unwrap_or("1"),
    )
    .expect("validated CLDR rounding increment");
    let rounding_helper = if rounding.increment == "1" {
        format!("roundTo{}FractionDigits", rounding.fraction_digits)
    } else {
        format!(
            "roundTo{}FractionDigitsIncrement{}",
            rounding.fraction_digits, rounding.increment
        )
    };
    let scale_shift = match positive.scale {
        1 => 0,
        100 => 2,
        1000 => 3,
        _ => unreachable!("validated CLDR scale"),
    };
    context! {
        prefix, suffix, negative_prefix, negative_suffix,
        min_integer => positive.min_integer_digits, min_fraction => positive.min_fraction_digits,
        max_fraction => positive.max_fraction_digits, primary_group => positive.primary_group_size,
        secondary_group => positive.secondary_group_size.or(positive.primary_group_size),
        decimal_symbol => numbers.decimal_symbol, group_symbol => numbers.group_symbol, digits => numbers.digits,
        scale => positive.scale, min_significant => positive.min_significant_digits, max_significant => positive.max_significant_digits,
        rounding_helper, rounding => rounding.increment, rounding_half => rounding.half_integer,
        rounding_half_fraction => rounding.half_fraction, scale_shift,
        exponent_digits => positive.exponent_digits, exponent_sign => positive.exponent_sign_always, padding,
    }
}

#[derive(Default)]
struct DateHelperRequirements {
    padding: bool,
    digits: bool,
}

fn generated_date_request(
    dates: &linguini_cldr::DateFormatData,
    request: &super::formatters::FormatterRequest,
    name: &str,
    exported: bool,
) -> (String, DateHelperRequirements) {
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
    let (parts, tables, helpers) = date_parts(&pattern, dates, name);
    (
        crate::templates::render(
            "formatters/date",
            minijinja::context! { name, exported, parts, tables, digits => dates.digits },
        ),
        helpers,
    )
}

fn date_parts(
    pattern: &str,
    dates: &linguini_cldr::DateFormatData,
    name: &str,
) -> (
    Vec<minijinja::Value>,
    Vec<minijinja::Value>,
    DateHelperRequirements,
) {
    use minijinja::context;
    let mut helpers = DateHelperRequirements::default();
    let calendar = linguini_cldr::compiled_calendar_formatting(dates.locale, "gregory")
        .expect("validated locale has pinned Gregorian symbols");
    let mut parts = Vec::new();
    let mut tables = std::collections::BTreeMap::<String, Vec<&str>>::new();
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
        let symbols = linguini_codegen::date_field_symbols(&calendar, character, width)
            .unwrap_or_else(|| {
                panic!(
                    "missing pinned CLDR symbols: {} {character}/{width}",
                    dates.locale
                )
            });
        if let Some(values) = symbols.names {
            let proposed = format!(
                "{name}_{}{}_{width}",
                if character == 'L' { "standalone_" } else { "" },
                if matches!(character, 'M' | 'L') {
                    "months"
                } else {
                    "weekdays"
                }
            );
            let binding = tables
                .iter()
                .find(|(_, existing)| **existing == values)
                .map(|(binding, _)| binding.clone())
                .unwrap_or(proposed);
            tables.entry(binding.clone()).or_insert(values);
            table = Some(binding);
        }
        let numeric = matches!(
            character,
            'y' | 'd' | 'H' | 'h' | 'K' | 'k' | 'm' | 's' | 'S'
        ) || matches!(character, 'M' | 'L') && width <= 2;
        helpers.padding |= character == 'S' || (numeric && width == 2);
        helpers.digits |= numeric && dates.digits != "0123456789";
        parts.push(context! { kind => "field", field => character.to_string(), width, table, numeric, am => symbols.am, pm => symbols.pm, ad => symbols.ad, bc => symbols.bc });
    }
    assert!(!quoted, "validated CLDR pattern has balanced quoting");
    if !literal.is_empty() {
        parts.push(context! { kind => "literal", value => literal });
    }
    (
        parts,
        tables
            .into_iter()
            .map(|(name, values)| context! { name, values })
            .collect(),
        helpers,
    )
}

#[cfg(test)]
fn date_pattern_expression(pattern: &str, dates: &linguini_cldr::DateFormatData) -> String {
    let (parts, _, _) = date_parts(pattern, dates, "test");
    crate::templates::render(
        "formatters/date_expression",
        minijinja::context! { parts, digits => dates.digits },
    )
    .trim()
    .to_owned()
}

#[cfg(test)]
mod tests;
