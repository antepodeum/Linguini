use proc_macro2::TokenStream;
use quote::quote;
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LocaleCoverage {
    pub(crate) language_aliases: usize,
    pub(crate) script_aliases: usize,
    pub(crate) territory_aliases: usize,
    pub(crate) variant_aliases: usize,
    pub(crate) parent_locales: usize,
    pub(crate) likely_subtags: usize,
}

pub(crate) fn generate_locale_tables(
    aliases_path: &Path,
    parent_locales_path: &Path,
    likely_subtags_path: &Path,
) -> Result<(TokenStream, LocaleCoverage), String> {
    let aliases = read_json(aliases_path)?;
    let parent_locales = read_json(parent_locales_path)?;
    let likely_subtags = read_json(likely_subtags_path)?;

    let aliases = object_at(&aliases, "/supplemental/metadata/alias", aliases_path)?;
    let language_aliases = alias_entries(aliases, "languageAlias", true, aliases_path)?;
    let script_aliases = alias_entries(aliases, "scriptAlias", false, aliases_path)?;
    let territory_aliases = alias_entries(aliases, "territoryAlias", false, aliases_path)?;
    let variant_aliases = alias_entries(aliases, "variantAlias", true, aliases_path)?;
    let parent_locales = string_entries(
        object_at(
            &parent_locales,
            "/supplemental/parentLocales/parentLocale",
            parent_locales_path,
        )?,
        parent_locales_path,
    )?;
    let likely_subtags = string_entries(
        object_at(
            &likely_subtags,
            "/supplemental/likelySubtags",
            likely_subtags_path,
        )?,
        likely_subtags_path,
    )?;

    let language_alias_table = string_lookup("generated_language_alias", &language_aliases);
    let script_alias_table = string_lookup("generated_script_alias", &script_aliases);
    let territory_alias_table = string_lookup("generated_territory_alias", &territory_aliases);
    let variant_alias_table = string_lookup("generated_variant_alias", &variant_aliases);
    let parent_locale_table = string_lookup("generated_parent_locale", &parent_locales);
    let likely_subtag_table = string_lookup("generated_likely_subtag", &likely_subtags);

    let coverage = LocaleCoverage {
        language_aliases: language_aliases.len(),
        script_aliases: script_aliases.len(),
        territory_aliases: territory_aliases.len(),
        variant_aliases: variant_aliases.len(),
        parent_locales: parent_locales.len(),
        likely_subtags: likely_subtags.len(),
    };

    Ok((
        quote! {
            #language_alias_table
            #script_alias_table
            #territory_alias_table
            #variant_alias_table
            #parent_locale_table
            #likely_subtag_table
        },
        coverage,
    ))
}

fn read_json(path: &Path) -> Result<Value, String> {
    let source =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_str(&source).map_err(|error| format!("{}: {error}", path.display()))
}

fn object_at<'a>(
    value: &'a Value,
    pointer: &str,
    path: &Path,
) -> Result<&'a Map<String, Value>, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{}: missing object `{pointer}`", path.display()))
}

fn alias_entries(
    aliases: &Map<String, Value>,
    key: &str,
    lowercase_key: bool,
    path: &Path,
) -> Result<Vec<(String, String)>, String> {
    let aliases = aliases
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{}: missing alias object `{key}`", path.display()))?;
    let mut entries = aliases
        .iter()
        .map(|(source, value)| {
            let replacement = value
                .get("_replacement")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    format!(
                        "{}: alias `{key}.{source}` has no string `_replacement`",
                        path.display()
                    )
                })?;
            let source = source.replace('_', "-");
            Ok((
                if lowercase_key {
                    source.to_ascii_lowercase()
                } else {
                    source
                },
                replacement.replace('_', "-"),
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    entries.sort();
    Ok(entries)
}

fn string_entries(
    values: &Map<String, Value>,
    path: &Path,
) -> Result<Vec<(String, String)>, String> {
    let mut entries = values
        .iter()
        .map(|(key, value)| {
            let value = value
                .as_str()
                .ok_or_else(|| format!("{}: `{key}` must map to a string", path.display()))?;
            Ok((key.replace('_', "-"), value.replace('_', "-")))
        })
        .collect::<Result<Vec<_>, String>>()?;
    entries.sort();
    Ok(entries)
}

fn string_lookup(name: &str, entries: &[(String, String)]) -> TokenStream {
    let name = quote::format_ident!("{name}");
    let match_arms = entries
        .iter()
        .map(|(key, value)| quote! { #key => Some(#value), });
    quote! {
        pub(crate) fn #name(value: &str) -> Option<&'static str> {
            match value {
                #(#match_arms)*
                _ => None,
            }
        }
    }
}
