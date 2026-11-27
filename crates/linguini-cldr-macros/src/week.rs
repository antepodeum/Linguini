//! Week rules and the validity of locale preferences, consumed only by the compiler.

use crate::source_paths::CldrSource;
use proc_macro2::TokenStream;
use quote::quote;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct WeekCoverage {
    pub(crate) regions: usize,
    pub(crate) subdivisions: usize,
}

pub(crate) fn generate_week_tables(
    source: &CldrSource,
) -> Result<(TokenStream, WeekCoverage), String> {
    let read = |name: &str| -> Result<Value, String> {
        let path = source.supplemental_path(name);
        serde_json::from_slice(
            &std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
        )
        .map_err(|e| format!("{}: {e}", path.display()))
    };
    let week = read("weekData")?;
    let week = &week["supplemental"]["weekData"];
    let territories = object(&week["firstDay"])?
        .keys()
        .chain(object(&week["minDays"])?.keys())
        .collect::<BTreeSet<_>>();
    let mut week_arms = Vec::new();
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
    let containment = read("territoryContainment")?;
    let subdivisions = read("subdivisionContainment")?;
    let aliases = read("aliases")?;
    let regions = regular_regions(&containment["supplemental"]["territoryContainment"])?;
    let subdivisions = regular_subdivisions(
        &subdivisions["supplemental"]["subdivisionContainment"],
        &aliases["supplemental"]["metadata"]["alias"]["subdivisionAlias"],
        &regions,
    )?;
    let coverage = WeekCoverage {
        regions: regions.len(),
        subdivisions: subdivisions.values().map(Vec::len).sum(),
    };
    let region_values = regions.iter();
    let subdivision_arms = subdivisions
        .iter()
        .map(|(region, codes)| quote! { #(#codes)|* => Some(#region), });
    Ok((
        quote! {
            fn generated_week_data(territory: &str) -> Option<WeekData> { match territory { #(#week_arms)* _ => None } }
            pub(crate) fn generated_regular_week_region(region: &str) -> bool { matches!(region, #(#region_values)|*) }
            pub(crate) fn generated_week_subdivision_region(subdivision: &str) -> Option<&'static str> {
                match subdivision { #(#subdivision_arms)* _ => None }
            }
        },
        coverage,
    ))
}

fn object(value: &Value) -> Result<&Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| "missing week preference data object".to_owned())
}

fn children(value: &Value) -> Result<Vec<&str>, String> {
    value["_contains"]
        .as_array()
        .ok_or("missing containment children")?
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| "invalid containment child".to_owned())
        })
        .collect()
}

fn regular_regions(containment: &Value) -> Result<BTreeSet<String>, String> {
    let containment = object(containment)?;
    let groups = containment
        .keys()
        .map(|key| key.split('-').next().unwrap())
        .collect::<BTreeSet<_>>();
    let mut regions = BTreeSet::new();
    for (group, value) in containment {
        if group.contains("-status-deprecated") {
            continue;
        }
        for region in children(value)? {
            if region.len() == 2
                && region.bytes().all(|b| b.is_ascii_uppercase())
                && !groups.contains(region)
            {
                regions.insert(region.to_owned());
            }
        }
    }
    if regions.is_empty() {
        return Err("empty week region validity data".to_owned());
    }
    Ok(regions)
}

fn regular_subdivisions(
    containment: &Value,
    aliases: &Value,
    regions: &BTreeSet<String>,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    let aliases = object(aliases)?;
    let mut codes = BTreeSet::new();
    for (parent, value) in object(containment)? {
        codes.insert(parent.as_str());
        codes.extend(children(value)?);
    }
    let mut subdivisions = BTreeMap::<_, Vec<_>>::new();
    for code in codes {
        if code.len() == 2 && regions.contains(&code.to_ascii_uppercase()) {
            continue;
        }
        if !(3..=8).contains(&code.len())
            || !code
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        {
            return Err(format!("invalid subdivision code {code}"));
        }
        let region = code[..2].to_ascii_uppercase();
        if !regions.contains(&region) {
            return Err(format!("invalid subdivision region {code}"));
        }
        // Country roots and obsolete codes are containment nodes, not regular subdivisions.
        if !aliases.contains_key(code) {
            subdivisions
                .entry(region)
                .or_default()
                .push(code.to_owned());
        }
    }
    if subdivisions.is_empty() {
        return Err("empty week subdivision validity data".to_owned());
    }
    Ok(subdivisions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validity_uses_regular_leaves_and_nested_subdivisions_without_aliases() {
        let regions = regular_regions(&json!({
            "001":{"_contains":["150","US","AC"]},
            "150":{"_contains":["GB","FR"]},
            "EU-grouping":{"_contains":["FR"]},
            "001-status-deprecated":{"_contains":["BU","EU"]}
        }))
        .unwrap();
        assert_eq!(regions, ["AC", "FR", "GB", "US"].map(str::to_owned).into());
        let subdivisions = regular_subdivisions(
            &json!({
            "GB":{"_contains":["gbeng"]}, "gbeng":{"_contains":["gbabd"]},
            "FR":{"_contains":["frges","frbl"]}
            }),
            &json!({"frbl":{"_replacement":"BL"}}),
            &regions,
        )
        .unwrap();
        assert_eq!(
            subdivisions,
            BTreeMap::from([
                ("FR".to_owned(), vec!["frges".to_owned()]),
                (
                    "GB".to_owned(),
                    vec!["gbabd".to_owned(), "gbeng".to_owned()]
                )
            ])
        );
        for bad in [
            json!({}),
            json!({"gb":{"_contains":[null]}}),
            json!({"gb":{"_contains":["xxbad"]}}),
        ] {
            assert!(regular_subdivisions(&bad, &json!({}), &regions).is_err());
        }
        assert!(regular_regions(&json!({"001":{}})).is_err());
    }
}
