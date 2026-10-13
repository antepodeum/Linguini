//! Closed web facade: statically selected locale resolvers and shared URL/request policy.

use super::runtime_code::RuntimeCode;
use super::{TypeScriptLocaleSource, TypeScriptWebOptions};
use crate::ecmascript::{
    EcmaImport, EcmaModule, EcmaModuleOutput, EcmaNamedImport, EcmaScriptTarget, EcmaStatement,
    RenderedEcmaModule,
};

/// Compile the closed web facade as TypeScript.
pub fn compile_typescript_web_module(options: &TypeScriptWebOptions) -> RenderedEcmaModule {
    web_module(options, EcmaScriptTarget::TypeScript).render(&[])
}

/// Compile the closed web facade as checked JavaScript.
pub fn compile_javascript_web_module(options: &TypeScriptWebOptions) -> RenderedEcmaModule {
    web_module(options, EcmaScriptTarget::JavaScript).render(&[])
}

fn web_module(options: &TypeScriptWebOptions, target: EcmaScriptTarget) -> EcmaModule {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        target,
        if target.is_typescript() {
            "web.ts"
        } else {
            "web.js"
        },
        Some("web.d.ts".to_owned()),
    ));
    if !options.exclude.is_empty() {
        module.push_import(EcmaImport::named(
            "./web/routes.js",
            vec![EcmaNamedImport::new("matchesRoute", "matchesRoute")],
        ));
    }
    for source in &options.features().sources {
        let (name, path) = match source {
            TypeScriptLocaleSource::Path => ("resolvePathLocale", "path"),
            TypeScriptLocaleSource::Cookie => ("resolveCookieLocale", "cookie"),
            TypeScriptLocaleSource::LocalStorage => ("resolveLocalStorageLocale", "local-storage"),
            TypeScriptLocaleSource::AcceptLanguage => {
                ("resolveAcceptLanguageLocale", "accept-language")
            }
        };
        module.push_import(EcmaImport::named(
            format!("./web/{path}.js"),
            vec![EcmaNamedImport::new(name, name)],
        ));
    }
    let mut code = RuntimeCode::default();
    code.typed("export function createWebLocaleI18n<Locale extends string>(runtime: LinguiniLocaleRuntime<Locale>, options: LinguiniWebOptions = {}, environment: LinguiniWebEnvironment = {}): LinguiniWebLocale<Locale> {", "/** @template {string} Locale @param {LinguiniLocaleRuntime<Locale>} runtime @param {LinguiniWebOptions} [options] @param {LinguiniWebEnvironment} [environment] @returns {LinguiniWebLocale<Locale>} */\nexport function createWebLocaleI18n(runtime, options = {}, environment = {}) {");
    code.shared(
        "\n  const normalized = normalizeOptions(runtime.baseLocale, options, environment);\n",
    );
    code.typed("  const matchLocale = (locale: unknown) =>", "  /** @param {unknown} locale @returns {Locale | undefined} */\n  const matchLocale = (locale) =>");
    code.shared(
        " runtime.normalizeLocale?.(locale) ?? matchLocaleValue(runtime.locales, locale);\n",
    );
    code.typed("  const getTextDirection = (locale: Locale) =>", "  /** @param {Locale} locale @returns {TextDirection} */\n  const getTextDirection = (locale) =>");
    code.shared(
        " runtime.getTextDirection?.(locale) ?? runtime.localeDirections?.[locale] ?? \"ltr\";\n\n",
    );
    selected_locale_resolver(&mut code, options);
    code.shared("\n");
    code.typed("  function localizeUrl(url: string | URL, locale: Locale, input: Record<string, unknown> = {}) {", "  /** @param {string | URL} url @param {Locale} locale @param {Record<string, unknown>} [input] @returns {URL} */\n  function localizeUrl(url, locale, input = {}) {");
    code.shared("\n    const resolved = matchLocale(locale) ?? runtime.baseLocale;\n    const base = resolveBaseUrl(normalized, input);\n    const copy = parseUrl(url, base);\n    if (!isInternalHttpUrl(copy, base.origin)) return copy;\n    return localizeParsedUrl(copy, resolved);\n  }\n\n");
    code.typed("  function localizeParsedUrl(copy: URL, locale: Locale) {", "  /** @param {URL} copy @param {Locale} locale @returns {URL} */\n  function localizeParsedUrl(copy, locale) {");
    code.shared("\n    const path = stripBasePath(copy.pathname, normalized.environment.base);\n    // In `never` mode locale-looking segments are ordinary application paths.\n    // Do not strip them while normalizing a URL that should remain unprefixed.\n    const withoutLocale = normalized.routing.localePrefix === \"never\"\n      ? path\n      : stripLeadingLocale(path, runtime.locales);\n    copy.pathname = joinPath(normalized.environment.base, shouldPrefixLocale(normalized, locale) ? joinPath(\"/\", locale, withoutLocale) : withoutLocale);\n    return copy;\n  }\n\n");
    code.typed("  function localizeHref(href: string, locale: Locale, input: Record<string, unknown> = {}) {", "  /** @param {string} href @param {Locale} locale @param {Record<string, unknown>} [input] @returns {string} */\n  function localizeHref(href, locale, input = {}) {");
    code.shared("\n    const value = String(href);\n    const trimmed = value.trim();\n    if (!trimmed || trimmed.startsWith(\"#\") || trimmed.startsWith(\"//\")) return value;\n    const scheme = trimmed.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);\n    if (scheme && ![\"http\", \"https\"].includes(scheme[1].toLowerCase())) return value;\n\n    const base = resolveBaseUrl(normalized, input);\n    const url = parseUrl(value, base);\n    if (!isInternalHttpUrl(url, base.origin)) return value;\n\n    const resolved = matchLocale(locale) ?? runtime.baseLocale;\n    localizeParsedUrl(url, resolved);\n    if (scheme) return url.toString();\n    return `${url.pathname}${url.search}${url.hash}`;\n  }\n\n");
    code.typed("  function shouldLocalizeHref(href: string, input: Record<string, unknown> = {}) {", "  /** @param {string} href @param {Record<string, unknown>} [input] @returns {boolean} */\n  function shouldLocalizeHref(href, input = {}) {");
    code.shared("\n    if (normalized.links.mode === \"manual\") return false;\n    const value = String(href ?? \"\").trim();\n    if (!value || value.startsWith(\"#\") || value.startsWith(\"//\")) return false;\n    const scheme = value.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);\n    if (scheme && ![\"http\", \"https\"].includes(scheme[1].toLowerCase())) return false;\n");
    code.typed(
        "    let parsed: URL;\n    let base: URL;",
        "    /** @type {URL} */\n    let parsed;\n    /** @type {URL} */\n    let base;",
    );
    code.shared("\n    try {\n      base = resolveBaseUrl(normalized, input);\n      parsed = parseUrl(value, base);\n    } catch {\n      return false;\n    }\n    if (!isInternalHttpUrl(parsed, base.origin)) return false;\n    return !shouldExclude(parsed, input);\n  }\n\n");
    code.typed("  function localizeHrefAttribute(href: string, locale: Locale, input: Record<string, unknown> = {}) {", "  /** @param {string} href @param {Locale} locale @param {Record<string, unknown>} [input] @returns {string} */\n  function localizeHrefAttribute(href, locale, input = {}) {");
    code.shared("\n    return shouldLocalizeHref(href, input) ? localizeHref(href, locale, input) : href;\n  }\n\n");
    code.typed("  function shouldLocalizeLink(\n    href: string,\n    attributes: LinkLocalizationAttributes = {},\n    input: Record<string, unknown> = {},\n  ) {", "  /** @param {string} href @param {LinkLocalizationAttributes} [attributes] @param {Record<string, unknown>} [input] @returns {boolean} */\n  function shouldLocalizeLink(href, attributes = {}, input = {}) {");
    code.shared("\n    if (attributes.download || attributes.ignored) return false;\n    const relations = String(attributes.rel ?? \"\").toLowerCase().split(/\\s+/);\n    if (relations.includes(\"external\")) return false;\n    return shouldLocalizeHref(href, input);\n  }\n\n");
    code.typed("  function delocalizeUrl(url: string | URL, input: Record<string, unknown> = {}) {", "  /** @param {string | URL} url @param {Record<string, unknown>} [input] @returns {URL} */\n  function delocalizeUrl(url, input = {}) {");
    code.shared("\n    const base = resolveBaseUrl(normalized, input);\n    const copy = parseUrl(url, base);\n    if (!isInternalHttpUrl(copy, base.origin)) return copy;\n    if (normalized.routing.localePrefix === \"never\") return copy;\n    copy.pathname = joinPath(normalized.environment.base, stripLeadingLocale(stripBasePath(copy.pathname, normalized.environment.base), runtime.locales));\n    return copy;\n  }\n\n");
    code.typed("  function delocalizePathname(pathname: string, input: Record<string, unknown> = {}) {", "  /** @param {string} pathname @param {Record<string, unknown>} [input] @returns {string} */\n  function delocalizePathname(pathname, input = {}) {");
    code.shared("\n    return delocalizeUrl(pathname, input).pathname;\n  }\n\n");
    code.typed("  function alternateLinks(url: string | URL, input: Record<string, unknown> = {}) {", "  /** @param {string | URL} url @param {Record<string, unknown>} [input] @returns {AlternateLink[]} */\n  function alternateLinks(url, input = {}) {");
    code.shared("\n    const parsed = parseRuntimeUrl(url, normalized, input);\n");
    code.typed(
        "    const links: AlternateLink[] =",
        "    /** @type {AlternateLink[]} */\n    const links =",
    );
    code.shared(" runtime.locales.map((locale) => ({\n");
    code.typed(
        "      rel: \"alternate\" as const,",
        "      rel: \"alternate\",",
    );
    code.shared("\n      hreflang: locale,\n      href: localizeUrl(parsed, locale, input).toString(),\n    }));\n    links.push({ rel: \"alternate\", hreflang: \"x-default\", href: localizeUrl(parsed, runtime.baseLocale, input).toString() });\n    return links;\n  }\n\n");
    code.typed("  function htmlAttrs(locale: Locale) {", "  /** @param {Locale} locale @returns {{lang: Locale, dir: TextDirection}} */\n  function htmlAttrs(locale) {");
    code.shared("\n    const resolved = matchLocale(locale) ?? runtime.baseLocale;\n    return { lang: resolved, dir: getTextDirection(resolved) };\n  }\n\n");
    code.typed("  function getCanonicalRedirect(url: string | URL, locale: Locale, input: Record<string, unknown> = {}) {", "  /** @param {string | URL} url @param {Locale} locale @param {Record<string, unknown>} [input] @returns {string | undefined} */\n  function getCanonicalRedirect(url, locale, input = {}) {");
    code.shared("\n    if (normalized.routing.localePrefix === \"never\" || normalized.routing.canonical === \"preserve\" || shouldExclude(url, input)) return undefined;\n    const parsed = parseRuntimeUrl(url, normalized, input);\n    const canonical = localizeUrl(parsed, locale, input);\n    return canonical.pathname === parsed.pathname ? undefined : `${canonical.pathname}${canonical.search}${canonical.hash}`;\n  }\n\n");
    route_exclusion(&mut code, !options.exclude.is_empty());
    code.shared("\n");
    code.typed("  function serializeLocaleCookie(locale: Locale, input: Record<string, unknown> = {}) {", "  /** @param {Locale} locale @param {Record<string, unknown>} [input] @returns {string} */\n  function serializeLocaleCookie(locale, input = {}) {");
    code.shared("\n    const parts = [`${normalized.cookie.name}=${encodeURIComponent(locale)}`];\n    parts.push(`Max-Age=${input.maxAge ?? normalized.cookie.maxAge}`);\n    parts.push(`Path=${normalized.cookie.path}`);\n    if (normalized.cookie.domain) parts.push(`Domain=${normalized.cookie.domain}`);\n    parts.push(`SameSite=${normalized.cookie.sameSite}`);\n    if (resolveCookieSecure(normalized.cookie.secure, input)) parts.push(\"Secure\");\n    if (input.httpOnly ?? normalized.cookie.httpOnly) parts.push(\"HttpOnly\");\n    return parts.join(\"; \");\n  }\n\n");
    code.typed("  function setLocaleCookie(target: unknown, locale: Locale, input: Record<string, unknown> = {}) {", "  /** @param {unknown} target @param {Locale} locale @param {Record<string, unknown>} [input] @returns {void} */\n  function setLocaleCookie(target, locale, input = {}) {");
    code.shared("\n    const cookie = serializeLocaleCookie(locale, input);\n");
    code.typed("    const sink = target as { headers?: Pick<Headers, \"append\">; cookies?: { set(name: string, value: string, options?: Record<string, unknown>): void } };", "    const sink = /** @type {{ headers?: Pick<Headers, \"append\">; cookies?: { set(name: string, value: string, options?: Record<string, unknown>): void } }} */ (target);");
    code.shared("\n    if (sink.cookies?.set) {\n      sink.cookies.set(normalized.cookie.name, locale, {\n        path: normalized.cookie.path,\n        domain: normalized.cookie.domain,\n        maxAge: input.maxAge ?? normalized.cookie.maxAge,\n        sameSite: normalized.cookie.sameSite,\n        secure: resolveCookieSecure(normalized.cookie.secure, input),\n        httpOnly: input.httpOnly ?? normalized.cookie.httpOnly,\n      });\n    } else {\n      sink.headers?.append?.(\"set-cookie\", cookie);\n    }\n  }\n\n  return {\n    ...runtime,\n    options: normalized,\n    matchLocale,\n    resolveLocale: async (input = {}) => resolveLocaleSync(input),\n    resolveLocaleSync,\n    localizeUrl,\n    localizeHref,\n    shouldLocalizeHref,\n    shouldLocalizeLink,\n    localizeHrefAttribute,\n    delocalizeUrl,\n    delocalizePathname,\n    alternateLinks,\n    htmlAttrs,\n    getCanonicalRedirect,\n    shouldExclude,\n    setLocaleCookie,\n    serializeLocaleCookie,\n    getTextDirection,\n  };\n}\n\n");
    code.typed("export function createWebI18n<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions = {}, environment: LinguiniWebEnvironment = {}): LinguiniWeb<Locale, Linguini> {", "/** @template {string} Locale @template Linguini @param {LinguiniRuntime<Locale, Linguini>} runtime @param {LinguiniWebOptions} [options] @param {LinguiniWebEnvironment} [environment] @returns {LinguiniWeb<Locale, Linguini>} */\nexport function createWebI18n(runtime, options = {}, environment = {}) {");
    code.shared("\n  const web = createWebLocaleI18n(runtime, options, environment);\n\n");
    code.typed("  function createRequestContext(locale: Locale, contextInput: Record<string, unknown> = {}): LinguiniRequestContext<Locale, Linguini> {", "  /** @param {Locale} locale @param {Record<string, unknown>} [contextInput] @returns {LinguiniRequestContext<Locale, Linguini>} */\n  function createRequestContext(locale, contextInput = {}) {");
    code.shared("\n    const resolved = web.matchLocale(locale) ?? runtime.baseLocale;\n    const messages = runtime.createLinguini(resolved);\n    return {\n      locale: resolved,\n      baseLocale: runtime.baseLocale,\n      locales: runtime.locales,\n      direction: web.getTextDirection(resolved),\n      textDirection: web.getTextDirection(resolved),\n      lang: resolved,\n      messages,\n      l: messages,\n      htmlAttrs: web.htmlAttrs(resolved),\n      localizeHref: (href, nextLocale = resolved, input = contextInput) => web.localizeHref(href, nextLocale, input),\n      localizeUrl: (url, nextLocale = resolved, input = contextInput) => web.localizeUrl(url, nextLocale, input),\n      shouldLocalizeHref: (href, input = contextInput) => web.shouldLocalizeHref(href, input),\n      shouldLocalizeLink: (href, attributes = {}, input = contextInput) => web.shouldLocalizeLink(href, attributes, input),\n      localizeHrefAttribute: (href, nextLocale = resolved, input = contextInput) => web.localizeHrefAttribute(href, nextLocale, input),\n      delocalizeUrl: (url, input = contextInput) => web.delocalizeUrl(url, input),\n      alternateLinks: (url, input = contextInput) => web.alternateLinks(url, input),\n    };\n  }\n\n  return {\n    ...web,\n    createLinguini: runtime.createLinguini,\n    resolveRequest: async (request, input = {}) => {\n      const requestInput = inputFromRequest(request, input);\n      const locale = web.resolveLocaleSync(requestInput);\n      await runtime.prepareLinguini?.(locale);\n      return createRequestContext(locale, requestInput);\n    },\n    createRequestContext,\n  };\n}\n\n");
    super::web_request::append(&mut code);
    code.typed("function normalizeOptions(baseLocale: string, options: LinguiniWebOptions, environment: LinguiniWebEnvironment) {", "/** @param {string} baseLocale @param {LinguiniWebOptions} options @param {LinguiniWebEnvironment} environment @returns {LinguiniWebLocale<string>[\"options\"]} */\nfunction normalizeOptions(baseLocale, options, environment) {");
    code.shared("\n  const sources = options.locale?.sources ?? DEFAULT_SOURCES;\n  const base = normalizeBasePath(environment.base ?? \"\");\n  const origin = environment.origin;\n  const cookiePath = options.cookie?.path === \"auto\" || options.cookie?.path === undefined\n    ? base || \"/\"\n    : options.cookie.path;\n  const secure = options.cookie?.secure ?? \"auto\";\n  return {\n    routing: {\n      localePrefix: options.routing?.localePrefix ?? \"except-default\"");
    code.typed(" as LocalePrefixMode", "");
    code.shared(",\n      canonical: options.routing?.canonical ?? \"redirect\"");
    code.typed(" as \"redirect\" | \"preserve\"", "");
    code.shared(",\n    },\n    locale: {\n      sources,\n      switch: options.locale?.switch ?? localeSwitchFromSources(sources),\n    },\n    cookie: {\n      name: options.cookie?.name ?? \"LINGUINI_LOCALE\",\n      path: cookiePath,\n      domain: options.cookie?.domain,\n      maxAge: options.cookie?.maxAge ?? DEFAULT_COOKIE_MAX_AGE,\n      sameSite: options.cookie?.sameSite ?? \"lax\"");
    code.typed(" as \"lax\" | \"strict\" | \"none\"", "");
    code.shared(",\n      secure,\n      httpOnly: Boolean(options.cookie?.httpOnly ?? false),\n    },\n    localStorage: { key: options.localStorage?.key ?? \"LINGUINI_LOCALE\" },\n    links: { mode: options.links?.mode ?? \"runtime\"");
    code.typed(" as \"transform\" | \"runtime\" | \"manual\"", "");
    code.shared(" },\n    routes: { exclude: options.routes?.exclude ?? [] },\n    environment: { base, origin },\n    baseLocale,\n  };\n}\n\n");
    code.typed("function localeSwitchFromSources(sources: readonly LocaleSource[]): LocaleSwitchPlan {", "/** @param {readonly LocaleSource[]} sources @returns {LocaleSwitchPlan} */\nfunction localeSwitchFromSources(sources) {");
    code.shared("\n  return {\n    writesPath: sources.includes(\"path\"),\n    writesCookie: sources.includes(\"cookie\"),\n    writesLocalStorage: sources.includes(\"local-storage\"),\n  };\n}\n\n");
    super::web_url::append(&mut code);
    let mut constants = RuntimeCode::default();
    constants.typed("const DEFAULT_SOURCES = [\"path\", \"cookie\", \"accept-language\"] as const;", "/** @type {readonly LocaleSource[]} */\nconst DEFAULT_SOURCES = [\"path\", \"cookie\", \"accept-language\"];");
    constants.shared("\nconst DEFAULT_COOKIE_MAX_AGE = 60 * 60 * 24 * 365;\nconst FALLBACK_URL = \"http://localhost\";\nconst INVALID_URL_MESSAGE = \"Linguini: invalid URL\";\n\n");
    module.push_statement(EcmaStatement::generated(constants.render(target)));
    super::web_types::append(&mut module, target);
    module.push_statement(EcmaStatement::generated(code.render(target)));
    module
}

fn selected_locale_resolver(code: &mut RuntimeCode, options: &TypeScriptWebOptions) {
    code.typed("  function resolveLocaleSync(input: Record<string, unknown> = {}) {\n", "  /** @param {Record<string, unknown>} [input] @returns {Locale} */\n  function resolveLocaleSync(input = {}) {\n");
    for (index, source) in options.features().sources.iter().enumerate() {
        let binding = format!("resolved_{index}");
        let call = match source {
            TypeScriptLocaleSource::Path => "resolvePathLocale(input, normalized, runtime.locales)",
            TypeScriptLocaleSource::Cookie => "resolveCookieLocale(input, normalized, matchLocale)",
            TypeScriptLocaleSource::LocalStorage => "resolveLocalStorageLocale(input, normalized, matchLocale)",
            TypeScriptLocaleSource::AcceptLanguage => "resolveAcceptLanguageLocale(input, runtime.locales, runtime.baseLocale, matchLocale)",
        };
        code.generated(format!(
            "    const {binding} = {call};\n    if ({binding}) return {binding};\n"
        ));
    }
    code.shared("    return runtime.baseLocale;\n  }\n");
}

fn route_exclusion(code: &mut RuntimeCode, selected: bool) {
    if selected {
        code.typed("  function shouldExclude(url: string | URL, input: Record<string, unknown> = {}) {", "  /** @param {string | URL} url @param {Record<string, unknown>} [input] @returns {boolean} */\n  function shouldExclude(url, input = {}) {");
        code.shared("\n    const parsed = parseRuntimeUrl(url, normalized, input);\n    return normalized.routes.exclude.some((matcher) => matchesRoute(matcher, parsed));\n  }\n");
    } else {
        code.typed("  function shouldExclude(_url: string | URL, _input: Record<string, unknown> = {}) {\n", "  /** @param {string | URL} _url @param {Record<string, unknown>} [_input] @returns {boolean} */\n  function shouldExclude(_url, _input = {}) {\n");
        code.shared("    return false;\n  }\n");
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        web_accept_language, web_cookie, web_local_storage, web_path, web_routes, web_types,
    };
    use super::*;
    use std::path::Path;

    #[test]
    fn web_facades_share_closed_source_policy_and_public_api() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web");
        assert_eq!(
            web_types::generate_web_declaration(),
            std::fs::read_to_string(root.join("legacy/web.runtime.d.ts")).unwrap()
        );
        let cases = [
            ("default", TypeScriptWebOptions::default()),
            (
                "all",
                TypeScriptWebOptions {
                    sources: vec![
                        TypeScriptLocaleSource::Path,
                        TypeScriptLocaleSource::Cookie,
                        TypeScriptLocaleSource::LocalStorage,
                        TypeScriptLocaleSource::AcceptLanguage,
                    ],
                    exclude: vec!["/api/**".to_owned()],
                    ..Default::default()
                },
            ),
            (
                "empty",
                TypeScriptWebOptions {
                    sources: vec![],
                    ..Default::default()
                },
            ),
            (
                "path",
                TypeScriptWebOptions {
                    sources: vec![TypeScriptLocaleSource::Path],
                    ..Default::default()
                },
            ),
            (
                "cookie",
                TypeScriptWebOptions {
                    sources: vec![TypeScriptLocaleSource::Cookie],
                    ..Default::default()
                },
            ),
            (
                "storage",
                TypeScriptWebOptions {
                    sources: vec![TypeScriptLocaleSource::LocalStorage],
                    ..Default::default()
                },
            ),
            (
                "language",
                TypeScriptWebOptions {
                    sources: vec![TypeScriptLocaleSource::AcceptLanguage],
                    ..Default::default()
                },
            ),
        ];
        let mut cases = cases.to_vec();
        cases.push((
            "language-cookie",
            TypeScriptWebOptions {
                sources: vec![
                    TypeScriptLocaleSource::AcceptLanguage,
                    TypeScriptLocaleSource::Cookie,
                ],
                ..Default::default()
            },
        ));
        let mut snapshots = vec![];
        for (name, options) in cases {
            let ts = compile_typescript_web_module(&options);
            let js = compile_javascript_web_module(&options);
            let meaningful_lines = |code: &str| {
                code.lines()
                    .filter(|line| {
                        !line.trim().is_empty() && !line.starts_with("//# sourceMappingURL=")
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            if matches!(name, "default" | "all") {
                assert_eq!(
                    meaningful_lines(&ts.code),
                    meaningful_lines(
                        &std::fs::read_to_string(root.join(format!("legacy/{name}.ts"))).unwrap()
                    )
                );
            }
            assert!(!js.code.contains("resolveLocaleSource"));
            assert!(!js.code.contains("createWebI18n<"));
            for source in [
                TypeScriptLocaleSource::Path,
                TypeScriptLocaleSource::Cookie,
                TypeScriptLocaleSource::LocalStorage,
                TypeScriptLocaleSource::AcceptLanguage,
            ] {
                let (path, ts_leaf, js_leaf) = match source {
                    TypeScriptLocaleSource::Path => (
                        "path",
                        web_path::compile_typescript_web_path_module(),
                        web_path::compile_javascript_web_path_module(),
                    ),
                    TypeScriptLocaleSource::Cookie => (
                        "cookie",
                        web_cookie::compile_typescript_web_cookie_module(),
                        web_cookie::compile_javascript_web_cookie_module(),
                    ),
                    TypeScriptLocaleSource::LocalStorage => (
                        "local-storage",
                        web_local_storage::compile_typescript_web_local_storage_module(),
                        web_local_storage::compile_javascript_web_local_storage_module(),
                    ),
                    TypeScriptLocaleSource::AcceptLanguage => (
                        "accept-language",
                        web_accept_language::compile_typescript_web_accept_language_module(),
                        web_accept_language::compile_javascript_web_accept_language_module(),
                    ),
                };
                assert_eq!(
                    js.code.contains(&format!("from \"./web/{path}.js\"")),
                    options.sources.contains(&source)
                );
                if options.sources.contains(&source) {
                    snapshots.push((
                        root.join(format!("{name}/typescript/web/{path}.ts")),
                        ts_leaf.code,
                    ));
                    snapshots.push((root.join(format!("{name}/web/{path}.js")), js_leaf.code));
                }
            }
            if !options.exclude.is_empty() {
                snapshots.push((
                    root.join(format!("{name}/typescript/web/routes.ts")),
                    web_routes::compile_typescript_web_routes_module().code,
                ));
                snapshots.push((
                    root.join(format!("{name}/web/routes.js")),
                    web_routes::compile_javascript_web_routes_module().code,
                ));
            }
            snapshots.extend([
                (root.join(format!("{name}/typescript/web.ts")), ts.code),
                (
                    root.join(format!("{name}/typescript/web.d.ts")),
                    web_types::generate_web_declaration(),
                ),
                (root.join(format!("{name}/web.js")), js.code),
                (root.join(format!("{name}/web.js.map")), js.source_map),
            ]);
        }
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, code) in &snapshots {
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, code).unwrap();
            }
        }
        for (path, code) in snapshots {
            assert_eq!(
                code,
                std::fs::read_to_string(&path).unwrap(),
                "{}",
                path.display()
            );
        }
    }
}
