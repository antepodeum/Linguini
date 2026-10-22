//! Resolve fixed and flexible day-period rules and symbol fallbacks during builds.

use linguini_cldr::{compiled_day_period_rules, CalendarFormatData};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayPeriodSpan {
    /// Exclusive upper bound in minutes from midnight; spans cover the whole day in order.
    pub before: u16,
    pub value: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactDayPeriod {
    pub at: u16,
    pub value: &'static str,
}

/// Resolved symbols and disjoint intervals, without CLDR names, lookup paths, or target syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayPeriodPlan {
    pub exact: Vec<ExactDayPeriod>,
    pub spans: Vec<DayPeriodSpan>,
}

/// Compile `b` (fixed periods) or `B` (flexible periods) from the pinned calendar and rules.
/// Missing flexible labels fall back to AM/PM; missing fixed labels use the general period.
/// Consumers compare exact periods against the time actually displayed by their pattern.
pub fn day_period_plan(
    calendar: &CalendarFormatData,
    field: char,
    width: usize,
) -> Option<DayPeriodPlan> {
    if !matches!(field, 'b' | 'B') {
        return None;
    }
    let width = match width {
        4 => "wide",
        5 => "narrow",
        _ => "abbreviated",
    };
    let symbol = |name| calendar.symbol(&format!("dayPeriods.format.{width}.{name}"));
    let am = symbol("am")?;
    let pm = symbol("pm")?;
    let rules = compiled_day_period_rules(calendar.locale).unwrap_or_default();
    let ranges: Vec<_> = rules
        .iter()
        .filter(|_| field == 'B')
        .filter_map(|rule| Some((rule.from?, rule.before?, symbol(rule.name)?)))
        .collect();
    let mut boundaries = vec![0, 720, 1440];
    for (from, before, _) in &ranges {
        boundaries.extend([*from, *before]);
    }
    boundaries.sort_unstable();
    boundaries.dedup();
    let mut spans: Vec<DayPeriodSpan> = Vec::new();
    for window in boundaries.windows(2) {
        let minute = window[0];
        let value = ranges
            .iter()
            .find(|(from, before, _)| {
                if from < before {
                    *from <= minute && minute < *before
                } else {
                    minute >= *from || minute < *before
                }
            })
            .map_or(if minute < 720 { am } else { pm }, |(_, _, value)| *value);
        if let Some(previous) = spans.last_mut().filter(|span| span.value == value) {
            previous.before = window[1];
        } else {
            spans.push(DayPeriodSpan {
                before: window[1],
                value,
            });
        }
    }
    let mut exact: Vec<_> = rules
        .iter()
        .filter(|rule| matches!(rule.name, "midnight" | "noon"))
        .filter_map(|rule| {
            Some(ExactDayPeriod {
                at: rule.at?,
                value: symbol(rule.name)?,
            })
        })
        .filter(|exact| {
            spans
                .iter()
                .find(|span| exact.at < span.before)
                .is_some_and(|span| span.value != exact.value)
        })
        .collect();
    exact.sort_by_key(|period| period.at);
    Some(DayPeriodPlan { exact, spans })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_and_flexible_periods_resolve_different_symbols_and_intervals() {
        let calendar = linguini_cldr::compiled_calendar_formatting("en", "gregory").unwrap();
        let fixed = day_period_plan(&calendar, 'b', 4).unwrap();
        assert_eq!(
            fixed.spans,
            vec![
                DayPeriodSpan {
                    before: 720,
                    value: "AM"
                },
                DayPeriodSpan {
                    before: 1440,
                    value: "PM"
                },
            ]
        );
        assert_eq!(
            fixed.exact,
            vec![
                ExactDayPeriod {
                    at: 0,
                    value: "midnight"
                },
                ExactDayPeriod {
                    at: 720,
                    value: "noon"
                },
            ]
        );
        let flexible = day_period_plan(&calendar, 'B', 4).unwrap();
        assert_eq!(
            flexible.spans,
            vec![
                DayPeriodSpan {
                    before: 720,
                    value: "in the morning"
                },
                DayPeriodSpan {
                    before: 1080,
                    value: "in the afternoon"
                },
                DayPeriodSpan {
                    before: 1260,
                    value: "in the evening"
                },
                DayPeriodSpan {
                    before: 1440,
                    value: "at night"
                },
            ]
        );
        assert_eq!(flexible.exact, fixed.exact);
        assert!(day_period_plan(&calendar, 'a', 1).is_none());
        assert_eq!(
            day_period_plan(&calendar, 'b', 5).unwrap().exact[1].value,
            "n"
        );
    }

    #[test]
    fn missing_period_labels_fall_back_and_adjacent_equal_spans_merge() {
        let mut calendar = linguini_cldr::compiled_calendar_formatting("en", "gregory").unwrap();
        calendar.symbols = &[&[
            ("dayPeriods.format.wide.am", "am"),
            ("dayPeriods.format.wide.pm", "pm"),
        ]];
        let fixed = day_period_plan(&calendar, 'b', 4).unwrap();
        let flexible = day_period_plan(&calendar, 'B', 4).unwrap();
        assert_eq!(fixed, flexible);
        assert!(fixed.exact.is_empty());
        assert_eq!(
            fixed.spans,
            vec![
                DayPeriodSpan {
                    before: 720,
                    value: "am"
                },
                DayPeriodSpan {
                    before: 1440,
                    value: "pm"
                },
            ]
        );
        calendar.symbols = &[&[
            ("dayPeriods.format.wide.am", "day"),
            ("dayPeriods.format.wide.pm", "day"),
            ("dayPeriods.format.wide.noon", "day"),
        ]];
        let constant = day_period_plan(&calendar, 'B', 4).unwrap();
        assert!(constant.exact.is_empty());
        assert_eq!(
            constant.spans,
            vec![DayPeriodSpan {
                before: 1440,
                value: "day"
            }]
        );
    }
}
