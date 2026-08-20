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
    WEB_ACCEPT_LANGUAGE_DECLARATIONS, WEB_ACCEPT_LANGUAGE_RUNTIME, WEB_COOKIE_DECLARATIONS,
    WEB_COOKIE_RUNTIME, WEB_DECLARATIONS, WEB_LINK_TRANSFORM_DECLARATIONS,
    WEB_LINK_TRANSFORM_RUNTIME, WEB_LOCAL_STORAGE_DECLARATIONS, WEB_LOCAL_STORAGE_RUNTIME,
    WEB_PATH_DECLARATIONS, WEB_PATH_RUNTIME, WEB_ROUTES_DECLARATIONS, WEB_ROUTES_RUNTIME,
    WEB_RUNTIME, WEB_RUNTIME_LINKS_DECLARATIONS, WEB_RUNTIME_LINKS_RUNTIME,
    WEB_SERVER_COOKIE_DECLARATIONS, WEB_SERVER_COOKIE_RUNTIME, WEB_SWITCH_ROUTE_DECLARATIONS,
    WEB_SWITCH_ROUTE_RUNTIME,
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
    base_locale: Option<&str>,
) -> String {
    let base_locale = base_locale.expect("validated TypeScript projects have a base locale");
    render_template(
        PROJECT_INDEX_ENTRY,
        &[
            ("IMPORTS", project_locale_import(base_locale)),
            ("LOCALE_MODULES", project_locale_modules(base_locale)),
            (
                "LOCALE_LOADERS",
                project_locale_loaders(locales, base_locale),
            ),
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
    _locales: &[TypeScriptLocaleModule],
    _base_locale: Option<&str>,
) -> String {
    render_template(
        PROJECT_INDEX_DECLARATIONS,
        &[(
            "INDEX_RUNTIME_DECLARATIONS",
            template_body(INDEX_RUNTIME_DECLARATIONS),
        )],
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
        "import { browser } from \"$app/environment\";\nimport { base } from \"$app/paths\";"
    } else {
        "const browser = typeof window !== \"undefined\" && typeof document !== \"undefined\";"
    };
    let mut rendered = render_template(
        SVELTE_EFFECTS_RUNTIME,
        &[
            ("BROWSER_RUNTIME", browser_runtime.to_owned()),
            ("OPTIONS", web_options_literal(options)),
            (
                "ENVIRONMENT",
                if sveltekit { "{ base }" } else { "{}" }.to_owned(),
            ),
            (
                "LINK_RUNTIME_IMPORT",
                if options.features().link_mode == super::TypeScriptLinkMode::Runtime {
                    "import { startRuntimeLinkLocalization } from \"./web/runtime-links.js\";"
                        .to_owned()
                } else {
                    String::new()
                },
            ),
            (
                "LINK_RUNTIME_START",
                if options.features().link_mode == super::TypeScriptLinkMode::Runtime {
                    "const linkEffects = browser\n  ? startRuntimeLinkLocalization(web, getCurrentLocale)\n  : undefined;"
                        .to_owned()
                } else {
                    "const linkEffects: { refresh(): void; destroy(): void } | undefined = undefined;"
                        .to_owned()
                },
            ),
        ],
    );
    rendered = gate_browser_capability_reads(rendered, &options.features());
    rendered
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

pub fn generate_project_svelte_control_module_with_options(
    sveltekit: bool,
    options: &TypeScriptWebOptions,
) -> String {
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
    let mut rendered = render_template(
        SVELTE_CONTROL_RUNTIME,
        &[
            ("NAVIGATION_RUNTIME", navigation_runtime.to_owned()),
            ("LOCALE_RUNTIME", locale_runtime.to_owned()),
            ("NAVIGATION", navigation.to_owned()),
        ],
    );
    rendered = gate_browser_capability_writes(rendered, &options.features());
    rendered
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
    render_template(SVELTEKIT_RUNTIME, &sveltekit_replacements(options))
}

pub fn generate_project_sveltekit_declaration() -> String {
    SVELTEKIT_DECLARATIONS.to_owned()
}

pub fn generate_project_sveltekit_control_module(options: &TypeScriptWebOptions) -> String {
    render_template(SVELTEKIT_CONTROL_RUNTIME, &sveltekit_replacements(options))
}

fn sveltekit_replacements(options: &TypeScriptWebOptions) -> Vec<(&'static str, String)> {
    let has_cookie = options.features().has_cookie;
    vec![
        ("OPTIONS", web_options_literal(options)),
        (
            "SERVER_COOKIE_IMPORT",
            if has_cookie {
                "import { persistLocaleCookie } from \"./web/server-cookie.js\";".to_owned()
            } else {
                String::new()
            },
        ),
        (
            "PERSIST_COOKIE_DECLARATION",
            if has_cookie {
                "  const persistCookie = options.persistCookie !== false && web.options.locale.switch.writesCookie;"
                    .to_owned()
            } else {
                String::new()
            },
        ),
        (
            "COOKIE_INPUT",
            if has_cookie {
                "      cookie: event.request.headers.get(\"cookie\") ?? undefined,".to_owned()
            } else {
                String::new()
            },
        ),
        (
            "PERSIST_REDIRECT_COOKIE",
            if has_cookie {
                "      if (persistCookie) persistLocaleCookie(web, response, context.locale, { origin: event.url.origin });"
                    .to_owned()
            } else {
                String::new()
            },
        ),
        (
            "PERSIST_RESPONSE_COOKIE",
            if has_cookie {
                "    if (persistCookie) persistLocaleCookie(web, response, context.locale, { origin: event.url.origin });"
                    .to_owned()
            } else {
                String::new()
            },
        ),
        (
            "SWITCH_ROUTE_IMPORT",
            if options.switch_route.is_some() {
                "import { handleLocaleSwitchRoute } from \"./web/switch-route.js\";".to_owned()
            } else {
                String::new()
            },
        ),
        (
            "SWITCH_ROUTE_BRANCH",
            if options.switch_route.is_some() {
                "    const switchResponse = handleLocaleSwitchRoute(web, event);\n    if (switchResponse) return switchResponse;"
                    .to_owned()
            } else {
                String::new()
            },
        ),
    ]
}

pub fn generate_project_sveltekit_control_declaration() -> String {
    SVELTEKIT_CONTROL_DECLARATIONS.to_owned()
}

/// Render the project web runtime with a closed, statically selected source set.
/// The source template intentionally keeps its compatibility resolver for the
/// standalone runtime; generated projects import one physical module per
/// selected source and call those functions in validated order.
pub fn generate_project_web_module_with_options(options: &TypeScriptWebOptions) -> String {
    let features = options.features();
    let mut runtime = WEB_RUNTIME.to_owned();
    let imports = web_source_imports(&features);
    if !imports.is_empty() {
        runtime = format!("{imports}\n{runtime}");
    }
    if !options.exclude.is_empty() {
        runtime = format!("import {{ matchesRoute }} from \"./web/routes.js\";\n{runtime}");
    }

    let start = runtime
        .find("  function resolveLocaleSync(input: Record<string, unknown> = {}) {")
        .expect("web runtime resolveLocaleSync template");
    let end = runtime[start..]
        .find("\n  function localizeUrl")
        .map(|offset| start + offset)
        .expect("web runtime locale resolver boundary");
    let resolver = selected_locale_resolver(&features);
    runtime.replace_range(start..end, &resolver);

    // Source helpers are emitted by the selected modules. Keep the shared locale
    // matching helpers in this runtime because URL localization uses them too.
    if let Some(start) = runtime.find("\nfunction resolveLocaleSource<") {
        let end = runtime[start..]
            .find("\nfunction matchLocaleValue")
            .map(|offset| start + offset)
            .expect("web runtime source helper boundary");
        runtime.replace_range(start..end, "\n");
    }
    if let Some(start) = runtime.find("\nfunction firstPathSegment(") {
        let end = runtime[start..]
            .find("\nfunction stripLeadingLocale")
            .map(|offset| start + offset)
            .expect("web runtime path helper boundary");
        runtime.replace_range(start..end, "\n");
    }
    if options.exclude.is_empty() {
        let start = runtime
            .find("  function shouldExclude(url: string | URL, input: Record<string, unknown> = {}) {")
            .expect("web runtime route matcher entry");
        let end = runtime[start..]
            .find("\n  function serializeLocaleCookie")
            .map(|offset| start + offset)
            .expect("web runtime route matcher boundary");
        runtime.replace_range(
            start..end,
            "  function shouldExclude(_url: string | URL, _input: Record<string, unknown> = {}) {\n    return false;\n  }\n",
        );
    }
    let start = runtime
        .find("\nfunction matchesRoute(")
        .expect("web runtime shared route matcher");
    let end = runtime[start..]
        .find("\nfunction resolveBaseUrl")
        .map(|offset| start + offset)
        .expect("web runtime route matcher helper boundary");
    runtime.replace_range(start..end, "\n");
    runtime
}

pub fn generate_project_web_source_module(source: TypeScriptLocaleSource) -> String {
    match source {
        TypeScriptLocaleSource::Path => WEB_PATH_RUNTIME.to_owned(),
        TypeScriptLocaleSource::Cookie => WEB_COOKIE_RUNTIME.to_owned(),
        TypeScriptLocaleSource::LocalStorage => WEB_LOCAL_STORAGE_RUNTIME.to_owned(),
        TypeScriptLocaleSource::AcceptLanguage => WEB_ACCEPT_LANGUAGE_RUNTIME.to_owned(),
    }
}

pub fn generate_project_web_source_declaration(source: TypeScriptLocaleSource) -> String {
    match source {
        TypeScriptLocaleSource::Path => WEB_PATH_DECLARATIONS.to_owned(),
        TypeScriptLocaleSource::Cookie => WEB_COOKIE_DECLARATIONS.to_owned(),
        TypeScriptLocaleSource::LocalStorage => WEB_LOCAL_STORAGE_DECLARATIONS.to_owned(),
        TypeScriptLocaleSource::AcceptLanguage => WEB_ACCEPT_LANGUAGE_DECLARATIONS.to_owned(),
    }
}

pub fn generate_project_web_link_module(mode: super::TypeScriptLinkMode) -> Option<String> {
    match mode {
        super::TypeScriptLinkMode::Transform => Some(WEB_LINK_TRANSFORM_RUNTIME.to_owned()),
        super::TypeScriptLinkMode::Runtime => Some(WEB_RUNTIME_LINKS_RUNTIME.to_owned()),
        super::TypeScriptLinkMode::Manual => None,
    }
}

pub fn generate_project_web_link_declaration(mode: super::TypeScriptLinkMode) -> Option<String> {
    match mode {
        super::TypeScriptLinkMode::Transform => Some(WEB_LINK_TRANSFORM_DECLARATIONS.to_owned()),
        super::TypeScriptLinkMode::Runtime => Some(WEB_RUNTIME_LINKS_DECLARATIONS.to_owned()),
        super::TypeScriptLinkMode::Manual => None,
    }
}

pub fn generate_project_web_server_cookie_module() -> String {
    WEB_SERVER_COOKIE_RUNTIME.to_owned()
}

pub fn generate_project_web_server_cookie_declaration() -> String {
    WEB_SERVER_COOKIE_DECLARATIONS.to_owned()
}

pub fn generate_project_web_routes_module() -> String {
    WEB_ROUTES_RUNTIME.to_owned()
}

pub fn generate_project_web_routes_declaration() -> String {
    WEB_ROUTES_DECLARATIONS.to_owned()
}

pub fn generate_project_web_switch_route_module(options: &TypeScriptWebOptions) -> Option<String> {
    let route = options.switch_route.as_ref()?;
    Some(render_template(
        WEB_SWITCH_ROUTE_RUNTIME,
        &[
            (
                "SERVER_COOKIE_IMPORT",
                if options.features().has_cookie {
                    "import { persistLocaleCookie } from \"./server-cookie.js\";".to_owned()
                } else {
                    String::new()
                },
            ),
            ("SWITCH_ROUTE_PATH", escape_string(&route.path)),
            (
                "SWITCH_ROUTE_RETURN_QUERY",
                escape_string(&route.return_query),
            ),
            ("SWITCH_ROUTE_STATUS", route.status.to_string()),
            (
                "PERSIST_SWITCH_COOKIE",
                if options.features().has_cookie {
                    "  if (web.options.locale.switch.writesCookie) {\n    persistLocaleCookie(web, response, locale, { origin: event.url.origin });\n  }"
                        .to_owned()
                } else {
                    String::new()
                },
            ),
        ],
    ))
}

pub fn generate_project_web_switch_route_declaration() -> String {
    WEB_SWITCH_ROUTE_DECLARATIONS.to_owned()
}

pub fn generate_project_web_declaration() -> String {
    WEB_DECLARATIONS.to_owned()
}

fn template_body(template: &str) -> String {
    template.strip_suffix('\n').unwrap_or(template).to_owned()
}

fn project_locale_import(locale: &str) -> String {
    format!(
        "import {} from \"./locales/{}\";",
        locale_identifier(locale),
        escape_string(locale)
    )
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

fn project_locale_modules(locale: &str) -> String {
    format!(
        "  {}: {},\n",
        property_key(locale),
        locale_identifier(locale)
    )
}

fn project_locale_loaders(locales: &[TypeScriptLocaleModule], base_locale: &str) -> String {
    locales
        .iter()
        .map(|locale| {
            if locale.locale == base_locale {
                format!(
                    "  {}: () => Promise.resolve({}),\n",
                    property_key(&locale.locale),
                    locale_identifier(&locale.locale)
                )
            } else {
                format!(
                    "  {}: () => import(\"./locales/{}\").then((module) => module.default),\n",
                    property_key(&locale.locale),
                    escape_string(&locale.locale)
                )
            }
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
    let features = options.features();
    let sources = js_locale_source_array(&options.sources);
    let locale_switch = locale_switch_literal(options.locale_switch);
    let exclude = js_string_array(&options.exclude);
    let mut fields = vec![
        format!(
            "routing: {{ localePrefix: \"{}\", canonical: \"{}\" }}",
            options.locale_prefix.as_str(),
            if options.canonical_redirect {
                "redirect"
            } else {
                "preserve"
            }
        ),
        format!("locale: {{ sources: [{sources}] as const, switch: {locale_switch} }}"),
        format!("links: {{ mode: \"{}\" }}", features.link_mode.as_str()),
        format!("routes: {{ exclude: [{exclude}] as const }}"),
    ];

    if features.has_cookie {
        let mut cookie = vec![
            format!("name: \"{}\"", escape_string(&options.cookie_name)),
            options.cookie_path.as_ref().map_or_else(
                || "path: \"auto\"".to_owned(),
                |path| format!("path: \"{}\"", escape_string(path)),
            ),
            format!("maxAge: {}", options.cookie_max_age),
            format!("sameSite: \"{}\"", escape_string(&options.cookie_same_site)),
            options.cookie_secure.map_or_else(
                || "secure: \"auto\"".to_owned(),
                |secure| format!("secure: {}", js_bool(secure)),
            ),
            format!("httpOnly: {}", js_bool(options.cookie_http_only)),
        ];
        if let Some(cookie_domain) = &options.cookie_domain {
            cookie.push(format!("domain: \"{}\"", escape_string(cookie_domain)));
        }
        fields.push(format!("cookie: {{ {} }}", cookie.join(", ")));
    }
    if features.has_local_storage {
        fields.push(format!(
            "localStorage: {{ key: \"{}\" }}",
            escape_string(&options.local_storage_key)
        ));
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

fn selected_locale_resolver(features: &super::TypeScriptWebFeatures) -> String {
    let mut body =
        String::from("  function resolveLocaleSync(input: Record<string, unknown> = {}) {\n");
    for (index, source) in features.sources.iter().enumerate() {
        let binding = format!("resolved_{index}");
        let statement = match source {
            TypeScriptLocaleSource::Path => {
                format!(
                    "    const {binding} = resolvePathLocale(input, normalized, runtime.locales);\n"
                )
            }
            TypeScriptLocaleSource::Cookie => {
                format!(
                    "    const {binding} = resolveCookieLocale(input, normalized, matchLocale);\n"
                )
            }
            TypeScriptLocaleSource::LocalStorage => {
                format!(
                    "    const {binding} = resolveLocalStorageLocale(input, normalized, matchLocale);\n"
                )
            }
            TypeScriptLocaleSource::AcceptLanguage => {
                format!(
                    "    const {binding} = resolveAcceptLanguageLocale(input, runtime.locales, runtime.baseLocale, matchLocale);\n"
                )
            }
        };
        body.push_str(&statement);
        body.push_str(&format!("    if ({binding}) return {binding};\n"));
    }
    body.push_str("    return runtime.baseLocale;\n  }\n");
    body
}

fn web_source_imports(features: &super::TypeScriptWebFeatures) -> String {
    features
        .sources
        .iter()
        .map(|source| {
            let (name, path) = match source {
                TypeScriptLocaleSource::Path => ("resolvePathLocale", "path"),
                TypeScriptLocaleSource::Cookie => ("resolveCookieLocale", "cookie"),
                TypeScriptLocaleSource::LocalStorage => {
                    ("resolveLocalStorageLocale", "local-storage")
                }
                TypeScriptLocaleSource::AcceptLanguage => {
                    ("resolveAcceptLanguageLocale", "accept-language")
                }
            };
            format!("import {{ {name} }} from \"./web/{path}.js\";")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn gate_browser_capability_writes(
    mut rendered: String,
    features: &super::TypeScriptWebFeatures,
) -> String {
    if !features.has_path {
        // Navigation is a path capability, not merely a persistence write.
        // Remove the whole branch so pathless policies do not ship dead URL
        // reads or framework navigation imports.
        let branch_start =
            rendered.find("      if (options.navigate && web.options.locale.switch.writesPath) {");
        let branch_end = rendered.find("\n      }\n      refreshLinguiniEffects();");
        if let (Some(start), Some(end)) = (branch_start, branch_end) {
            rendered.replace_range(start..end + "\n      }\n".len(), "");
        }
        rendered = rendered.replace("import { goto } from \"$app/navigation\";\n", "");
        rendered = rendered.replace("  clearCurrentLocaleOverride,\n", "");
    }
    if !features.has_local_storage {
        rendered = rendered.replace(
            "      if (web.options.locale.switch.writesLocalStorage) {\n        writeLocalStorage(web, resolved);\n      }\n",
            "",
        );
        if let Some(start) = rendered.find("\nfunction writeLocalStorage(") {
            let end = rendered[start..]
                .find("\nfunction writeLocaleCookie(")
                .unwrap_or(0);
            if end > 0 {
                rendered.replace_range(start..start + end, "\n");
            }
        }
    }
    if !features.has_cookie {
        rendered = rendered.replace(
            "      if (options.cookie && web.options.locale.switch.writesCookie) {\n        writeLocaleCookie(web, resolved);\n      }\n",
            "",
        );
        if let Some(start) = rendered.find("\nfunction writeLocaleCookie(") {
            rendered.truncate(start);
        }
    }
    rendered
}

fn gate_browser_capability_reads(
    mut rendered: String,
    features: &super::TypeScriptWebFeatures,
) -> String {
    let mut reads = Vec::new();
    if features.has_path {
        reads.push("    url: readBrowserCapability(() => window.location.href),");
    }
    if features.has_cookie {
        reads.push("    cookie: readBrowserCapability(() => document.cookie),");
    }
    if features.has_local_storage {
        reads.push("    localStorage: readBrowserCapability(() => window.localStorage),");
    }
    if features.has_accept_language {
        reads.push("    navigator: readBrowserCapability(() => window.navigator),");
    }
    let replacement = format!(
        "  return web.resolveLocaleSync({{\n{}\n  }});",
        reads.join("\n")
    );
    if let Some(start) = rendered.find("  return web.resolveLocaleSync({\n") {
        if let Some(end_offset) = rendered[start..].find("  });") {
            let end = start + end_offset + "  });".len();
            rendered.replace_range(start..end, &replacement);
        }
    }
    rendered
}
