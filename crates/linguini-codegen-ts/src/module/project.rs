use std::collections::BTreeMap;

use super::names::{escape_string, property_key, safe_identifier};
use super::templates::{
    render_template, INDEX_RUNTIME, INDEX_RUNTIME_DECLARATIONS, LOCALE_DECLARATIONS,
    LOCALE_RUNTIME, PROJECT_INDEX_DECLARATIONS, PROJECT_INDEX_ENTRY,
    SVELTEKIT_CONTROL_DECLARATIONS, SVELTEKIT_CONTROL_RUNTIME, SVELTEKIT_DECLARATIONS,
    SVELTEKIT_RUNTIME, SVELTE_CONTEXT_DECLARATIONS, SVELTE_CONTEXT_RUNTIME,
    SVELTE_CONTROL_DECLARATIONS, SVELTE_CONTROL_RUNTIME, SVELTE_DECLARATIONS,
    SVELTE_EFFECTS_DECLARATIONS, SVELTE_EFFECTS_RUNTIME, SVELTE_LOCALE_CONTEXT_DECLARATIONS,
    SVELTE_LOCALE_CONTEXT_RUNTIME, SVELTE_LOCALE_DECLARATIONS, SVELTE_LOCALE_RUNTIME,
    SVELTE_LOCALE_STANDALONE_DECLARATIONS, SVELTE_LOCALE_STANDALONE_RUNTIME, SVELTE_RUNTIME,
    WEB_DECLARATIONS, WEB_RUNTIME,
};
use super::{
    TypeScriptLocaleModule, TypeScriptLocaleSource, TypeScriptLocaleSwitchPlan,
    TypeScriptWebOptions,
};
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

pub fn generate_project_svelte_locale_module(web: bool, sveltekit: bool) -> String {
    if web && sveltekit {
        SVELTE_LOCALE_RUNTIME.to_owned()
    } else if web {
        SVELTE_LOCALE_STANDALONE_RUNTIME.to_owned()
    } else {
        SVELTE_LOCALE_CONTEXT_RUNTIME.to_owned()
    }
}

pub fn generate_project_svelte_locale_declaration(web: bool, sveltekit: bool) -> String {
    if web && sveltekit {
        SVELTE_LOCALE_DECLARATIONS.to_owned()
    } else if web {
        SVELTE_LOCALE_STANDALONE_DECLARATIONS.to_owned()
    } else {
        SVELTE_LOCALE_CONTEXT_DECLARATIONS.to_owned()
    }
}

pub fn generate_project_svelte_effects_module(
    options: &TypeScriptWebOptions,
    sveltekit: bool,
) -> String {
    let browser_runtime = if sveltekit {
        "import { browser } from \"$app/environment\";"
    } else {
        "const browser = typeof window !== \"undefined\" && typeof document !== \"undefined\";"
    };
    render_template(
        SVELTE_EFFECTS_RUNTIME,
        &[
            ("BROWSER_RUNTIME", browser_runtime.to_owned()),
            ("OPTIONS", web_options_literal(options)),
        ],
    )
}

pub fn generate_project_svelte_effects_declaration() -> String {
    SVELTE_EFFECTS_DECLARATIONS.to_owned()
}

pub fn generate_project_svelte_module(
    options: Option<&TypeScriptWebOptions>,
    sveltekit: bool,
) -> String {
    match options {
        Some(_) => render_project_svelte_web_module(sveltekit),
        None => SVELTE_CONTEXT_RUNTIME.to_owned(),
    }
}

fn render_project_svelte_web_module(_sveltekit: bool) -> String {
    SVELTE_RUNTIME.to_owned()
}

pub fn generate_project_svelte_control_module(sveltekit: bool) -> String {
    let (navigation_runtime, locale_runtime, navigation) = if sveltekit {
        (
            "import { browser } from \"$app/environment\";\nimport { goto } from \"$app/navigation\";",
            "import {\n  clearCurrentLocaleOverride,\n  getCurrentLocale,\n  prepareLocale,\n  setCurrentLocale,\n} from \"./svelte-locale.svelte.js\";",
            "        const href = web.localizeHref(window.location.href, resolved);\n        await goto(href, {\n          replaceState: Boolean(options.replaceState),\n          invalidateAll: Boolean(options.invalidateAll),\n          keepFocus: options.keepFocus as boolean | undefined,\n          noScroll: options.noScroll as boolean | undefined,\n          state: options.state as App.PageState | undefined,\n        });\n        clearCurrentLocaleOverride();",
        )
    } else {
        (
            "const browser = typeof window !== \"undefined\" && typeof document !== \"undefined\";",
            "import {\n  getCurrentLocale,\n  prepareLocale,\n  setCurrentLocale,\n} from \"./svelte-locale.svelte.js\";",
            "        const href = web.localizeHref(window.location.href, resolved);\n        if (options.replaceState) {\n          window.location.replace(href);\n        } else {\n          window.location.assign(href);\n        }",
        )
    };
    render_template(
        SVELTE_CONTROL_RUNTIME,
        &[
            ("NAVIGATION_RUNTIME", navigation_runtime.to_owned()),
            ("LOCALE_RUNTIME", locale_runtime.to_owned()),
            ("NAVIGATION", navigation.to_owned()),
        ],
    )
}

pub fn generate_project_svelte_declaration(web: bool, _sveltekit: bool) -> String {
    if web {
        SVELTE_DECLARATIONS.to_owned()
    } else {
        SVELTE_CONTEXT_DECLARATIONS.to_owned()
    }
}

pub fn generate_project_svelte_control_declaration(sveltekit: bool) -> String {
    render_template(
        SVELTE_CONTROL_DECLARATIONS,
        &[(
            "PAGE_STATE",
            if sveltekit {
                "App.PageState"
            } else {
                "unknown"
            }
            .to_owned(),
        )],
    )
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

pub fn generate_project_sveltekit_control_module(options: &TypeScriptWebOptions) -> String {
    render_template(
        SVELTEKIT_CONTROL_RUNTIME,
        &[("OPTIONS", web_options_literal(options))],
    )
}

pub fn generate_project_sveltekit_control_declaration() -> String {
    SVELTEKIT_CONTROL_DECLARATIONS.to_owned()
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
    let locale_switch = locale_switch_literal(options.locale_switch);
    let exclude = js_string_array(&options.exclude);
    let mut fields = vec![
        format!("sources: [{sources}] as const"),
        format!("localeSwitch: {locale_switch}"),
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

fn locale_switch_literal(plan: TypeScriptLocaleSwitchPlan) -> String {
    format!(
        "{{ writesPath: {}, writesCookie: {}, writesLocalStorage: {} }} as const",
        js_bool(plan.writes_path),
        js_bool(plan.writes_cookie),
        js_bool(plan.writes_local_storage),
    )
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
