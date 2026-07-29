use proc_macro2::TokenStream;
use quote::quote;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

type ExtensionKeyAliases = Vec<((String, String), String)>;
type ExtensionTypeAliases = Vec<((String, String, String), String)>;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LocaleCoverage {
    pub(crate) language_aliases: usize,
    pub(crate) script_aliases: usize,
    pub(crate) territory_aliases: usize,
    pub(crate) variant_aliases: usize,
    pub(crate) parent_locales: usize,
    pub(crate) locale_rules: usize,
    pub(crate) likely_subtags: usize,
    pub(crate) extension_key_aliases: usize,
    pub(crate) extension_type_aliases: usize,
    pub(crate) locale_candidates: usize,
}

pub(crate) fn generate_locale_tables(
    aliases_path: &Path,
    parent_locales_path: &Path,
    likely_subtags_path: &Path,
    bcp47_path: &Path,
) -> Result<(TokenStream, LocaleCoverage), String> {
    let aliases = read_json(aliases_path)?;
    let parent_locales = read_json(parent_locales_path)?;
    let likely_subtags = read_json(likely_subtags_path)?;

    let aliases = object_at(&aliases, "/supplemental/metadata/alias", aliases_path)?;
    let language_aliases = alias_entries(aliases, "languageAlias", true, aliases_path)?;
    let script_aliases = alias_entries(aliases, "scriptAlias", false, aliases_path)?;
    let territory_aliases = alias_entries(aliases, "territoryAlias", false, aliases_path)?;
    let variant_aliases = alias_entries(aliases, "variantAlias", true, aliases_path)?;
    let nonlikely_script_parent = parent_locales
        .pointer("/supplemental/parentLocales/_localeRules/parentLocale/nonlikelyScript")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            format!(
                "{}: missing nonlikelyScript parent locale rule",
                parent_locales_path.display()
            )
        })?
        .replace('_', "-");
    if nonlikely_script_parent != "root" {
        return Err(format!(
            "{}: nonlikelyScript parent is `{nonlikely_script_parent}`, expected `root`",
            parent_locales_path.display()
        ));
    }
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
    let (extension_key_aliases, extension_type_aliases) = extension_alias_entries(bcp47_path)?;
    let locale_candidates = language_aliases
        .iter()
        .map(|(key, _)| key.clone())
        .chain(parent_locales.iter().map(|(key, _)| key.clone()))
        .chain(likely_subtags.iter().map(|(key, _)| key.clone()))
        .collect::<BTreeSet<_>>();

    let language_alias_table = string_lookup("generated_language_alias", &language_aliases);
    let script_alias_table = string_lookup("generated_script_alias", &script_aliases);
    let territory_alias_table = string_lookup("generated_territory_alias", &territory_aliases);
    let variant_alias_table = string_lookup("generated_variant_alias", &variant_aliases);
    let parent_locale_table = string_lookup("generated_parent_locale", &parent_locales);
    let likely_subtag_table = string_lookup("generated_likely_subtag", &likely_subtags);
    let extension_key_alias_table =
        tuple2_lookup("generated_extension_key_alias", &extension_key_aliases);
    let extension_type_alias_table =
        tuple3_lookup("generated_extension_type_alias", &extension_type_aliases);
    let locale_candidate_values = locale_candidates.iter();
    let locale_candidate_table = quote! {
        pub(crate) const GENERATED_LOCALE_CANDIDATES: &[&str] = &[
            #(#locale_candidate_values),*
        ];
    };
    let nonlikely_script_rule = quote! {
        pub(crate) const GENERATED_NONLIKELY_SCRIPT_PARENT: &str = #nonlikely_script_parent;
    };

    let coverage = LocaleCoverage {
        language_aliases: language_aliases.len(),
        script_aliases: script_aliases.len(),
        territory_aliases: territory_aliases.len(),
        variant_aliases: variant_aliases.len(),
        parent_locales: parent_locales.len(),
        locale_rules: 1,
        likely_subtags: likely_subtags.len(),
        extension_key_aliases: extension_key_aliases.len(),
        extension_type_aliases: extension_type_aliases.len(),
        locale_candidates: locale_candidates.len(),
    };

    Ok((
        quote! {
            #language_alias_table
            #script_alias_table
            #territory_alias_table
            #variant_alias_table
            #parent_locale_table
            #nonlikely_script_rule
            #likely_subtag_table
            #extension_key_alias_table
            #extension_type_alias_table
            #locale_candidate_table
        },
        coverage,
    ))
}

fn extension_alias_entries(
    directory: &Path,
) -> Result<(ExtensionKeyAliases, ExtensionTypeAliases), String> {
    let mut paths = fs::read_dir(directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{}: {error}", directory.display()))?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();

    let mut key_aliases = BTreeMap::new();
    let mut type_aliases = BTreeMap::new();
    for path in paths {
        let value = read_json(&path)?;
        let keywords = object_at(&value, "/keyword", &path)?;
        for extension in ["u", "t"] {
            let Some(keys) = keywords.get(extension).and_then(Value::as_object) else {
                continue;
            };
            for (key, metadata) in keys {
                let metadata = metadata.as_object().ok_or_else(|| {
                    format!("{}: `{extension}.{key}` must be an object", path.display())
                })?;
                if let Some(aliases) = metadata.get("_alias").and_then(Value::as_str) {
                    for alias in aliases.split_whitespace().filter(|alias| alias.len() == 2) {
                        key_aliases.insert(
                            (extension.to_owned(), alias.to_ascii_lowercase()),
                            key.to_ascii_lowercase(),
                        );
                    }
                }
                for (canonical_type, type_metadata) in metadata {
                    if canonical_type.starts_with('_') || !valid_extension_type(canonical_type) {
                        continue;
                    }
                    let Some(aliases) = type_metadata.get("_alias").and_then(Value::as_str) else {
                        continue;
                    };
                    for alias in aliases.split_whitespace() {
                        let alias = alias.replace('_', "-").to_ascii_lowercase();
                        if valid_extension_type(&alias) {
                            type_aliases.insert(
                                (extension.to_owned(), key.to_ascii_lowercase(), alias),
                                canonical_type.to_ascii_lowercase(),
                            );
                        }
                    }
                }
            }
        }
    }
    Ok((
        key_aliases.into_iter().collect(),
        type_aliases.into_iter().collect(),
    ))
}

fn valid_extension_type(value: &str) -> bool {
    value.split('-').all(|part| {
        (3..=8).contains(&part.len()) && part.bytes().all(|byte| byte.is_ascii_alphanumeric())
    })
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

fn tuple2_lookup(name: &str, entries: &[((String, String), String)]) -> TokenStream {
    let name = quote::format_ident!("{name}");
    if entries.is_empty() {
        return quote! {
            pub(crate) fn #name(_: &str, _: &str) -> Option<&'static str> {
                None
            }
        };
    }
    let match_arms = entries.iter().map(|((first, second), value)| {
        quote! { (#first, #second) => Some(#value), }
    });
    quote! {
        pub(crate) fn #name(first: &str, second: &str) -> Option<&'static str> {
            match (first, second) {
                #(#match_arms)*
                _ => None,
            }
        }
    }
}

fn tuple3_lookup(name: &str, entries: &[((String, String, String), String)]) -> TokenStream {
    let name = quote::format_ident!("{name}");
    if entries.is_empty() {
        return quote! {
            pub(crate) fn #name(_: &str, _: &str, _: &str) -> Option<&'static str> {
                None
            }
        };
    }
    let match_arms = entries.iter().map(|((first, second, third), value)| {
        quote! { (#first, #second, #third) => Some(#value), }
    });
    quote! {
        pub(crate) fn #name(
            first: &str,
            second: &str,
            third: &str,
        ) -> Option<&'static str> {
            match (first, second, third) {
                #(#match_arms)*
                _ => None,
            }
        }
    }
}
