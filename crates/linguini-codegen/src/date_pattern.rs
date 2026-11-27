//! Build-time CLDR date-pattern compilation for the supported Gregorian/UTC fields.

use crate::FormatterRequest;
use linguini_cldr::{compiled_time_zone_formatting, DateFormatData, FormatWidths};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatePatternPart {
    Literal(String),
    Field { symbol: char, width: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatePatternError {
    ConflictingSkeletonStyles,
    MissingSkeleton {
        locale: String,
        skeleton: String,
    },
    UnclosedQuote {
        offset: usize,
    },
    UnsupportedField {
        symbol: char,
        width: usize,
        offset: usize,
    },
    MissingTimeZoneData {
        locale: String,
    },
    MissingWeekData {
        locale: String,
    },
}

impl fmt::Display for DatePatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictingSkeletonStyles => write!(
                f,
                "date skeleton cannot be combined with style or time_style"
            ),
            Self::MissingSkeleton { locale, skeleton } => write!(
                f,
                "no pinned Gregorian date pattern for skeleton {skeleton:?} in {locale:?}"
            ),
            Self::UnclosedQuote { offset } => {
                write!(f, "unclosed date-pattern quote at byte {offset}")
            }
            Self::UnsupportedField {
                symbol,
                width,
                offset,
            } => write!(
                f,
                "unsupported date field `{symbol}` with width {width} at byte {offset}"
            ),
            Self::MissingTimeZoneData { locale } => {
                write!(f, "missing pinned time-zone data for `{locale}`")
            }
            Self::MissingWeekData { locale } => {
                write!(f, "missing pinned week data for `{locale}`")
            }
        }
    }
}

impl std::error::Error for DatePatternError {}

/// Select and compose the exact statically requested CLDR widths.
pub fn date_request_pattern(
    dates: &DateFormatData,
    request: &FormatterRequest,
) -> Result<String, DatePatternError> {
    if let Some(skeleton) = request.argument("skeleton") {
        if request.argument("style").is_some() || request.argument("time_style").is_some() {
            return Err(DatePatternError::ConflictingSkeletonStyles);
        }
        return linguini_cldr::compiled_calendar_formatting(dates.locale, "gregory")
            .and_then(|calendar| {
                calendar
                    .skeletons
                    .iter()
                    .find(|(name, _)| *name == skeleton)
                    .map(|(_, pattern)| (*pattern).to_owned())
            })
            .ok_or_else(|| DatePatternError::MissingSkeleton {
                locale: dates.locale.to_owned(),
                skeleton: skeleton.to_owned(),
            });
    }
    let select = |widths: FormatWidths, style: &str| match style {
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
    Ok(match (date, time) {
        (Some(date), Some(time)) => select(dates.date_time_formats, style.unwrap_or("medium"))
            .replace("{1}", date)
            .replace("{0}", time),
        (Some(part), None) | (None, Some(part)) => part.to_owned(),
        (None, None) => unreachable!("at least a date or time pattern is selected"),
    })
}

/// Parse CLDR quoting and specialize UTC zones to literals before target emission.
/// Unsupported fields fail here instead of becoming a fabricated runtime value.
pub fn compile_date_pattern(
    locale: &str,
    pattern: &str,
) -> Result<Vec<DatePatternPart>, DatePatternError> {
    compile_pattern(locale, pattern, false)
}

pub(crate) fn compile_week_pattern(
    locale: &str,
    pattern: &str,
) -> Result<Vec<DatePatternPart>, DatePatternError> {
    compile_pattern(locale, pattern, true)
}

fn compile_pattern(
    locale: &str,
    pattern: &str,
    week_fields: bool,
) -> Result<Vec<DatePatternPart>, DatePatternError> {
    let mut parts = Vec::new();
    let mut literal = String::new();
    let mut quoted = None;
    let mut chars = pattern.char_indices().peekable();
    while let Some((offset, symbol)) = chars.next() {
        if symbol == '\'' {
            if chars.peek().is_some_and(|(_, next)| *next == '\'') {
                chars.next();
                literal.push('\'');
            } else if quoted.is_some() {
                quoted = None;
            } else {
                quoted = Some(offset);
            }
            continue;
        }
        if quoted.is_some() || !symbol.is_ascii_alphabetic() {
            literal.push(symbol);
            continue;
        }
        let mut width = 1;
        while chars.peek().is_some_and(|(_, next)| *next == symbol) {
            chars.next();
            width += 1;
        }
        let unsupported = || DatePatternError::UnsupportedField {
            symbol,
            width,
            offset,
        };
        let zone = match (symbol, width) {
            ('X', 1..=5) | ('Z', 5) => Some("Z"),
            ('x', 1) => Some("+00"),
            ('x', 2 | 4) | ('Z', 1..=3) => Some("+0000"),
            ('x', 3 | 5) => Some("+00:00"),
            ('z', 1..=4) | ('v' | 'O', 1 | 4) | ('Z', 4) => {
                let data = compiled_time_zone_formatting(locale).ok_or_else(|| {
                    DatePatternError::MissingTimeZoneData {
                        locale: locale.to_owned(),
                    }
                })?;
                Some(match symbol {
                    'z' | 'v' if width == 4 => {
                        data.utc_standard_long.unwrap_or(data.gmt_zero_format)
                    }
                    'z' | 'v' => data.utc_standard_short.unwrap_or(data.gmt_zero_format),
                    _ => data.gmt_zero_format,
                })
            }
            ('z' | 'v' | 'O' | 'Z' | 'X' | 'x', _) => return Err(unsupported()),
            _ => None,
        };
        if let Some(zone) = zone {
            literal.push_str(zone);
            continue;
        }
        let supported = match symbol {
            'y' | 'Y' | 'S' => true,
            'M' | 'L' | 'Q' | 'q' | 'a' | 'b' | 'B' | 'G' => width <= 5,
            'E' => width <= 6,
            'e' | 'c' => (3..=6).contains(&width),
            'd' | 'H' | 'h' | 'K' | 'k' | 'm' | 's' => width <= 2,
            'w' => week_fields && width <= 2,
            'W' => week_fields && width == 1,
            _ => false,
        };
        if !supported {
            return Err(unsupported());
        }
        if matches!(symbol, 'Y' | 'w' | 'W')
            && crate::GregorianWeekYearPlan::for_locale(locale).is_none()
        {
            return Err(DatePatternError::MissingWeekData {
                locale: locale.to_owned(),
            });
        }
        if !literal.is_empty() {
            parts.push(DatePatternPart::Literal(std::mem::take(&mut literal)));
        }
        parts.push(DatePatternPart::Field { symbol, width });
    }
    if let Some(offset) = quoted {
        return Err(DatePatternError::UnclosedQuote { offset });
    }
    if !literal.is_empty() {
        parts.push(DatePatternPart::Literal(literal));
    }
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_skeleton_selection_keeps_locale_patterns_and_rejects_missing_ids() {
        let request = |skeleton: &str| {
            FormatterRequest::new(&linguini_ir::IrFormatter {
                kind: linguini_ir::IrFormatterKind::Date,
                arguments: vec![linguini_ir::IrFormatterArgument {
                    name: "skeleton".to_owned(),
                    value: skeleton.to_owned(),
                }],
            })
        };
        let mut checked = 0;
        let mut quarters = 0;
        let mut locales = std::collections::BTreeSet::new();
        for locale in linguini_cldr::locale_resolution_candidates() {
            let Some(dates) = linguini_cldr::compiled_date_formatting(locale) else {
                continue;
            };
            if !locales.insert(dates.locale) {
                continue;
            }
            let calendar =
                linguini_cldr::compiled_calendar_formatting(dates.locale, "gregory").unwrap();
            for &(skeleton, expected) in calendar.skeletons {
                assert_eq!(
                    date_request_pattern(&dates, &request(skeleton)).unwrap(),
                    expected,
                    "{}: {skeleton}",
                    dates.locale
                );
                checked += 1;
                if skeleton.bytes().all(|byte| byte.is_ascii_alphabetic()) {
                    compile_date_pattern(dates.locale, expected)
                        .unwrap_or_else(|error| panic!("{}: {skeleton}: {error}", dates.locale));
                }
                quarters += usize::from(skeleton.contains('Q'));
            }
            assert!(matches!(
                date_request_pattern(&dates, &request("notASkeleton")),
                Err(DatePatternError::MissingSkeleton { .. })
            ));
        }
        assert_eq!(locales.len(), 508);
        assert!(
            quarters > 900,
            "insufficient quarter skeleton coverage: {quarters}"
        );
        assert!(
            checked > 10_000,
            "insufficient pinned skeleton coverage: {checked}"
        );
        let dates = linguini_cldr::compiled_date_formatting("en").unwrap();
        let mut conflicting = request("yMMMd");
        conflicting
            .formatter
            .arguments
            .push(linguini_ir::IrFormatterArgument {
                name: "style".to_owned(),
                value: "long".to_owned(),
            });
        assert_eq!(
            date_request_pattern(&dates, &conflicting),
            Err(DatePatternError::ConflictingSkeletonStyles)
        );
    }

    #[test]
    fn gregorian_styles_reachable_from_the_pinned_resolution_graph_compile() {
        let mut locales = std::collections::BTreeMap::new();
        for locale in linguini_cldr::locale_resolution_candidates() {
            if let Some(dates) = linguini_cldr::compiled_date_formatting(locale) {
                locales.insert(dates.locale, dates);
            }
        }
        assert_eq!(locales.len(), 508);
        for (locale, dates) in locales {
            let widths = |patterns: FormatWidths| {
                [
                    patterns.full,
                    patterns.long,
                    patterns.medium,
                    patterns.short,
                ]
            };
            for date in widths(dates.date_formats) {
                compile_date_pattern(locale, date)
                    .unwrap_or_else(|error| panic!("{locale}: {date:?}: {error}"));
                for time in widths(dates.time_formats) {
                    compile_date_pattern(locale, time)
                        .unwrap_or_else(|error| panic!("{locale}: {time:?}: {error}"));
                    for composition in widths(dates.date_time_formats) {
                        let pattern = composition.replace("{1}", date).replace("{0}", time);
                        compile_date_pattern(locale, &pattern)
                            .unwrap_or_else(|error| panic!("{locale}: {pattern:?}: {error}"));
                    }
                }
            }
        }
    }

    #[test]
    fn quoting_offsets_and_zone_constants_are_compiled_without_target_syntax() {
        assert_eq!(
            compile_date_pattern("en", "'at' zzzz 'on' y '' 'z'"),
            Ok(vec![
                DatePatternPart::Literal("at Coordinated Universal Time on ".to_owned()),
                DatePatternPart::Field {
                    symbol: 'y',
                    width: 1
                },
                DatePatternPart::Literal(" ' z".to_owned()),
            ])
        );
        assert_eq!(
            compile_date_pattern("!", "Y"),
            Err(DatePatternError::MissingWeekData { locale: "!".into() })
        );
        assert!(compile_date_pattern("en-GB", "Y YY YYYY").is_ok());
        assert_eq!(
            compile_date_pattern("en", "é QQQQQQ"),
            Err(DatePatternError::UnsupportedField {
                symbol: 'Q',
                width: 6,
                offset: 3
            })
        );
        assert_eq!(
            compile_date_pattern("en", "é 'open"),
            Err(DatePatternError::UnclosedQuote { offset: 3 })
        );
        for pattern in [
            "w", "W", "cc", "eeeeeee", "MMMMMM", "zzzzz", "OO", "vv", "XXXXXX",
        ] {
            assert!(
                matches!(
                    compile_date_pattern("en", pattern),
                    Err(DatePatternError::UnsupportedField { .. })
                ),
                "{pattern}"
            );
        }
        assert_eq!(
            compile_date_pattern("ar", "O OOOO ZZZZZ X x xx xxx xxxx xxxxx"),
            Ok(vec![DatePatternPart::Literal(
                "غرينتش غرينتش Z Z +00 +0000 +00:00 +0000 +00:00".to_owned()
            )])
        );
    }
}
