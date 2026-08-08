use std::collections::BTreeMap;

use super::names::{escape_string, property_key, safe_identifier};
use super::templates::{
    render_template, INDEX_RUNTIME, INDEX_RUNTIME_DECLARATIONS, LOCALE_DECLARATIONS,
    LOCALE_RUNTIME, PROJECT_INDEX_DECLARATIONS, PROJECT_INDEX_ENTRY, SVELTEKIT_DECLARATIONS,
    SVELTEKIT_RUNTIME, SVELTE_CONTEXT_DECLARATIONS, SVELTE_CONTEXT_RUNTIME, SVELTE_DECLARATIONS,
    SVELTE_LOCALE_CONTEXT_DECLARATIONS, SVELTE_LOCALE_CONTEXT_RUNTIME, SVELTE_LOCALE_DECLARATIONS,
    SVELTE_LOCALE_RUNTIME, SVELTE_RUNTIME, WEB_DECLARATIONS, WEB_RUNTIME,
};
use super::{TypeScriptLocaleModule, TypeScriptLocaleSource, TypeScriptWebOptions};
use linguini_cldr::{
    built_in_text_direction, canonicalize_locale, locale_resolution_candidates, maximize_locale,
};

pub fn generate_project_index(
    locales: &[TypeScriptLocaleModule],
    _base_locale: Option<&str>,
) -> String {
    render_template(
        PROJECT_INDEX_ENTRY,
        &[
            ("IMPORTS", project_locale_imports(locales)),
            ("LOCALE_MODULES", project_locale_modules(locales)),
            ("LOCALE_LOADERS", project_locale_loaders(locales)),
            ("INDEX_RUNTIME", template_body(INDEX_RUNTIME)),
        ],
    )
}

pub fn generate_project_locale(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    render_template(
        LOCALE_RUNTIME,
        &[
            ("LOCALES", locale_literals(locales).join(", ")),
            ("BASE_LOCALE", base_locale_literal(locales, base_locale)),
            ("LOCALE_DIRECTIONS", project_locale_directions(locales)),
            (
                "LOCALE_RESOLUTION_OVERRIDES",
                project_locale_resolution_overrides(locales),
            ),
        ],
    )
}

pub fn generate_project_locale_declaration(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    render_template(
        LOCALE_DECLARATIONS,
        &[
            ("LOCALES", locale_literals(locales).join(", ")),
            ("BASE_LOCALE", base_locale_literal(locales, base_locale)),
            (
                "LOCALE_DIRECTIONS",
                project_locale_direction_declarations(locales),
            ),
        ],
    )
}

fn project_locale_resolution_overrides(locales: &[TypeScriptLocaleModule]) -> String {
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

    let mut overrides = BTreeMap::new();
    for candidate in candidates {
        let resolved = super::locale_fallback_chain(locales, &candidate, None)
            .into_iter()
            .next();
        let runtime_fallback = runtime_locale_match(locales, &candidate);
        if resolved != runtime_fallback {
            overrides.insert(candidate.to_ascii_lowercase(), resolved);
        }
    }

    overrides
        .into_iter()
        .map(|(candidate, resolved)| {
            let value = resolved.map_or_else(
                || "null".to_owned(),
                |locale| format!("\"{}\"", escape_string(&locale)),
            );
            format!("  \"{}\": {value},\n", escape_string(&candidate))
        })
        .collect()
}

fn runtime_locale_match(locales: &[TypeScriptLocaleModule], locale: &str) -> Option<String> {
    let mut tag = locale;
    loop {
        if let Some(locale) = locales
            .iter()
            .find(|locale| locale.locale.eq_ignore_ascii_case(tag))
        {
            return Some(locale.locale.clone());
        }
        if is_language_script_tag(tag) {
            return None;
        }
        let dash = tag.rfind('-')?;
        if dash == 0 {
            return None;
        }
        tag = &tag[..dash];
    }
}

fn is_language_script_tag(locale: &str) -> bool {
    let Some((language, script)) = locale.split_once('-') else {
        return false;
    };
    !language.contains('-')
        && (2..=8).contains(&language.len())
        && language.bytes().all(|byte| byte.is_ascii_alphabetic())
        && script.len() == 4
        && script.bytes().all(|byte| byte.is_ascii_alphabetic())
}

pub fn generate_project_index_declaration(
    locales: &[TypeScriptLocaleModule],
    _base_locale: Option<&str>,
) -> String {
    render_template(
        PROJECT_INDEX_DECLARATIONS,
        &[
            ("IMPORTS", project_locale_imports(locales)),
            (
                "LOCALE_MODULES",
                project_locale_module_declarations(locales),
            ),
            (
                "LOCALE_LOADERS",
                project_locale_loader_declarations(locales),
            ),
            (
                "INDEX_RUNTIME_DECLARATIONS",
                template_body(INDEX_RUNTIME_DECLARATIONS),
            ),
        ],
    )
}

pub fn generate_project_svelte_locale_module(web: bool) -> String {
    if web {
        SVELTE_LOCALE_RUNTIME.to_owned()
    } else {
        SVELTE_LOCALE_CONTEXT_RUNTIME.to_owned()
    }
}

pub fn generate_project_svelte_locale_declaration(web: bool) -> String {
    if web {
        SVELTE_LOCALE_DECLARATIONS.to_owned()
    } else {
        SVELTE_LOCALE_CONTEXT_DECLARATIONS.to_owned()
    }
}

pub fn generate_project_svelte_module(options: Option<&TypeScriptWebOptions>) -> String {
    match options {
        Some(options) => {
            render_template(SVELTE_RUNTIME, &[("OPTIONS", web_options_literal(options))])
        }
        None => SVELTE_CONTEXT_RUNTIME.to_owned(),
    }
}

pub fn generate_project_svelte_declaration(web: bool) -> String {
    if web {
        SVELTE_DECLARATIONS.to_owned()
    } else {
        SVELTE_CONTEXT_DECLARATIONS.to_owned()
    }
}

pub fn generate_project_sveltekit_module(options: &TypeScriptWebOptions) -> String {
    render_template(
        SVELTEKIT_RUNTIME,
        &[("OPTIONS", web_options_literal(options))],
    )
}

pub fn generate_project_sveltekit_declaration() -> String {
    SVELTEKIT_DECLARATIONS.to_owned()
}

pub fn generate_project_web_module() -> String {
    WEB_RUNTIME.to_owned()
}

pub fn generate_project_web_declaration() -> String {
    WEB_DECLARATIONS.to_owned()
}

fn template_body(template: &str) -> String {
    template.strip_suffix('\n').unwrap_or(template).to_owned()
}

fn project_locale_imports(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "import {} from \"./locales/{}\";",
                locale_identifier(&locale.locale),
                escape_string(&locale.locale)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn project_locale_directions(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  {}: \"{}\",\n",
                property_key(&locale.locale),
                locale_direction(&locale.locale)
            )
        })
        .collect::<String>()
}

fn project_locale_direction_declarations(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  readonly {}: \"{}\";\n",
                property_key(&locale.locale),
                locale_direction(&locale.locale)
            )
        })
        .collect::<String>()
}

fn project_locale_modules(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  {}: {},\n",
                property_key(&locale.locale),
                locale_identifier(&locale.locale)
            )
        })
        .collect::<String>()
}

fn project_locale_module_declarations(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  readonly {}: typeof {};\n",
                property_key(&locale.locale),
                locale_identifier(&locale.locale)
            )
        })
        .collect::<String>()
}

fn project_locale_loaders(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  {}: () => Promise.resolve({}),\n",
                property_key(&locale.locale),
                locale_identifier(&locale.locale)
            )
        })
        .collect::<String>()
}

fn project_locale_loader_declarations(locales: &[TypeScriptLocaleModule]) -> String {
    locales
        .iter()
        .map(|locale| {
            format!(
                "  readonly {}: () => Promise<typeof {}>;\n",
                property_key(&locale.locale),
                locale_identifier(&locale.locale)
            )
        })
        .collect::<String>()
}

fn locale_identifier(locale: &str) -> String {
    format!("locale_{}", safe_identifier(locale))
}

fn locale_literals(locales: &[TypeScriptLocaleModule]) -> Vec<String> {
    locales
        .iter()
        .map(|locale| format!("\"{}\"", escape_string(&locale.locale)))
        .collect()
}

fn base_locale_literal(locales: &[TypeScriptLocaleModule], base_locale: Option<&str>) -> String {
    let locale = base_locale.expect("validated TypeScript projects have an explicit base locale");
    debug_assert!(
        locales.iter().any(|entry| entry.locale == locale),
        "validated TypeScript projects contain the configured base locale"
    );
    format!("\"{}\"", escape_string(locale))
}

fn web_options_literal(options: &TypeScriptWebOptions) -> String {
    let sources = js_locale_source_array(&options.sources);
    let exclude = js_string_array(&options.exclude);
    let mut fields = vec![
        format!("sources: [{sources}] as const"),
        format!("cookieName: \"{}\"", escape_string(&options.cookie_name)),
        format!("cookiePath: \"{}\"", escape_string(&options.cookie_path)),
        format!("cookieMaxAge: {}", options.cookie_max_age),
        format!(
            "cookieSameSite: \"{}\"",
            escape_string(&options.cookie_same_site)
        ),
        format!("cookieSecure: {}", js_bool(options.cookie_secure)),
        format!("cookieHttpOnly: {}", js_bool(options.cookie_http_only)),
        format!(
            "localStorageKey: \"{}\"",
            escape_string(&options.local_storage_key)
        ),
        format!(
            "prefixDefaultLocale: {}",
            js_bool(options.prefix_default_locale)
        ),
        format!("basePath: \"{}\"", escape_string(&options.base_path)),
        format!("redirect: {}", js_bool(options.redirect)),
        format!("exclude: [{exclude}] as const"),
        format!("localizeLinks: {}", js_bool(options.localize_links)),
    ];

    if let Some(cookie_domain) = &options.cookie_domain {
        fields.push(format!(
            "cookieDomain: \"{}\"",
            escape_string(cookie_domain)
        ));
    }
    if let Some(origin) = &options.origin {
        fields.push(format!("origin: \"{}\"", escape_string(origin)));
    }

    format!("{{ {} }} as const", fields.join(", "))
}

fn js_locale_source_array(values: &[TypeScriptLocaleSource]) -> String {
    values
        .iter()
        .map(|source| format!("\"{}\"", source.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn js_string_array(values: &[String]) -> String {
    values
        .iter()
        .map(|item| format!("\"{}\"", escape_string(item)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn js_bool(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn locale_direction(locale: &str) -> &'static str {
    built_in_text_direction(locale).unwrap_or("ltr")
}
