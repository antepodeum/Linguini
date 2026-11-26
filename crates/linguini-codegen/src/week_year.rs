//! Build-time Gregorian week-year boundaries from CLDR territory week conventions.

use linguini_cldr::{compiled_week_data, WeekData};

/// Only a bounded January/December window can change a Gregorian week-year.
/// Weekdays are numbered from Sunday at zero; backends emit these scalar facts directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GregorianWeekYearPlan {
    pub weekday_shift: u8,
    pub anchor_offset: u8,
    pub previous_last_day: Option<u8>,
    pub next_first_day: Option<u8>,
}

impl GregorianWeekYearPlan {
    pub fn for_locale(locale: &str) -> Option<Self> {
        Self::for_week_data(compiled_week_data(locale)?)
    }

    pub fn for_week_data(week: WeekData) -> Option<Self> {
        if week.first_day > 6 || !(1..=7).contains(&week.min_days) {
            return None;
        }
        Some(Self {
            weekday_shift: (7 - week.first_day) % 7,
            anchor_offset: 7 - week.min_days,
            previous_last_day: (week.min_days > 1).then_some(week.min_days - 1),
            next_first_day: (week.min_days < 7).then_some(25 + week.min_days),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn days_before_year(year: i64) -> i64 {
        let previous = year - 1;
        365 * previous + previous.div_euclid(4) - previous.div_euclid(100)
            + previous.div_euclid(400)
    }

    fn first_week_start(year: i64, week: WeekData) -> i64 {
        let weekday = (days_before_year(year) + 1).rem_euclid(7);
        let preceding = (weekday - i64::from(week.first_day)).rem_euclid(7);
        let start = 1 - preceding;
        if 7 - preceding < i64::from(week.min_days) {
            start + 7
        } else {
            start
        }
    }

    #[test]
    fn boundary_windows_match_complete_week_rules_across_a_gregorian_cycle() {
        let years = (1600..2000).chain([-400, -100, -4, -1, 0, 1, 4, 100, 400]);
        for year in years {
            let days = days_before_year(year + 1) - days_before_year(year);
            let january_weekday = (days_before_year(year) + 1).rem_euclid(7);
            for first_day in 0..7 {
                for min_days in 1..=7 {
                    let week = WeekData {
                        first_day,
                        min_days,
                    };
                    let plan = GregorianWeekYearPlan::for_week_data(week).unwrap();
                    let start = first_week_start(year, week);
                    let next_start = days + first_week_start(year + 1, week);
                    for ordinal in (1..=7).chain(days - 6..=days).chain([166]) {
                        let (month, day) = if ordinal <= 7 {
                            (0, ordinal)
                        } else if ordinal > days - 7 {
                            (11, ordinal - (days - 31))
                        } else {
                            (5, ordinal - 151 - (days - 365))
                        };
                        let weekday = (january_weekday + ordinal - 1).rem_euclid(7);
                        let local = (weekday + i64::from(plan.weekday_shift)) % 7;
                        let anchor = day + i64::from(plan.anchor_offset) - local;
                        let actual = if month == 0
                            && plan
                                .previous_last_day
                                .is_some_and(|last| day <= i64::from(last))
                            && anchor < 1
                        {
                            year - 1
                        } else if month == 11
                            && plan
                                .next_first_day
                                .is_some_and(|first| day >= i64::from(first))
                            && anchor > 31
                        {
                            year + 1
                        } else {
                            year
                        };
                        let expected = if ordinal < start {
                            year - 1
                        } else if ordinal >= next_start {
                            year + 1
                        } else {
                            year
                        };
                        assert_eq!(actual, expected, "{year}/{ordinal}: {week:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn locale_facts_prune_impossible_windows_and_reject_invalid_week_data() {
        assert_eq!(
            GregorianWeekYearPlan::for_locale("en-US"),
            Some(GregorianWeekYearPlan {
                weekday_shift: 0,
                anchor_offset: 6,
                previous_last_day: None,
                next_first_day: Some(26),
            })
        );
        assert_eq!(
            GregorianWeekYearPlan::for_locale("en-GB"),
            Some(GregorianWeekYearPlan {
                weekday_shift: 6,
                anchor_offset: 3,
                previous_last_day: Some(3),
                next_first_day: Some(29),
            })
        );
        assert_eq!(
            GregorianWeekYearPlan::for_week_data(WeekData {
                first_day: 0,
                min_days: 7
            })
            .unwrap()
            .next_first_day,
            None
        );
        for week in [
            WeekData {
                first_day: 7,
                min_days: 1,
            },
            WeekData {
                first_day: 0,
                min_days: 0,
            },
            WeekData {
                first_day: 0,
                min_days: 8,
            },
        ] {
            assert_eq!(GregorianWeekYearPlan::for_week_data(week), None);
        }
        assert_eq!(GregorianWeekYearPlan::for_locale("!"), None);
    }
}
