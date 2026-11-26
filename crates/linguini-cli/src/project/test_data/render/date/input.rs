//! Parse the generated date runtime's explicit ISO string input domain in UTC.

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
