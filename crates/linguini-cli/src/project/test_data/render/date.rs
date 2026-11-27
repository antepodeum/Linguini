//! Native previews of ISO strings and numeric timestamps through the shared CLDR planner.

use linguini_cldr::{compiled_calendar_formatting, compiled_date_formatting};
use linguini_codegen::{
    date_request_pattern, decimal_digit_symbols, DateFormatPart, DatePatternPlan, FormatterRequest,
    GregorianWeekYearPlan,
};
use linguini_ir::{IrFormatter, IrFormatterArgument, IrFormatterKind};

use super::{append_localized_digits, RenderError, SampleValue};

pub(super) fn format_date(
    locale: &str,
    value: &str,
    arguments: &[IrFormatterArgument],
) -> Result<String, RenderError> {
    format_date_input(locale, DateInput::Iso(value), arguments)
}

pub(super) fn format_sample_date(
    locale: &str,
    value: &SampleValue,
    arguments: &[IrFormatterArgument],
) -> Result<String, RenderError> {
    match value {
        SampleValue::String(value) => format_date(locale, value, arguments),
        SampleValue::Number(value) => {
            format_date_input(locale, DateInput::Timestamp(value), arguments)
        }
        SampleValue::Boolean(_) => Err(RenderError::InvalidDate {
            value: value.as_text(),
        }),
    }
}

enum DateInput<'a> {
    Iso(&'a str),
    Timestamp(&'a str),
}

fn format_date_input(
    locale: &str,
    value: DateInput<'_>,
    arguments: &[IrFormatterArgument],
) -> Result<String, RenderError> {
    let dates = compiled_date_formatting(locale)
        .ok_or_else(|| super::missing_cldr_data("date formatting", locale))?;
    let calendar = compiled_calendar_formatting(dates.locale, "gregory")
        .ok_or_else(|| super::missing_cldr_data("Gregorian calendar", locale))?;
    let request = FormatterRequest::new(&IrFormatter {
        kind: IrFormatterKind::Date,
        arguments: arguments.to_owned(),
    });
    let pattern = date_request_pattern(&dates, &request).map_err(pattern_error)?;
    let plan = DatePatternPlan::new(locale, &pattern, &calendar).map_err(pattern_error)?;
    let date = match value {
        DateInput::Iso(value) => parse_date(value)?,
        DateInput::Timestamp(value) => input::parse_timestamp(value)?,
    };
    let displayed_minutes = if plan.fields.contains(&'m') {
        date.minute
    } else {
        0
    };
    let exact_period = (!plan.fields.contains(&'s') || date.second == 0)
        && (!plan.fields.contains(&'S') || date.millisecond == 0);
    let period_minute = u16::from(date.hour) * 60 + u16::from(displayed_minutes);
    let digits = decimal_digit_symbols(dates.digits)
        .ok_or_else(|| super::missing_cldr_data("numeric date symbols", locale))?;
    let mut output = String::new();
    for part in &plan.parts {
        match part {
            DateFormatPart::Literal(value) => output.push_str(value),
            DateFormatPart::Field(field) => {
                let symbol = field.symbol;
                let width = field.width;
                let symbols = &field.symbols;
                if let Some(names) = &symbols.names {
                    let index = match symbol {
                        'M' | 'L' => date.month - 1,
                        'Q' | 'q' => (date.month - 1) / 3,
                        _ => date.weekday,
                    };
                    output.push_str(names[usize::from(index)]);
                    continue;
                }
                let numeric = match symbol {
                    'y' => {
                        let year = if date.year <= 0 {
                            1 - date.year
                        } else {
                            date.year
                        };
                        pad(if width == 2 { year % 100 } else { year }, width)
                    }
                    'Y' => {
                        let plan = GregorianWeekYearPlan::for_locale(locale)
                            .ok_or_else(|| super::missing_cldr_data("week rules", locale))?;
                        let year = i64::from(date.year)
                            + i64::from(plan.year_offset(date.month - 1, date.day, date.weekday));
                        let magnitude = year.unsigned_abs();
                        let mut text = pad(
                            if width == 2 {
                                magnitude % 100
                            } else {
                                magnitude
                            },
                            width,
                        );
                        if year < 0 {
                            let numbers = linguini_cldr::compiled_number_formatting(locale)
                                .ok_or_else(|| {
                                    super::missing_cldr_data("numeric date symbols", locale)
                                })?;
                            text.insert_str(0, numbers.minus_sign);
                        }
                        text
                    }
                    'M' | 'L' => pad(date.month, width),
                    'Q' | 'q' => pad((date.month - 1) / 3 + 1, width),
                    'd' => pad(date.day, width),
                    'H' => pad(date.hour, width),
                    'K' => pad(date.hour % 12, width),
                    'h' => pad(
                        if date.hour % 12 == 0 {
                            12
                        } else {
                            date.hour % 12
                        },
                        width,
                    ),
                    'k' => pad(if date.hour == 0 { 24 } else { date.hour }, width),
                    'm' => pad(date.minute, width),
                    's' => pad(date.second, width),
                    'S' => {
                        let mut fraction = pad(date.millisecond, 3);
                        fraction.truncate(width);
                        if width > 3 {
                            fraction.push_str(&"0".repeat(width - 3));
                        }
                        fraction
                    }
                    'G' => {
                        output.push_str(
                            if date.year <= 0 {
                                symbols.bc
                            } else {
                                symbols.ad
                            }
                            .ok_or_else(|| super::missing_cldr_data("era symbols", locale))?,
                        );
                        continue;
                    }
                    'a' | 'b' | 'B' => {
                        let period = field
                            .period
                            .and_then(|index| plan.periods.get(index))
                            .and_then(|plan| {
                                plan.exact
                                    .iter()
                                    .find(|period| exact_period && period.at == period_minute)
                                    .map(|period| period.value)
                                    .or_else(|| {
                                        plan.spans
                                            .iter()
                                            .find(|span| period_minute < span.before)
                                            .map(|span| span.value)
                                    })
                            })
                            .or(if date.hour < 12 {
                                symbols.am
                            } else {
                                symbols.pm
                            })
                            .ok_or_else(|| {
                                super::missing_cldr_data("day-period symbols", locale)
                            })?;
                        output.push_str(period);
                        continue;
                    }
                    _ => {
                        return Err(RenderError::Unsupported {
                            detail: format!("date field `{symbol}` cannot be previewed"),
                        })
                    }
                };
                append_localized_digits(&mut output, &numeric, &digits);
            }
        }
    }
    Ok(output)
}

fn pattern_error(error: impl std::fmt::Display) -> RenderError {
    RenderError::Unsupported {
        detail: error.to_string(),
    }
}

fn pad(value: impl std::fmt::Display, width: usize) -> String {
    format!("{value:0>width$}")
}

mod input;
use input::parse_date;

#[cfg(test)]
mod tests;
