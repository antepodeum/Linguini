//! Resolve locale week preferences before any target code is generated.

use super::{canonical_locale_id, maximize_id};
use crate::data::WeekData;

pub(crate) fn resolve_week_data(
    locale: &str,
    lookup: impl Fn(&str) -> Option<WeekData>,
) -> Option<WeekData> {
    let locale = canonical_locale_id(locale).ok()?;
    if locale.private_only {
        return None;
    }
    let maximized = maximize_id(&locale);
    let territory = maximized.as_ref().unwrap_or(&locale).region.as_deref()?;
    let mut week = lookup(territory).or_else(|| lookup("001"))?;
    // CLDR fw overrides only the first weekday; minDays still belongs to the territory.
    if let Some([day]) = locale.unicode_keyword("fw") {
        if let Some(first_day) = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"]
            .iter()
            .position(|candidate| *candidate == day)
        {
            week.first_day = first_day as u8;
        }
    }
    Some(week)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_weekday_overrides_preserve_territory_minimum_days_and_extension_boundaries() {
        use crate::compiled_week_data;
        let days = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
        for locale in crate::locale_resolution_candidates() {
            let Some(base) = compiled_week_data(locale) else {
                continue;
            };
            // Form a language tag before appending a Unicode extension: grandfathered
            // aliases and private-use suffixes cannot accept keywords at their raw end.
            let canonical = super::canonical_locale_id(locale)
                .unwrap()
                .without_extensions()
                .to_tag();
            for (first_day, day) in days.iter().enumerate() {
                let requested = format!("{canonical}-u-fw-{day}");
                assert_eq!(
                    compiled_week_data(&requested),
                    Some(WeekData {
                        first_day: first_day as u8,
                        min_days: base.min_days,
                    }),
                    "{requested}"
                );
            }
        }
        for (locale, first_day, min_days) in [
            ("EN-gb-u-FW-SUN", 0, 4),
            ("eng-USA-u-fw-thu", 4, 1),
            ("en-u-attr-fw-mon-nu-latn", 1, 1),
            ("en-u-ca-gregory-fw-tue", 2, 1),
            ("en-a-fw-mon", 0, 1),
            ("en-x-u-fw-mon", 0, 1),
            ("en-u-fw-mon-tue", 0, 1),
            ("en-u-fw-friday", 0, 1),
            ("en-u-fw", 0, 1),
            ("en-u-fw-true", 0, 1),
            ("en-u-nu-latn-x-fw-mon", 0, 1),
        ] {
            assert_eq!(
                compiled_week_data(locale),
                Some(WeekData {
                    first_day,
                    min_days
                }),
                "{locale}"
            );
        }
        for locale in ["", "en_US", "en-u-fw-mon-fw-tue", "x-u-fw-mon"] {
            assert_eq!(compiled_week_data(locale), None, "{locale}");
        }
    }
}
