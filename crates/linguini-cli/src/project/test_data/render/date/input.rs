//! Parse the generated date runtime's ISO strings and numeric UTC millisecond inputs.

use super::RenderError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DateParts {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub weekday: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
}

pub(super) fn parse_date(value: &str) -> Result<DateParts, RenderError> {
    parse_iso(value).ok_or_else(|| RenderError::InvalidDate {
        value: value.to_owned(),
    })
}

pub(super) fn parse_timestamp(value: &str) -> Result<DateParts, RenderError> {
    timestamp_parts(value).ok_or_else(|| RenderError::InvalidDate {
        value: value.to_owned(),
    })
}

fn timestamp_parts(value: &str) -> Option<DateParts> {
    const MILLISECONDS_PER_DAY: i64 = 86_400_000;
    const MAX_TIMESTAMP: f64 = 8_640_000_000_000_000.0;
    let milliseconds = value.parse::<f64>().ok()?;
    if !milliseconds.is_finite() || milliseconds.abs() > MAX_TIMESTAMP {
        return None;
    }
    // ECMAScript TimeClip truncates toward zero before splitting negative UTC days.
    let milliseconds = milliseconds.trunc() as i64;
    let days = milliseconds.div_euclid(MILLISECONDS_PER_DAY);
    let time = milliseconds.rem_euclid(MILLISECONDS_PER_DAY);
    let ordinal = days + days_before_year(1970);
    // TimeClip's endpoints lie inside these bounds. Find the containing Gregorian year
    // without relying on a host timezone or overflowing a native calendar library.
    let (mut lower, mut upper) = (-300_000, 300_001);
    while lower + 1 < upper {
        let middle = lower + (upper - lower) / 2;
        if days_before_year(middle) <= ordinal {
            lower = middle;
        } else {
            upper = middle;
        }
    }
    let year = lower;
    let mut day = ordinal - days_before_year(year);
    let mut month = 1;
    while day >= i64::from(days_in_month(year, month)) {
        day -= i64::from(days_in_month(year, month));
        month += 1;
    }
    Some(DateParts {
        year,
        month,
        day: (day + 1) as u8,
        weekday: (days + 4).rem_euclid(7) as u8,
        hour: (time / 3_600_000) as u8,
        minute: (time / 60_000 % 60) as u8,
        second: (time / 1_000 % 60) as u8,
        millisecond: (time % 1_000) as u16,
    })
}

fn days_before_year(year: i32) -> i64 {
    let year = i64::from(year);
    year * 365 + (year + 3).div_euclid(4) - (year + 99).div_euclid(100)
        + (year + 399).div_euclid(400)
}

fn parse_iso(value: &str) -> Option<DateParts> {
    let bytes = value.as_bytes();
    if !value.is_ascii() || bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return None;
    }
    let mut year = i32::from(digits(bytes.get(..4)?)?);
    let mut month = u8::try_from(digits(bytes.get(5..7)?)?).ok()?;
    let mut day = i32::from(digits(bytes.get(8..10)?)?);
    if !(1..=12).contains(&month) || !(1..=days_in_month(year, month)).contains(&day) {
        return None;
    }
    let mut hour = 0;
    let mut minute = 0;
    let mut second = 0;
    let mut millisecond = 0;
    let mut offset = 0;
    if bytes.len() > 10 {
        if bytes[10] != b'T' || bytes.get(13) != Some(&b':') {
            return None;
        }
        hour = i32::from(digits(bytes.get(11..13)?)?);
        minute = i32::from(digits(bytes.get(14..16)?)?);
        let mut cursor = 16;
        let mut nonzero_fraction = false;
        if bytes.get(cursor) == Some(&b':') {
            second = u8::try_from(digits(bytes.get(cursor + 1..cursor + 3)?)?).ok()?;
            cursor += 3;
            if bytes.get(cursor) == Some(&b'.') {
                cursor += 1;
                let start = cursor;
                while let Some(digit) = bytes.get(cursor).filter(|digit| digit.is_ascii_digit()) {
                    nonzero_fraction |= *digit != b'0';
                    if cursor - start < 3 {
                        millisecond +=
                            u16::from(*digit - b'0') * 10_u16.pow(2 - (cursor - start) as u32);
                    }
                    cursor += 1;
                }
                if start == cursor {
                    return None;
                }
            }
        }
        match bytes.get(cursor..) {
            Some([] | [b'Z']) => {}
            Some([sign @ (b'+' | b'-'), h1, h2, b':', m1, m2]) => {
                let hours = digits(&[*h1, *h2])?;
                let minutes = digits(&[*m1, *m2])?;
                if hours > 23 || minutes > 59 {
                    return None;
                }
                offset = i32::from(hours * 60 + minutes) * if *sign == b'-' { -1 } else { 1 };
            }
            _ => return None,
        }
        if hour > 24
            || minute > 59
            || second > 59
            || (hour == 24 && (minute != 0 || second != 0 || nonzero_fraction))
        {
            return None;
        }
    }
    let minutes = hour * 60 + minute - offset;
    day += minutes.div_euclid(1440);
    if day < 1 {
        if month == 1 {
            year -= 1;
            month = 12;
        } else {
            month -= 1;
        }
        day += days_in_month(year, month);
    } else if day > days_in_month(year, month) {
        day -= days_in_month(year, month);
        if month == 12 {
            year += 1;
            month = 1;
        } else {
            month += 1;
        }
    }
    const OFFSETS: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let adjusted_year = year - i32::from(month < 3);
    let weekday = (adjusted_year + adjusted_year.div_euclid(4) - adjusted_year.div_euclid(100)
        + adjusted_year.div_euclid(400)
        + OFFSETS[usize::from(month - 1)]
        + day)
        .rem_euclid(7) as u8;
    let minutes = minutes.rem_euclid(1440);
    Some(DateParts {
        year,
        month,
        day: day as u8,
        weekday,
        hour: (minutes / 60) as u8,
        minute: (minutes % 60) as u8,
        second,
        millisecond,
    })
}

fn digits(bytes: &[u8]) -> Option<u16> {
    bytes.iter().try_fold(0, |value, byte| {
        byte.is_ascii_digit()
            .then(|| value * 10 + u16::from(byte - b'0'))
    })
}

fn days_in_month(year: i32, month: u8) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 30,
    }
}
