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
        'M' | 'L' | 'Q' | 'q' if width >= 3 => {
            let (group, count) = if matches!(field, 'Q' | 'q') {
                ("quarters", 4)
            } else {
                ("months", 12)
            };
            let context = if matches!(field, 'L' | 'q') {
                "stand-alone"
            } else {
                "format"
            };
            symbols.names = Some(
                (1..=count)
                    .map(|index| {
                        calendar.symbol(&format!("{group}.{context}.{symbol_width}.{index}"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarter_symbols_cover_both_contexts_and_all_widths_in_pinned_locales() {
        let mut locales = std::collections::BTreeSet::new();
        for locale in linguini_cldr::locale_resolution_candidates() {
            let Some(calendar) = linguini_cldr::compiled_calendar_formatting(locale, "gregory")
            else {
                continue;
            };
            if !locales.insert(calendar.locale) {
                continue;
            }
            for (field, context) in [('Q', "format"), ('q', "stand-alone")] {
                for width in 1..=5 {
                    let symbols =
                        date_field_symbols(&calendar, field, width).unwrap_or_else(|| {
                            panic!(
                                "{}: {field}/{width} has missing quarter symbols",
                                calendar.locale
                            )
                        });
                    if width <= 2 {
                        assert_eq!(symbols, DateFieldSymbols::default());
                        continue;
                    }
                    let names = symbols.names.unwrap();
                    assert_eq!(names.len(), 4);
                    let width = match width {
                        3 => "abbreviated",
                        4 => "wide",
                        _ => "narrow",
                    };
                    for (index, name) in names.into_iter().enumerate() {
                        assert_eq!(
                            Some(name),
                            calendar.symbol(&format!("quarters.{context}.{width}.{}", index + 1))
                        );
                    }
                }
            }
        }
        assert_eq!(locales.len(), 508);
    }
}
