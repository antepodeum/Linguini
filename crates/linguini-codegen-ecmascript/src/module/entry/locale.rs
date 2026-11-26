//! Semantic locale metadata and build-time locale resolution.

use linguini_codegen::ProjectLocalePlan;
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
    let resolutions: Vec<_> =
        ProjectLocalePlan::new(locales.iter().map(|entry| entry.locale.as_str()))
            .resolutions()
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
