//! The pinned upstream validity file contains obsolete IDs absent from CLDR JSON containment.

use crate::sha256::{digest, hex};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const SOURCE: &str = include_str!("../data/subdivision.xml");
pub(crate) const SOURCE_REF: &str = "release-48-2";
pub(crate) const SOURCE_URL: &str = "https://github.com/unicode-org/cldr/blob/11299982335beb974c1c63c45265184e759c0f41/common/validity/subdivision.xml";
pub(crate) const SOURCE_SHA256: &str =
    "93b12c9d55938266c96d44a7ccbb66800afeef4f9dd48b0dc16edfab89833d95";

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SubdivisionValidity {
    pub(crate) regular: BTreeSet<String>,
    pub(crate) deprecated: BTreeSet<String>,
    pub(crate) unknown: BTreeSet<String>,
}

pub(crate) fn subdivisions() -> Result<SubdivisionValidity, String> {
    if hex(&digest(SOURCE.as_bytes())) != SOURCE_SHA256 {
        return Err("bundled CLDR subdivision validity SHA-256 mismatch".to_owned());
    }
    parse(SOURCE)
}

fn parse(source: &str) -> Result<SubdivisionValidity, String> {
    // The upstream file declares a DTD. No entity resolver is installed: parsing stays offline.
    let document = roxmltree::Document::parse_with_options(
        source,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )
    .map_err(|error| format!("invalid subdivision validity XML: {error}"))?;
    let root = document.root_element();
    if !root.has_tag_name("supplementalData") {
        return Err("missing subdivision validity root".to_owned());
    }
    let mut groups = BTreeMap::new();
    for node in root
        .children()
        .filter(|node| node.has_tag_name("idValidity"))
        .flat_map(|node| node.children())
        .filter(|node| node.is_element())
    {
        if !node.has_tag_name("id") || node.attribute("type") != Some("subdivision") {
            return Err("invalid subdivision validity element".to_owned());
        }
        let status = node
            .attribute("idStatus")
            .ok_or("missing subdivision status")?;
        if !matches!(status, "regular" | "deprecated" | "unknown") {
            return Err(format!("unsupported subdivision status {status}"));
        }
        let mut codes = BTreeSet::new();
        for child in node.children() {
            if child.is_element() {
                return Err("nested subdivision validity element".to_owned());
            }
            if !child.is_text() {
                continue;
            }
            for token in child.text().unwrap_or_default().split_whitespace() {
                for code in expand(token)? {
                    if !codes.insert(code.clone()) {
                        return Err(format!("duplicate subdivision validity code {code}"));
                    }
                }
            }
        }
        if codes.is_empty() || groups.insert(status, codes).is_some() {
            return Err(format!("empty or duplicate subdivision status {status}"));
        }
    }
    let mut take = |status| {
        groups
            .remove(status)
            .ok_or_else(|| format!("missing subdivision status {status}"))
    };
    let validity = SubdivisionValidity {
        regular: take("regular")?,
        deprecated: take("deprecated")?,
        unknown: take("unknown")?,
    };
    if !validity.regular.is_disjoint(&validity.deprecated)
        || !validity.regular.is_disjoint(&validity.unknown)
        || !validity.deprecated.is_disjoint(&validity.unknown)
    {
        return Err("overlapping subdivision validity statuses".to_owned());
    }
    Ok(validity)
}

fn expand(token: &str) -> Result<Vec<String>, String> {
    let Some((start, end)) = token.split_once('~') else {
        validate_code(token)?;
        return Ok(vec![token.to_owned()]);
    };
    validate_code(start)?;
    if end.len() != 1 || start.as_bytes()[start.len() - 1] > end.as_bytes()[0] {
        return Err(format!("invalid subdivision validity range {token}"));
    }
    (start.as_bytes()[start.len() - 1]..=end.as_bytes()[0])
        .map(|last| {
            let mut code = start[..start.len() - 1].to_owned();
            code.push(char::from(last));
            validate_code(&code)?;
            Ok(code)
        })
        .collect()
}

fn validate_code(code: &str) -> Result<(), String> {
    if !(3..=8).contains(&code.len())
        || !code
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        || !code.as_bytes()[..2].iter().all(u8::is_ascii_lowercase)
    {
        return Err(format!("invalid subdivision validity code {code}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(regular: &str, deprecated: &str, unknown: &str) -> String {
        format!("<supplementalData><idValidity><id type='subdivision' idStatus='regular'>{regular}</id><id type='subdivision' idStatus='deprecated'>{deprecated}</id><id type='subdivision' idStatus='unknown'>{unknown}</id></idValidity></supplementalData>")
    }

    #[test]
    fn pinned_validity_preserves_regular_deprecated_and_unknown_identities() {
        let data = subdivisions().unwrap();
        assert_eq!(
            (
                data.regular.len(),
                data.deprecated.len(),
                data.unknown.len()
            ),
            (5_027, 626, 257)
        );
        assert!(data.regular.contains("gbeng"));
        assert!(data.deprecated.contains("gbgbn"));
        assert!(data.unknown.contains("gbzzzz"));
        let ranged = parse(&fixture(
            "<!-- items -->ad02~8 <!-- next -->gbeng",
            "gbgbn",
            "gbzzzz",
        ))
        .unwrap();
        assert_eq!(
            ranged.regular,
            ["ad02", "ad03", "ad04", "ad05", "ad06", "ad07", "ad08", "gbeng"]
                .map(str::to_owned)
                .into()
        );
        assert_eq!(ranged.deprecated, ["gbgbn".to_owned()].into());
        assert_eq!(ranged.unknown, ["gbzzzz".to_owned()].into());
    }

    #[test]
    fn compiled_week_preferences_match_every_pinned_validity_status() {
        use linguini_cldr::{canonicalize_locale, compiled_week_data, WeekData};
        let data = subdivisions().unwrap();
        let base = WeekData {
            first_day: 0,
            min_days: 1,
        };
        let mut checked = 0;
        let mut aliases = 0;
        for code in data
            .regular
            .iter()
            .chain(&data.deprecated)
            .chain(&data.unknown)
        {
            for key in ["sd", "rg"] {
                let tag = format!("en-u-{key}-{code}");
                let canonical = canonicalize_locale(&tag).unwrap();
                aliases += usize::from(canonical != tag);
                let value = canonical.strip_prefix(&format!("en-u-{key}-")).unwrap();
                let valid = data.regular.contains(value)
                    || key == "sd" && data.deprecated.contains(value)
                    || key == "rg" && data.unknown.contains(value);
                // An explicit territory is the independent oracle for a valid preference.
                let expected = if valid {
                    compiled_week_data(&format!("en-{}", value[..2].to_ascii_uppercase())).unwrap()
                } else {
                    base
                };
                assert_eq!(
                    compiled_week_data(&tag),
                    Some(expected),
                    "{tag} -> {canonical}"
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 11_820);
        assert_eq!(aliases, 294);
    }

    #[test]
    fn invalid_xml_codes_ranges_and_status_overlaps_fail_closed() {
        for source in ["", "<wrong/>", "<supplementalData><idValidity>"] {
            assert!(parse(source).is_err());
        }
        for regular in [
            "",
            "GBENG",
            "éabc",
            "1abc",
            "gb",
            "gbeng gbeng",
            "ad08~2",
            "ad02~09",
            "ad09~a",
            "ad02~8~9",
            "<child/>",
        ] {
            assert!(
                parse(&fixture(regular, "gbgbn", "gbzzzz")).is_err(),
                "{regular}"
            );
        }
        assert!(parse(&fixture("gbeng", "gbeng", "gbzzzz")).is_err());
        assert!(parse(&fixture("gbeng", "gbgbn", "gbeng")).is_err());
        let duplicate = fixture("gbeng", "gbgbn", "gbzzzz").replace(
            "</idValidity>",
            "<id type='subdivision' idStatus='regular'>usca</id></idValidity>",
        );
        assert!(parse(&duplicate).is_err());
    }
}
