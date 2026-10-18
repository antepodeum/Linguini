//! Pinned calendar symbols and patterns; this runs only in the offline maintainer tool.

use crate::source_paths::CldrSource;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) const CALENDARS: [&str; 4] = ["gregorian", "buddhist", "roc", "japanese"];

pub(crate) fn generate_calendar_tables(source: &CldrSource) -> Result<TokenStream, String> {
    let mut arms = Vec::new();
    let mut symbol_tables = BTreeMap::new();
    let mut definitions = Vec::new();
    let mut pair_count = 0;
    let mut groups = BTreeMap::new();
    for locale in source.locales() {
        let mut calendar_arms = Vec::new();
        for calendar in CALENDARS {
            let path = source.calendar_path(locale, calendar);
            let json = read_json(&path)?;
            let data = json
                .pointer(&format!("/main/{locale}/dates/calendars/{calendar}"))
                .ok_or_else(|| format!("{}: missing calendar", path.display()))?;
            let Some(dates) = widths(&data["dateFormats"], true)? else {
                if locale != "haw" {
                    return Err(format!(
                        "unexpected calendar exclusion: {locale}/{calendar}"
                    ));
                }
                continue;
            };
            let times = widths(&data["timeFormats"], false)?.unwrap();
            let japanese_year_patterns = ["full", "long", "medium", "short"]
                .into_iter()
                .filter_map(|width| {
                    let value = &data["dateFormats"][width];
                    (value["_numbers"] == "y=jpanyear").then(|| value["_value"].as_str().unwrap())
                })
                .collect::<Vec<_>>();
            let combined = widths(&data["dateTimeFormats"], false)?.unwrap();
            let mut symbols = Vec::new();
            for name in ["months", "days", "quarters", "dayPeriods", "eras"] {
                let mut entries = Vec::new();
                flatten(name, &data[name], &mut entries)?;
                let next = symbol_tables.len();
                let id = symbol_tables.entry(entries.clone()).or_insert_with(|| {
                    let id = format_ident!("CALENDAR_SYMBOLS_{next}");
                    let entries = entries
                        .iter()
                        .map(|(name, value)| quote! { (#name, #value) });
                    definitions.push(quote! { static #id: &[(&str, &str)] = &[#(#entries),*]; });
                    id
                });
                symbols.push(id.clone());
            }
            let next_group = groups.len();
            let group = groups.entry(symbols.clone()).or_insert_with(|| {
                let id = format_ident!("CALENDAR_SYMBOL_GROUP_{next_group}");
                definitions.push(quote! { static #id: &[&[(&str, &str)]] = &[#(#symbols),*]; });
                id
            });
            let skeletons = data["dateTimeFormats"]["availableFormats"]
                .as_object()
                .ok_or_else(|| format!("{}: missing availableFormats", path.display()))?;
            let skeletons = skeletons
                .iter()
                .map(|(name, value)| {
                    let value = value
                        .as_str()
                        .ok_or_else(|| format!("{}: invalid skeleton {name}", path.display()))?;
                    Ok(quote! { (#name, #value) })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let target_calendar = if calendar == "gregorian" {
                "gregory"
            } else {
                calendar
            };
            pair_count += 1;
            calendar_arms.push(quote! {
                #target_calendar => Some(CalendarFormatData {
                    locale: #locale, calendar: #target_calendar,
                    date_formats: #dates, time_formats: #times, date_time_formats: #combined,
                    symbols: #group, skeletons: &[#(#skeletons),*],
                    japanese_year_patterns: &[#(#japanese_year_patterns),*],
                }),
            });
        }
        if !calendar_arms.is_empty() {
            arms.push(quote! { #locale => match calendar { #(#calendar_arms)* _ => None }, });
        }
    }
    if pair_count != (source.locales().len() - 1) * CALENDARS.len() {
        return Err("calendar coverage differs from the pinned exclusion manifest".to_owned());
    }
    Ok(quote! {
        #(#definitions)*
        fn generated_calendar_formatting(locale: &str, calendar: &str) -> Option<CalendarFormatData> {
            match locale { #(#arms)* _ => None }
        }
    })
}

fn read_json(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn widths(value: &Value, allow_numbering_override: bool) -> Result<Option<TokenStream>, String> {
    let mut result = Vec::new();
    for width in ["full", "long", "medium", "short"] {
        let mut item = &value[width];
        if allow_numbering_override && item.get("_numbers").is_some() {
            if item["_numbers"] == "M=romanlow" {
                return Ok(None);
            }
            if item["_numbers"] != "y=jpanyear" {
                return Err(format!("unsupported date numbering override: {item}"));
            }
            item = &item["_value"];
        }
        result.push(
            item.as_str()
                .ok_or_else(|| format!("invalid date width {width}: {item}"))?,
        );
    }
    let (full, long, medium, short) = (result[0], result[1], result[2], result[3]);
    Ok(Some(
        quote! { FormatWidths { full: #full, long: #long, medium: #medium, short: #short } },
    ))
}

fn flatten(prefix: &str, value: &Value, output: &mut Vec<(String, String)>) -> Result<(), String> {
    if let Some(text) = value.as_str() {
        output.push((prefix.to_owned(), text.to_owned()));
    } else if let Some(object) = value.as_object() {
        if object.is_empty() {
            return Err(format!("empty calendar symbols: {prefix}"));
        }
        for (name, child) in object {
            flatten(&format!("{prefix}.{name}"), child, output)?;
        }
    } else {
        return Err(format!("invalid calendar symbol {prefix}: {value}"));
    }
    Ok(())
}

fn minutes(value: &Value) -> Result<Option<u16>, String> {
    if value.is_null() {
        return Ok(None);
    }
    let text = value.as_str().ok_or("invalid day-period time")?;
    let (hour, minute) = text.split_once(':').ok_or("invalid day-period clock")?;
    let hour = hour.parse::<u16>().map_err(|_| "invalid hour")?;
    let minute = minute.parse::<u16>().map_err(|_| "invalid minute")?;
    if hour > 24 || minute > 59 || (hour == 24 && minute != 0) {
        return Err("day-period clock out of range".to_owned());
    }
    Ok(Some(hour * 60 + minute))
}

fn option_tokens(value: Option<u16>) -> TokenStream {
    value.map_or_else(|| quote! { None }, |value| quote! { Some(#value) })
}

pub(crate) fn generate_supplemental_tables(source: &CldrSource) -> Result<TokenStream, String> {
    let periods = read_json(&source.supplemental_path("dayPeriods"))?;
    let periods = periods["supplemental"]["dayPeriodRuleSet"]
        .as_object()
        .ok_or("missing day period rules")?;
    let mut period_arms = Vec::new();
    for (locale, rules) in periods {
        let mut items = Vec::new();
        for (name, rule) in rules.as_object().ok_or("invalid day-period rules")? {
            let at = minutes(&rule["_at"])?;
            let from = minutes(&rule["_from"])?;
            let before = minutes(&rule["_before"])?;
            if at.is_none() && (from.is_none() || before.is_none()) {
                return Err(format!("incomplete day-period rule {locale}/{name}"));
            }
            let (at, from, before) = (
                option_tokens(at),
                option_tokens(from),
                option_tokens(before),
            );
            items.push(
                quote! { DayPeriodRule { name: #name, at: #at, from: #from, before: #before } },
            );
        }
        period_arms.push(quote! { #locale => Some(&[#(#items),*]), });
    }
    let week = read_json(&source.supplemental_path("weekData"))?;
    let week = &week["supplemental"]["weekData"];
    let mut week_arms = Vec::new();
    let territories = week["firstDay"]
        .as_object()
        .ok_or("missing firstDay")?
        .keys()
        .chain(week["minDays"].as_object().ok_or("missing minDays")?.keys())
        .collect::<std::collections::BTreeSet<_>>();
    for territory in territories {
        let first = week["firstDay"]
            .get(territory)
            .unwrap_or(&week["firstDay"]["001"])
            .as_str()
            .ok_or("invalid firstDay")?;
        let first = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"]
            .iter()
            .position(|day| *day == first)
            .ok_or("invalid weekday")? as u8;
        let min_days = week["minDays"]
            .get(territory)
            .unwrap_or(&week["minDays"]["001"])
            .as_str()
            .ok_or("invalid minDays")?
            .parse::<u8>()
            .map_err(|_| "invalid minDays")?;
        if !(1..=7).contains(&min_days) {
            return Err("minDays out of range".to_owned());
        }
        week_arms.push(
            quote! { #territory => Some(WeekData { first_day: #first, min_days: #min_days }), },
        );
    }
    let eras = read_json(&source.supplemental_path("calendarData"))?;
    let eras = eras["supplemental"]["calendarData"]["japanese"]["eras"]
        .as_object()
        .ok_or("missing Japanese eras")?;
    let mut starts = Vec::new();
    for (id, era) in eras {
        let id = id.parse::<u16>().map_err(|_| "invalid era ID")?;
        let start = era["_start"].as_str().ok_or("missing era start")?;
        let parts = start
            .split('-')
            .map(str::parse::<i32>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "invalid Japanese era start")?;
        if parts.len() != 3 || !(1..=12).contains(&parts[1]) || !(1..=31).contains(&parts[2]) {
            return Err("invalid era date".to_owned());
        }
        starts.push((id, parts));
    }
    starts.sort_by_key(|(id, _)| *id);
    let starts = starts.iter().map(|(id, parts)| {
        let (year, month, day) = (parts[0], parts[1] as u8, parts[2] as u8);
        quote! { (#id, #year, #month, #day) }
    });
    let mut zones = Vec::new();
    for locale in source.locales() {
        let json = read_json(&source.dates_main().join(locale).join("timeZoneNames.json"))?;
        let data = &json["main"][locale]["dates"]["timeZoneNames"];
        let gmt = data["gmtFormat"].as_str().ok_or("missing gmtFormat")?;
        let zero = data["gmtZeroFormat"]
            .as_str()
            .ok_or("missing gmtZeroFormat")?;
        let hour = data["hourFormat"].as_str().ok_or("missing hourFormat")?;
        zones.push(quote! { #locale => Some(TimeZoneFormatData { gmt_format: #gmt, gmt_zero_format: #zero, hour_format: #hour }), });
    }
    Ok(quote! {
        fn generated_day_period_rules(locale: &str) -> Option<&'static [DayPeriodRule]> { match locale { #(#period_arms)* _ => None } }
        fn generated_week_data(territory: &str) -> Option<WeekData> { match territory { #(#week_arms)* _ => None } }
        fn generated_japanese_era_starts() -> &'static [(u16, i32, u8, u8)] { &[#(#starts),*] }
        fn generated_time_zone_formatting(locale: &str) -> Option<TimeZoneFormatData> { match locale { #(#zones)* _ => None } }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn calendar_payloads_preserve_numbering_overrides_and_reject_malformed_symbols() {
        let mut input = json!({"full":"G y", "long":"G y", "medium":"G y", "short":"G y"});
        input["full"] = json!({"_numbers":"y=jpanyear", "_value":"Gy年"});
        assert!(widths(&input, true)
            .unwrap()
            .unwrap()
            .to_string()
            .contains("Gy年"));
        input["full"]["_numbers"] = json!("unknown");
        assert!(widths(&input, true).is_err());
        input["full"]["_numbers"] = json!("M=romanlow");
        assert!(widths(&input, true).unwrap().is_none());
        let mut symbols = Vec::new();
        flatten(
            "months",
            &json!({"stand-alone":{"wide":{"1":"январь"}}}),
            &mut symbols,
        )
        .unwrap();
        assert_eq!(
            symbols,
            [("months.stand-alone.wide.1".to_owned(), "январь".to_owned())]
        );
        assert!(flatten("months", &json!({}), &mut symbols).is_err());
        assert!(flatten("months", &json!({"1":12}), &mut symbols).is_err());
    }

    #[test]
    fn day_period_boundaries_validate_midnight_and_reject_invalid_clocks() {
        assert_eq!(minutes(&json!("00:00")).unwrap(), Some(0));
        assert_eq!(minutes(&json!("24:00")).unwrap(), Some(1440));
        assert_eq!(minutes(&Value::Null).unwrap(), None);
        for value in [
            json!("24:01"),
            json!("25:00"),
            json!("12:60"),
            json!("12"),
            json!(12),
        ] {
            assert!(minutes(&value).is_err());
        }
    }
}
