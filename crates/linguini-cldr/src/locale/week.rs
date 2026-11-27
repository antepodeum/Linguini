//! Resolve locale week preferences before any target code is generated.

use super::{canonical_locale_id, language_likely_target, normalize_placeholder_subtags};
use crate::data::{generated_regular_week_region, generated_week_subdivision_region, WeekData};

pub(crate) fn resolve_week_data(
    locale: &str,
    lookup: impl Fn(&str) -> Option<WeekData>,
) -> Option<WeekData> {
    let locale = canonical_locale_id(locale).ok()?;
    if locale.private_only {
        return None;
    }
    let region_override = match locale.unicode_keyword("rg") {
        Some([region]) => region
            .strip_suffix("zzzz")
            .map(str::to_ascii_uppercase)
            .filter(|region| generated_regular_week_region(region))
            .or_else(|| generated_week_subdivision_region(region).map(str::to_owned)),
        _ => None,
    };
    let territory = region_override
        .as_deref()
        .or(locale.region.as_deref())
        .or_else(|| match locale.unicode_keyword("sd") {
            Some([subdivision]) => generated_week_subdivision_region(subdivision),
            _ => None,
        });
    let likely = if territory.is_none() {
        let mut normalized = locale.clone();
        normalize_placeholder_subtags(&mut normalized);
        language_likely_target(&normalized)
    } else {
        None
    };
    let territory = territory
        .or_else(|| likely.as_ref()?.region.as_deref())
        .unwrap_or("001");
    let mut week = lookup(territory).or_else(|| lookup("001"))?;
    if matches!(locale.unicode_keyword("ca"), Some([calendar]) if calendar == "iso8601") {
        // ISO minimum days remain four even if rg or fw selects another first weekday.
        week.min_days = 4;
        if region_override.is_none() {
            week.first_day = 1;
        }
    }
    // fw takes precedence over both region and calendar first-weekday preferences.
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
    fn week_preferences_follow_cldr_precedence_and_calendar_minimum_days() {
        for (locale, first_day, min_days) in [
            ("en-US-u-rg-gbzzzz", 1, 4),
            ("en-u-rg-gbeng", 1, 4),
            ("en-u-rg-frges", 1, 4),
            ("en-u-rg-fi01", 1, 4),
            ("en-u-rg-frbl", 1, 1),
            ("en-u-sd-fra", 1, 4),
            ("en-u-sd-cn11", 1, 1),
            ("en-u-sd-lud", 1, 4),
            ("en-u-ca-iso8601-rg-afbal", 6, 4),
            ("en-GB-u-rg-uszzzz", 0, 1),
            // AC is a regular territory with no explicit week-data row.
            ("en-US-u-rg-aczzzz", 1, 1),
            ("EN-us-u-RG-GBZZZZ", 1, 4),
            ("en-US-u-ca-iso8601", 1, 4),
            ("en-US-u-ca-iso8601-fw-sun", 0, 4),
            ("en-AU-u-ca-iso8601-fw-tue-rg-afzzzz-sd-cabc", 2, 4),
            ("en-AU-u-ca-iso8601-rg-afzzzz-sd-cabc", 6, 4),
            ("en-AU-u-ca-iso8601-sd-cabc", 1, 4),
            ("en-AU-u-sd-cabc", 1, 1),
            ("en-u-sd-cabc", 0, 1),
            ("en-u-sd-gbeng", 1, 4),
            ("en-u-sd-gbabd", 1, 4),
            ("en-u-sd-frges", 1, 4),
            ("en-US-u-sd-gbeng", 0, 1),
            ("en-u-fw-sun-sd-gbeng", 0, 4),
            ("en-u-rg-uszzzz-sd-gbeng", 0, 1),
            ("en-ZZ-u-sd-gbeng", 1, 1),
            ("zxx", 1, 1),
            ("zxx-u-fw-tue", 2, 1),
            ("und-Arab", 6, 1),
            ("en-u-ca-gregory", 0, 1),
        ] {
            assert_eq!(
                crate::compiled_week_data(locale),
                Some(WeekData {
                    first_day,
                    min_days
                }),
                "{locale}"
            );
        }
    }

    #[test]
    fn invalid_preferences_and_other_extensions_cannot_override_week_data() {
        for locale in [
            "en-u-rg-zzzzzz",
            "en-u-rg-xxzzzz",
            "en-u-rg-buzzzz",
            "en-u-rg-ukzzzz",
            "en-u-rg-euzzzz",
            "en-u-rg-001zzzz",
            "en-u-rg-419zzzz",
            "en-u-rg-gb",
            "en-u-rg-gbzzzz-uszzzz",
            "en-u-rg-gbxxxx",
            "en-u-rg",
            "en-u-sd-gbxxx",
            "en-u-sd-fi01",
            "en-u-sd-shta",
            "en-u-sd-frxxx",
            "en-u-sd-frzzzz",
            "en-u-sd-gbeng-usca",
            "en-u-sd-gb",
            "en-u-sd",
            "en-u-ca-iso8601-unknown",
            "en-a-rg-gbzzzz",
            "en-t-und-rg-gbzzzz",
            "en-x-u-rg-gbzzzz",
            "en-u-nu-latn-x-sd-gbeng",
            "en-x-u-ca-iso8601",
        ] {
            assert_eq!(
                crate::compiled_week_data(locale),
                Some(WeekData {
                    first_day: 0,
                    min_days: 1
                }),
                "{locale}"
            );
        }
        for locale in ["en-u-rg-gbzzzz-rg-uszzzz", "x-u-sd-gbeng"] {
            assert_eq!(crate::compiled_week_data(locale), None, "{locale}");
        }
    }

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
