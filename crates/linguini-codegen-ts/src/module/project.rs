use super::names::escape_string;
use super::templates::{
    render_template, SVELTEKIT_CONTROL_DECLARATIONS, SVELTEKIT_CONTROL_RUNTIME,
    SVELTEKIT_DECLARATIONS, SVELTEKIT_RUNTIME, SVELTE_CONTEXT_DECLARATIONS, SVELTE_CONTEXT_RUNTIME,
    SVELTE_CONTROL_DECLARATIONS, SVELTE_CONTROL_RUNTIME, SVELTE_DECLARATIONS,
    SVELTE_EFFECTS_DECLARATIONS, SVELTE_EFFECTS_RUNTIME, SVELTE_LOCALE_CONTEXT_DECLARATIONS,
    SVELTE_LOCALE_CONTEXT_RUNTIME, SVELTE_LOCALE_DECLARATIONS, SVELTE_LOCALE_RUNTIME,
    SVELTE_LOCALE_STANDALONE_DECLARATIONS, SVELTE_LOCALE_STANDALONE_RUNTIME, SVELTE_RUNTIME,
    WEB_DECLARATIONS, WEB_RUNTIME, WEB_SERVER_COOKIE_DECLARATIONS, WEB_SERVER_COOKIE_RUNTIME,
    WEB_SWITCH_ROUTE_DECLARATIONS, WEB_SWITCH_ROUTE_RUNTIME,
};
use super::web_runtime_links;
use super::{web_accept_language, web_cookie, web_link_transform, web_local_storage, web_path};
use super::{TypeScriptLocaleSource, TypeScriptLocaleSwitchPlan, TypeScriptWebOptions};

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
        TypeScriptLocaleSource::Path => web_path::generate_typescript_web_path_module(),
        TypeScriptLocaleSource::Cookie => web_cookie::generate_typescript_web_cookie_module(),
        TypeScriptLocaleSource::LocalStorage => {
            web_local_storage::generate_typescript_web_local_storage_module()
        }
        TypeScriptLocaleSource::AcceptLanguage => {
            web_accept_language::generate_typescript_web_accept_language_module()
        }
    }
}

pub fn generate_project_web_source_declaration(source: TypeScriptLocaleSource) -> String {
    match source {
        TypeScriptLocaleSource::Path => web_path::generate_web_path_declaration(),
        TypeScriptLocaleSource::Cookie => web_cookie::generate_web_cookie_declaration(),
        TypeScriptLocaleSource::LocalStorage => {
            web_local_storage::generate_web_local_storage_declaration()
        }
        TypeScriptLocaleSource::AcceptLanguage => {
            web_accept_language::generate_web_accept_language_declaration()
        }
    }
}

pub fn generate_project_web_link_module(mode: super::TypeScriptLinkMode) -> Option<String> {
    match mode {
        super::TypeScriptLinkMode::Transform => {
            Some(web_link_transform::generate_typescript_web_link_transform_module())
        }
        super::TypeScriptLinkMode::Runtime => {
            Some(web_runtime_links::generate_typescript_web_runtime_links_module())
        }
        super::TypeScriptLinkMode::Manual => None,
    }
}

pub fn generate_project_web_link_declaration(mode: super::TypeScriptLinkMode) -> Option<String> {
    match mode {
        super::TypeScriptLinkMode::Transform => {
            Some(web_link_transform::generate_web_link_transform_declaration())
        }
        super::TypeScriptLinkMode::Runtime => {
            Some(web_runtime_links::generate_web_runtime_links_declaration())
        }
        super::TypeScriptLinkMode::Manual => None,
    }
}

pub fn generate_project_web_server_cookie_module() -> String {
    WEB_SERVER_COOKIE_RUNTIME.to_owned()
}

pub fn generate_project_web_server_cookie_declaration() -> String {
    WEB_SERVER_COOKIE_DECLARATIONS.to_owned()
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
