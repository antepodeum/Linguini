//! Semantic project imports and locale loaders for the canonical provider template.

use minijinja::context;

use super::{locale_identifier, EcmaLocaleModule};
use crate::module::names::{property_key, string_literal};

pub(super) fn render(locales: &[EcmaLocaleModule], base_locale: Option<&str>) -> String {
    let base_locale = base_locale.expect("validated ECMAScript projects have a base locale");
    let entries: Vec<_> = locales
        .iter()
        .map(|entry| {
            context! {
                property => property_key(&entry.locale),
                path => string_literal(&format!("./locales/{}.js", entry.locale)),
                is_base => entry.locale == base_locale,
            }
        })
        .collect();
    crate::templates::render(
        "project/index",
        context! {
            base_binding => locale_identifier(base_locale),
            base_property => property_key(base_locale),
            base_path => string_literal(&format!("./locales/{base_locale}.js")),
            locales => entries,
        },
    )
}
