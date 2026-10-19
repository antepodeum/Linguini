//! Semantic locale metadata and build-time locale resolution.

use std::collections::BTreeMap;

use linguini_cldr::{canonicalize_locale, locale_resolution_candidates, maximize_locale};
use minijinja::{context, Value};

use super::{locale_direction, EcmaLocaleModule};
use crate::module::names::{property_key, string_literal};

fn metadata(locales: &[EcmaLocaleModule], base_locale: Option<&str>) -> Value {
    let base_locale =
        base_locale.expect("validated ECMAScript projects have an explicit base locale");
    debug_assert!(locales.iter().any(|entry| entry.locale == base_locale));
    let entries: Vec<_> = locales
        .iter()
        .map(|entry| {
            context! {
                literal => string_literal(&entry.locale),
                property => property_key(&entry.locale),
                direction => locale_direction(&entry.locale),
            }
        })
        .collect();
    let resolutions: Vec<_> = project_locale_resolution(locales)
        .into_iter()
        .map(|(candidate, resolved)| {
            context! {
                candidate => string_literal(&candidate),
                locale => string_literal(&resolved),
            }
        })
        .collect();
    context! { locales => entries, base_locale => string_literal(base_locale), resolutions }
}

pub(super) fn render(locales: &[EcmaLocaleModule], base_locale: Option<&str>) -> String {
    crate::templates::render("project/locale", metadata(locales, base_locale))
}

pub(super) fn declaration(locales: &[EcmaLocaleModule], base_locale: Option<&str>) -> String {
    crate::templates::render(
        "project/locale_declarations",
        metadata(locales, base_locale),
    )
}

fn project_locale_resolution(locales: &[EcmaLocaleModule]) -> BTreeMap<String, String> {
    let mut candidates = locale_resolution_candidates()
        .iter()
        .map(|locale| (*locale).to_owned())
        .collect::<Vec<_>>();
    for locale in locales {
        let Ok(canonical) = canonicalize_locale(&locale.locale) else {
            continue;
        };
        let Some(language) = canonical.split('-').next() else {
            continue;
        };
        let Ok(maximized) = maximize_locale(language) else {
            continue;
        };
        if let Some(script) = maximized
            .split('-')
            .nth(1)
            .filter(|subtag| subtag.len() == 4)
        {
            candidates.push(format!("{language}-{script}"));
        }
    }
    candidates.sort();
    candidates.dedup();

    let mut resolution = locales
        .iter()
        .map(|locale| (locale.locale.to_ascii_lowercase(), locale.locale.clone()))
        .collect::<BTreeMap<_, _>>();
    for candidate in candidates {
        if let Some(resolved) = super::super::locale_fallback_chain(locales, &candidate, None)
            .into_iter()
            .next()
        {
            resolution.insert(candidate.to_ascii_lowercase(), resolved);
        }
    }
    resolution
}
