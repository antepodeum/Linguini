//! Native previews of date-only sample values through the shared CLDR request planner.

use linguini_cldr::{compiled_calendar_formatting, compiled_date_formatting};
use linguini_codegen::{
    compile_date_pattern, date_field_symbols, date_request_pattern, day_period_plan,
    decimal_digit_symbols, DatePatternPart, FormatterRequest, GregorianWeekYearPlan,
};
use linguini_ir::{IrFormatter, IrFormatterArgument, IrFormatterKind};

use super::RenderError;

pub(super) fn format_date(
    locale: &str,
    value: &str,
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
    let parts = compile_date_pattern(locale, &pattern).map_err(pattern_error)?;
    let date = parse_date(value)?;
    let digits = decimal_digit_symbols(dates.digits)
        .ok_or_else(|| super::missing_cldr_data("numeric date symbols", locale))?;
    let mut output = String::new();
    for part in parts {
        match part {
            DatePatternPart::Literal(value) => output.push_str(&value),
            DatePatternPart::Field { symbol, width } => {
                let symbols = date_field_symbols(&calendar, symbol, width)
                    .ok_or_else(|| super::missing_cldr_data("date symbols", locale))?;
                if let Some(names) = symbols.names {
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
                        let year = if date.year == 0 { 1 } else { date.year };
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
                    // Date-only sample inputs are UTC midnight, including time-only requests.
                    'H' | 'K' | 'm' | 's' => pad(0, width),
                    'h' => pad(12, width),
                    'k' => pad(24, width),
                    'S' => "0".repeat(width),
                    'G' => {
                        output.push_str(
                            if date.year == 0 {
                                symbols.bc
                            } else {
                                symbols.ad
                            }
                            .ok_or_else(|| super::missing_cldr_data("era symbols", locale))?,
                        );
                        continue;
                    }
                    'a' | 'b' | 'B' => {
                        let period = day_period_plan(&calendar, symbol, width)
                            .and_then(|plan| {
                                plan.exact
                                    .iter()
                                    .find(|period| period.at == 0)
                                    .map(|period| period.value)
                                    .or_else(|| plan.spans.first().map(|span| span.value))
                            })
                            .or(symbols.am)
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
                for character in numeric.chars() {
                    output.push(if character.is_ascii_digit() {
                        digits[usize::from(character as u8 - b'0')]
                    } else {
                        character
                    });
                }
            }
        }
    }
    Ok(output)
}

fn pattern_error(error: linguini_codegen::DatePatternError) -> RenderError {
    RenderError::Unsupported {
        detail: error.to_string(),
    }
}

fn pad(value: impl std::fmt::Display, width: usize) -> String {
    format!("{value:0>width$}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DateParts {
    year: u32,
    month: u8,
    day: u8,
    weekday: u8,
}

fn parse_date(value: &str) -> Result<DateParts, RenderError> {
    let invalid = || RenderError::InvalidDate {
        value: value.to_owned(),
    };
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes[..4]
            .iter()
            .chain(&bytes[5..7])
            .chain(&bytes[8..])
            .all(u8::is_ascii_digit)
    {
        return Err(invalid());
    }
    let year = value[..4].parse::<u32>().map_err(|_| invalid())?;
    let month = value[5..7].parse::<u8>().map_err(|_| invalid())?;
    let day = value[8..].parse::<u8>().map_err(|_| invalid())?;
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        _ => 0,
    };
    if day == 0 || day > max_day {
        return Err(invalid());
    }
    const OFFSETS: [i64; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let adjusted_year = i64::from(year) - i64::from(month < 3);
    let weekday = (adjusted_year + adjusted_year.div_euclid(4) - adjusted_year.div_euclid(100)
        + adjusted_year.div_euclid(400)
        + OFFSETS[usize::from(month - 1)]
        + i64::from(day))
    .rem_euclid(7) as u8;
    Ok(DateParts {
        year,
        month,
        day,
        weekday,
    })
}

#[cfg(test)]
mod tests;
