//! Locale formatter lowering and demand-selected runtime assembly.

mod week;

#[cfg(test)]
use linguini_cldr::NumberPattern;
use linguini_cldr::{compiled_date_formatting, compiled_number_formatting, NumberPaddingPosition};

use super::formatter_helpers::{
    date_formatter_helpers, digit_formatter_helper, digit_lookup, week_year_formatter_helper,
    DigitBindings,
};

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
    let mut digit_bindings = DigitBindings::new(needs_date.then(|| {
        compiled_date_formatting(locale)
            .expect("validated date locale")
            .digits
    }));
    let mut output = crate::templates::render("formatters/types", context! { needs_number });
    let mut plans = Vec::new();
    let mut date_helpers = DateHelperRequirements::default();
    let mut date_tables = DateTables::new();
    for request in requests {
        let name = super::formatters::formatter_binding(request);
        match request.formatter.kind {
            linguini_ir::IrFormatterKind::Number | linguini_ir::IrFormatterKind::Currency => {
                let plan = if request.formatter.kind == linguini_ir::IrFormatterKind::Currency {
                    let selected = linguini_codegen::NumberFormatPlan::currency_with_display(
                        locale,
                        request.argument("code").unwrap_or("USD"),
                        request.argument("accounting") == Some("true"),
                        request.argument("display").unwrap_or("symbol"),
                    )
                    .expect("validated CLDR currency formatting plan");
                    number_plan_context(&selected, &mut digit_bindings)
                } else {
                    let selected = linguini_codegen::NumberFormatPlan::number(
                        locale,
                        request.argument("style").unwrap_or("decimal"),
                    )
                    .expect("validated CLDR number formatting plan");
                    number_plan_context(&selected, &mut digit_bindings)
                };
                plans.push(plan.clone());
                output.push_str(&crate::templates::render(
                    "formatters/number",
                    context! { name, plan, exported => export_helpers },
                ));
            }
            linguini_ir::IrFormatterKind::Date => {
                let dates = compiled_date_formatting(locale).expect("validated date locale");
                let (source, helpers) = generated_date_request(
                    locale,
                    &dates,
                    request,
                    &name,
                    export_helpers,
                    &mut date_tables,
                );
                output.push_str(&source);
                date_helpers.padding |= helpers.padding;
                date_helpers.digits |= helpers.digits;
                date_helpers.week_year |= helpers.week_year;
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
    if date_helpers.week_year {
        output.push_str(&week_year_formatter_helper(locale));
    }
    if date_helpers.digits {
        let dates = compiled_date_formatting(locale).expect("validated date locale");
        if !plans
            .iter()
            .any(|plan| plan.get_attr("digits").unwrap() == minijinja::Value::from(dates.digits))
        {
            output.push_str(&digit_formatter_helper(dates.digits));
        }
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
    let precision = precision::PrecisionHelpers::for_plans(plans, MAX_NUMBER_DECIMAL_DIGITS);
    let mut output = crate::templates::render(
        "number_helpers",
        context! {
            rounding_plans => unique("rounding_helper", &|plan| !present(plan, "max_significant") && plan.get_attr("unlimited_precision").unwrap() != minijinja::Value::from(true)),
            scale_plans => unique("scale", &|plan| plan.get_attr("scale").unwrap() != minijinja::Value::from(1)),
            significant_plans => precision.rounding,
            padding_plans => precision.padding,
            significant_fraction_rounding => precision.variable_rounding,
            significant_fraction_bounded => precision.fraction_bounded,
            max_decimal_digits => MAX_NUMBER_DECIMAL_DIGITS,
            scientific_plans => unique("scientific_helper", &|plan| present(plan, "exponent_digits")),
            scientific_shift => plans.iter().any(|plan| present(plan, "exponent_digits") && !present(plan, "carry_fraction") && plan.get_attr("unlimited_precision").unwrap() != minijinja::Value::from(true)),
        },
    );
    for plan in unique("digit_helper", &|plan| present(plan, "digit_helper")) {
        output.push_str(&crate::templates::render(
            "digit_helpers",
            context! { name => plan.get_attr("digit_helper").unwrap(), lookup => plan.get_attr("digit_lookup").unwrap() },
        ));
    }
    output
}

const MAX_NUMBER_DECIMAL_DIGITS: usize = 8192;

mod precision;

#[cfg(test)]
fn number_plan(
    pattern: &NumberPattern,
    symbol: Option<&str>,
    numbers: &linguini_cldr::NumberFormatData,
    rounding: Option<&str>,
    digit_bindings: &mut DigitBindings,
) -> minijinja::Value {
    let plan = linguini_codegen::NumberFormatPlan::new(pattern, symbol, numbers, rounding)
        .expect("validated CLDR number formatting plan");
    number_plan_context(&plan, digit_bindings)
}

fn number_plan_context(
    plan: &linguini_codegen::NumberFormatPlan,
    digit_bindings: &mut DigitBindings,
) -> minijinja::Value {
    use minijinja::context;
    let rounding = &plan.rounding;
    let grouping = plan.grouping;
    let scientific = plan.scientific.as_ref();
    let significant = match plan.precision() {
        linguini_codegen::NumberPrecisionPlan::Significant { minimum, maximum } => {
            Some((minimum, maximum))
        }
        _ => None,
    };
    let scientific_helper = scientific.map(|scientific| {
        if scientific.exponent_step > 1 {
            format!("engineeringWith{}IntegerDigits", scientific.exponent_step)
        } else if scientific.integer_digits != 1 {
            format!("scientificWith{}IntegerDigits", scientific.integer_digits)
        } else {
            "scientificGeneratedDecimal".to_owned()
        }
    });
    // Significant rounding already pads nonzero mantissas. Zero alone needs a
    // literal suffix; engineering carries may shorten the remaining fraction.
    let fixed_significant_zero = scientific
        .and(significant)
        .filter(|(minimum, maximum)| {
            minimum == maximum
                && *maximum > 0
                && plan.minimum_significant_digits() == Some(*minimum)
        })
        .map(|(_, maximum)| "0".repeat(usize::from(maximum) - 1));
    let padding = number_padding_context(plan);
    let rounding_helper = if rounding.increment == "1" {
        format!("roundTo{}FractionDigits", rounding.fraction_digits)
    } else {
        format!(
            "roundTo{}FractionDigitsIncrement{}",
            rounding.fraction_digits, rounding.increment
        )
    };
    context! {
        prefix => &plan.affixes.prefix, suffix => &plan.affixes.suffix,
        negative_prefix => &plan.affixes.negative_prefix, negative_suffix => &plan.affixes.negative_suffix,
        min_integer => plan.min_integer_digits, min_fraction => plan.minimum_fraction_digits(),
        omit_integer_zero => plan.can_omit_integer_zero(),
        max_fraction => plan.max_fraction_digits, primary_group => grouping.map(|group| group.primary_size),
        grouping_threshold => grouping.map(|group| group.threshold), secondary_group => grouping.map(|group| group.secondary_size),
        decimal_symbol => plan.decimal_symbol, group_symbol => plan.group_symbol, digits => plan.digits, digit_lookup => digit_lookup(plan.digits),
        decimal_separator_always => plan.decimal_separator_always,
        digit_helper => digit_bindings.binding(plan.digits),
        scale => plan.scale, scale_shift => plan.scale_shift,
        min_significant => plan.minimum_significant_digits(),
        max_significant => significant.map(|(_, maximum)| maximum),
        fixed_significant_zero,
        rounding_helper, rounding => &rounding.increment, rounding_half => &rounding.half_integer,
        rounding_half_fraction => rounding.half_fraction,
        exponent_digits => scientific.map(|scientific| scientific.exponent_digits),
        exponential_symbol => scientific.map(|scientific| scientific.separator),
        minus_sign => scientific.map(|scientific| scientific.negative_sign),
        exponent_plus => scientific.map(|scientific| scientific.positive_sign),
        carry_fraction => scientific.and_then(|scientific| scientific.carry_fraction.as_deref()), padding,
        scientific_helper,
        scientific_integer_digits => scientific.map_or(1, |scientific| scientific.integer_digits),
        scientific_exponent_step => scientific.map_or(1, |scientific| scientific.exponent_step),
        scientific_carry_width => scientific.map_or(1, |scientific| scientific.integer_digits.max(scientific.exponent_step)),
        scientific_carry_integer => scientific.map(|scientific| if scientific.integer_digits == 0 { "0".to_owned() } else { "1".to_owned() + &"0".repeat(usize::from(scientific.integer_digits) - 1) }),
        unlimited_precision => plan.precision() == linguini_codegen::NumberPrecisionPlan::Unlimited,
    }
}

fn number_padding_context(plan: &linguini_codegen::NumberFormatPlan) -> Option<minijinja::Value> {
    use minijinja::context;
    let padding = plan.padding.as_ref()?;
    let exponent_sign_width = plan.scientific.as_ref().and_then(|scientific| {
        let positive = scientific.positive_sign.chars().count();
        let negative = scientific.negative_sign.chars().count();
        (positive != negative).then(|| context! { positive, negative })
    });
    let scientific_width = plan.scientific.as_ref().map_or(0, |scientific| {
        scientific.separator.chars().count()
            + if exponent_sign_width.is_none() {
                scientific.positive_sign.chars().count()
            } else {
                0
            }
    });
    let available_width = |prefix: &str, suffix: &str| {
        usize::from(padding.width)
            .saturating_sub(prefix.chars().count() + suffix.chars().count() + scientific_width)
    };
    let positive_width = available_width(&plan.affixes.prefix, &plan.affixes.suffix);
    let negative_width =
        available_width(&plan.affixes.negative_prefix, &plan.affixes.negative_suffix);
    if positive_width == 0 && negative_width == 0 {
        return None;
    }
    // Each digit substitution replaces one scalar with one scalar. Measure the body
    // before substitution and correct only the selected separators' UTF-16 widths.
    let surrogates = |text: &str| text.encode_utf16().count() - text.chars().count();
    Some(context! {
        character => padding.character.to_string(), positive_width, negative_width, exponent_sign_width,
        decimal_surrogates => surrogates(plan.decimal_symbol), group_surrogates => surrogates(plan.group_symbol),
        position => match padding.position { NumberPaddingPosition::BeforePrefix => 0, NumberPaddingPosition::AfterPrefix => 1, NumberPaddingPosition::BeforeSuffix => 2, NumberPaddingPosition::AfterSuffix => 3 },
    })
}

#[derive(Default)]
struct DateHelperRequirements {
    week_year: bool,
    padding: bool,
    digits: bool,
}

type DateTables = std::collections::BTreeMap<String, Vec<&'static str>>;

fn generated_date_request(
    locale: &str,
    dates: &linguini_cldr::DateFormatData,
    request: &super::formatters::FormatterRequest,
    name: &str,
    exported: bool,
    shared_tables: &mut DateTables,
) -> (String, DateHelperRequirements) {
    if matches!(request.argument("skeleton"), Some("yw" | "MMMMW")) {
        return week::generate(locale, dates, request, name, exported, shared_tables);
    }

    let pattern = linguini_codegen::date_request_pattern(dates, request)
        .expect("validated CLDR date request");
    let pattern = date_parts(&pattern, dates, name, locale, shared_tables);
    (
        crate::templates::render(
            "formatters/date",
            minijinja::context! { name, exported, parts => pattern.parts, tables => pattern.tables, periods => pattern.periods, clock => pattern.clock, digits => dates.digits, localize_result => pattern.localize_result },
        ),
        pattern.helpers,
    )
}

struct CompiledDatePattern {
    parts: Vec<minijinja::Value>,
    tables: Vec<minijinja::Value>,
    periods: Vec<minijinja::Value>,
    helpers: DateHelperRequirements,
    clock: minijinja::Value,
    localize_result: bool,
}

fn date_parts(
    pattern: &str,
    dates: &linguini_cldr::DateFormatData,
    name: &str,
    requested_locale: &str,
    shared_tables: &mut DateTables,
) -> CompiledDatePattern {
    let calendar = linguini_cldr::compiled_calendar_formatting(dates.locale, "gregory")
        .expect("validated locale has pinned Gregorian symbols");
    let plan = linguini_codegen::DatePatternPlan::new(requested_locale, pattern, &calendar)
        .expect("validated date pattern and symbols");
    project_date_parts(&plan, dates, name, requested_locale, shared_tables)
}

fn project_date_parts(
    plan: &linguini_codegen::DatePatternPlan,
    dates: &linguini_cldr::DateFormatData,
    name: &str,
    requested_locale: &str,
    shared_tables: &mut DateTables,
) -> CompiledDatePattern {
    use minijinja::context;
    let mut helpers = DateHelperRequirements::default();
    let mut parts = Vec::new();
    let mut tables = std::collections::BTreeMap::<String, Vec<&str>>::new();
    let fields = &plan.fields;
    let mut has_periods = false;
    let mut exact_periods = false;
    let mut total_minutes = false;
    let mut exact_count = 0;
    let period_binding = |index| {
        if index == 0 {
            "dayPeriod".to_owned()
        } else {
            format!("dayPeriod{}", index + 1)
        }
    };
    let periods = plan.periods.iter().enumerate().map(|(index, plan)| {
        let binding = period_binding(index);
        has_periods |= !plan.exact.is_empty() || plan.spans.len() > 1;
        exact_periods |= !plan.exact.is_empty();
        exact_count += plan.exact.len();
        let minute_clock = plan.spans.iter().any(|span| span.before % 60 != 0);
        total_minutes |= minute_clock;
        let spans: Vec<_> = plan.spans.iter().map(|span| context! {
            before => if minute_clock { span.before } else { span.before / 60 }, value => span.value,
        }).collect();
        let exact: Vec<_> = plan.exact.iter().map(|period| context! {
            hour => period.at / 60, minute => period.at % 60, value => period.value,
        }).collect();
        context! { binding, spans, exact, minute_clock }
    }).collect();
    for part in &plan.parts {
        let field = match part {
            linguini_codegen::DateFormatPart::Literal(value) => {
                parts.push(context! { kind => "literal", value });
                continue;
            }
            linguini_codegen::DateFormatPart::Field(field) => field,
        };
        let character = field.symbol;
        let width = field.width;
        let period = field
            .period
            .map(|index| context! { binding => period_binding(index) });
        let mut table = None;
        let symbols = &field.symbols;
        if let Some(values) = &symbols.names {
            let proposed = format!(
                "{name}_{}{}_{width}",
                if matches!(character, 'L' | 'c' | 'q') {
                    "standalone_"
                } else {
                    ""
                },
                match character {
                    'M' | 'L' => "months",
                    'Q' | 'q' => "quarters",
                    _ => "weekdays",
                }
            );
            let binding = shared_tables
                .iter()
                .find(|(_, existing)| existing.as_slice() == values.as_slice())
                .map(|(binding, _)| binding.clone())
                .unwrap_or(proposed);
            if !shared_tables.contains_key(&binding) {
                shared_tables.insert(binding.clone(), values.clone());
                tables.insert(binding.clone(), values.clone());
            }
            table = Some(binding);
        }
        let numeric = matches!(
            character,
            'y' | 'Y' | 'w' | 'W' | 'd' | 'H' | 'h' | 'K' | 'k' | 'm' | 's' | 'S'
        ) || matches!(character, 'M' | 'L' | 'Q' | 'q') && width <= 2;
        helpers.padding |= character == 'S'
            || (numeric && width == 2)
            || (matches!(character, 'y' | 'Y') && width > 2);
        helpers.digits |= numeric && dates.digits != "0123456789";
        parts.push(context! { kind => "field", field => character.to_string(), width, table, numeric, period, am => symbols.am, pm => symbols.pm, ad => symbols.ad, bc => symbols.bc });
    }
    helpers.week_year = fields.contains(&'Y');
    let minus_sign = helpers.week_year.then(|| {
        compiled_number_formatting(requested_locale)
            .expect("validated numeric date symbols")
            .minus_sign
    });
    let signed_week_year = minus_sign.is_some_and(|sign| sign != "-")
        || plan.parts.iter().any(|part| matches!(part, linguini_codegen::DateFormatPart::Field(field) if field.symbol == 'Y' && field.width > 1));
    CompiledDatePattern {
        localize_result: helpers.digits && plan.can_localize_whole_result,
        parts,
        tables: tables
            .into_iter()
            .map(|(name, values)| context! { name, values })
            .collect(),
        periods,
        helpers,
        clock: context! {
            exact_time => exact_count > 1 && (fields.contains(&'m') || fields.contains(&'s') || fields.contains(&'S')),
            year => fields.contains(&'y'),
            week_year => fields.contains(&'Y'),
            month_week => fields.contains(&'W'),
            minus_sign,
            signed_week_year,
            era => fields.contains(&'G'),
            hours => has_periods,
            minutes => (exact_periods || total_minutes) && fields.contains(&'m'),
            seconds => exact_periods && fields.contains(&'s'),
            milliseconds => exact_periods && fields.contains(&'S'),
            total_minutes,
        },
    }
}

#[cfg(test)]
mod tests;
