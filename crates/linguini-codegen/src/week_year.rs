//! Gregorian week-year and week-number arithmetic from CLDR territory conventions.

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

    /// Offset from calendar year for validated Gregorian fields. Month and weekday are
    /// zero-based (January and Sunday); day is one-based.
    pub fn year_offset(self, month: u8, day: u8, weekday: u8) -> i8 {
        let january = month == 0 && self.previous_last_day.is_some_and(|last| day <= last);
        let december = month == 11 && self.next_first_day.is_some_and(|first| day >= first);
        if !january && !december {
            return 0;
        }
        let local_weekday = (u16::from(weekday) + u16::from(self.weekday_shift)) % 7;
        let anchor = i32::from(day) + i32::from(self.anchor_offset) - i32::from(local_weekday);
        if january && anchor < 1 {
            -1
        } else if december && anchor > 31 {
            1
        } else {
            0
        }
    }

    /// Gregorian week number (1–53) for validated fields, using the same convention as
    /// [`Self::year_offset`]. Month and weekday are zero-based; day is one-based.
    pub fn week_number(self, year: i32, month: u8, day: u8, weekday: u8) -> u8 {
        let leap = |year: i64| year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let year = i64::from(year);
        let ordinal = (275 * i32::from(month) + 5) / 9 + i32::from(day)
            - if month > 1 {
                if leap(year) {
                    1
                } else {
                    2
                }
            } else {
                0
            };
        let local_weekday = (u16::from(weekday) + u16::from(self.weekday_shift)) % 7;
        let mut anchor = ordinal + i32::from(self.anchor_offset) - i32::from(local_weekday);
        if anchor < 1 {
            anchor += if leap(year - 1) { 366 } else { 365 };
        } else {
            let days = if leap(year) { 366 } else { 365 };
            if anchor > days {
                anchor -= days;
            }
        }
        ((anchor - 1) / 7 + 1) as u8
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
                        let actual =
                            year + i64::from(plan.year_offset(month, day as u8, weekday as u8));
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
    fn week_numbers_match_complete_week_rules_for_every_day_and_convention() {
        let years = (1600..2000).chain([
            -271_820,
            -400,
            -100,
            -4,
            -1,
            0,
            1,
            4,
            100,
            400,
            275_000,
            i64::from(i32::MIN),
            i64::from(i32::MAX),
        ]);
        for year in years {
            let days = days_before_year(year + 1) - days_before_year(year);
            let previous_days = days_before_year(year) - days_before_year(year - 1);
            let january_weekday = (days_before_year(year) + 1).rem_euclid(7);
            for first_day in 0..7 {
                for min_days in 1..=7 {
                    let week = WeekData {
                        first_day,
                        min_days,
                    };
                    let plan = GregorianWeekYearPlan::for_week_data(week).unwrap();
                    let start = first_week_start(year, week);
                    let previous_start = -previous_days + first_week_start(year - 1, week);
                    let next_start = days + first_week_start(year + 1, week);
                    let mut ordinal = 0;
                    for (month, count) in [
                        31,
                        28 + (days - 365),
                        31,
                        30,
                        31,
                        30,
                        31,
                        31,
                        30,
                        31,
                        30,
                        31,
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        for day in 1..=count {
                            ordinal += 1;
                            let weekday = (january_weekday + ordinal - 1).rem_euclid(7);
                            let (expected_year, first) = if ordinal < start {
                                (year - 1, previous_start)
                            } else if ordinal >= next_start {
                                (year + 1, next_start)
                            } else {
                                (year, start)
                            };
                            let actual_year = year
                                + i64::from(plan.year_offset(
                                    month as u8,
                                    day as u8,
                                    weekday as u8,
                                ));
                            let number = plan.week_number(
                                year as i32,
                                month as u8,
                                day as u8,
                                weekday as u8,
                            );
                            assert_eq!(
                                (actual_year, i64::from(number)),
                                (expected_year, (ordinal - first) / 7 + 1),
                                "{year}/{month}/{day}: {week:?}"
                            );
                        }
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
