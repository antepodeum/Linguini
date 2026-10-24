//! Resolve used date-field symbols before target-language emission.

use linguini_cldr::CalendarFormatData;

/// Only the symbol values needed by one compiled date field.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DateFieldSymbols {
    pub names: Option<Vec<&'static str>>,
    pub am: Option<&'static str>,
    pub pm: Option<&'static str>,
    pub ad: Option<&'static str>,
    pub bc: Option<&'static str>,
}

/// Resolve CLDR context/width choices in Rust, with no target syntax or runtime lookup data.
/// `None` indicates a required symbol is absent from the supplied calendar.
/// `day_period_plan` separately resolves fixed/flexible rules and their symbol fallbacks.
pub fn date_field_symbols(
    calendar: &CalendarFormatData,
    field: char,
    width: usize,
) -> Option<DateFieldSymbols> {
    let symbol_width = match width {
        4 => "wide",
        5 => "narrow",
        6 => "short",
        _ => "abbreviated",
    };
    let mut symbols = DateFieldSymbols::default();
    match field {
        'M' | 'L' if width >= 3 => {
            let context = if field == 'L' {
                "stand-alone"
            } else {
                "format"
            };
            symbols.names = Some(
                (1..=12)
                    .map(|month| {
                        calendar.symbol(&format!("months.{context}.{symbol_width}.{month}"))
                    })
                    .collect::<Option<Vec<_>>>()?,
            );
        }
        'E' | 'e' | 'c' if field == 'E' || width >= 3 => {
            let context = if field == 'c' {
                "stand-alone"
            } else {
                "format"
            };
            symbols.names = Some(
                ["sun", "mon", "tue", "wed", "thu", "fri", "sat"]
                    .iter()
                    .map(|day| {
                        calendar
                            .symbol(&format!("days.{context}.{symbol_width}.{day}"))
                            .or_else(|| {
                                (symbol_width == "short")
                                    .then(|| {
                                        calendar
                                            .symbol(&format!("days.{context}.abbreviated.{day}"))
                                    })
                                    .flatten()
                            })
                    })
                    .collect::<Option<Vec<_>>>()?,
            );
        }
        'a' | 'b' | 'B' => {
            symbols.am = Some(calendar.symbol(&format!("dayPeriods.format.{symbol_width}.am"))?);
            symbols.pm = Some(calendar.symbol(&format!("dayPeriods.format.{symbol_width}.pm"))?);
        }
        'G' => {
            let era_width = match width {
                4 => "eraNames",
                5 => "eraNarrow",
                _ => "eraAbbr",
            };
            symbols.ad = Some(calendar.symbol(&format!("eras.{era_width}.1"))?);
            symbols.bc = Some(calendar.symbol(&format!("eras.{era_width}.0"))?);
        }
        _ => {}
    }
    Some(symbols)
}
