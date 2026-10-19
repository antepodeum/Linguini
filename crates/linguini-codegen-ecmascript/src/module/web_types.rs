//! Public web API model shared by TypeScript, JSDoc, and declaration output.

use crate::ecmascript::{EcmaModule, EcmaModuleKind, EcmaModuleOutput, EcmaStatement};

struct WebType {
    header: &'static str,
    documentation: &'static str,
    jsdoc_templates: &'static str,
    name: &'static str,
    parents: &'static str,
    alias: Option<&'static str>,
    members: &'static [WebMember],
}

struct WebMember {
    name: &'static str,
    optional: bool,
    parameters: Option<&'static str>,
    value_type: &'static str,
}

const TYPES: &[WebType] = &[
    WebType { header: "export type TextDirection =", documentation: "", jsdoc_templates: "", name: "TextDirection", parents: "", alias: Some("\"ltr\" | \"rtl\""), members: &[
    ] },
    WebType { header: "export type LocaleSource =", documentation: "", jsdoc_templates: "", name: "LocaleSource", parents: "", alias: Some("\"path\" | \"cookie\" | \"local-storage\" | \"accept-language\""), members: &[
    ] },
    WebType { header: "export type LocalePrefixMode =", documentation: "", jsdoc_templates: "", name: "LocalePrefixMode", parents: "", alias: Some("\"always\" | \"except-default\" | \"never\""), members: &[
    ] },
    WebType { header: "export interface LocaleSwitchPlan", documentation: "", jsdoc_templates: "", name: "LocaleSwitchPlan", parents: "", alias: None, members: &[
        WebMember { name: "writesPath", optional: false, parameters: None, value_type: "boolean" },
        WebMember { name: "writesCookie", optional: false, parameters: None, value_type: "boolean" },
        WebMember { name: "writesLocalStorage", optional: false, parameters: None, value_type: "boolean" },
    ] },
    WebType { header: "export interface AlternateLink", documentation: "", jsdoc_templates: "", name: "AlternateLink", parents: "", alias: None, members: &[
        WebMember { name: "rel", optional: false, parameters: None, value_type: "\"alternate\"" },
        WebMember { name: "hreflang", optional: false, parameters: None, value_type: "string" },
        WebMember { name: "href", optional: false, parameters: None, value_type: "string" },
    ] },
    WebType { header: "export interface LinkLocalizationAttributes", documentation: "", jsdoc_templates: "", name: "LinkLocalizationAttributes", parents: "", alias: None, members: &[
        WebMember { name: "download", optional: true, parameters: None, value_type: "boolean" },
        WebMember { name: "ignored", optional: true, parameters: None, value_type: "boolean" },
        WebMember { name: "rel", optional: true, parameters: None, value_type: "string | null" },
    ] },
    WebType { header: "export interface LinguiniWebOptions", documentation: "", jsdoc_templates: "", name: "LinguiniWebOptions", parents: "", alias: None, members: &[
        WebMember { name: "routing", optional: true, parameters: None, value_type: "{\n    localePrefix?: LocalePrefixMode;\n    canonical?: \"redirect\" | \"preserve\";\n  }" },
        WebMember { name: "locale", optional: true, parameters: None, value_type: "{\n    sources?: readonly LocaleSource[];\n    switch?: LocaleSwitchPlan;\n  }" },
        WebMember { name: "cookie", optional: true, parameters: None, value_type: "{\n    name?: string;\n    path?: string | \"auto\";\n    domain?: string;\n    maxAge?: number;\n    sameSite?: \"lax\" | \"strict\" | \"none\";\n    secure?: boolean | \"auto\";\n    httpOnly?: boolean;\n  }" },
        WebMember { name: "localStorage", optional: true, parameters: None, value_type: "{ key?: string }" },
        WebMember { name: "links", optional: true, parameters: None, value_type: "{ mode?: \"transform\" | \"runtime\" | \"manual\" }" },
        WebMember { name: "routes", optional: true, parameters: None, value_type: "{ exclude?: readonly (string | RegExp | ((url: URL) => boolean))[] }" },
    ] },
    WebType { header: "export interface LinguiniWebEnvironment", documentation: "", jsdoc_templates: "", name: "LinguiniWebEnvironment", parents: "", alias: None, members: &[
        WebMember { name: "base", optional: true, parameters: None, value_type: "string" },
        WebMember { name: "origin", optional: true, parameters: None, value_type: "string | URL" },
    ] },
    WebType { header: "export interface LinguiniLocaleRuntime<Locale extends string = string>", documentation: "", jsdoc_templates: "@template {string} [Locale=string]", name: "LinguiniLocaleRuntime", parents: "", alias: None, members: &[
        WebMember { name: "locales", optional: false, parameters: None, value_type: "readonly Locale[]" },
        WebMember { name: "baseLocale", optional: false, parameters: None, value_type: "Locale" },
        WebMember { name: "localeDirections", optional: true, parameters: None, value_type: "Readonly<Record<Locale, TextDirection>>" },
        WebMember { name: "normalizeLocale", optional: true, parameters: Some("locale: unknown"), value_type: "Locale | undefined" },
        WebMember { name: "getTextDirection", optional: true, parameters: Some("locale: Locale"), value_type: "TextDirection" },
    ] },
    WebType { header: "export interface LinguiniRuntime<Locale extends string = string, Linguini = unknown> extends LinguiniLocaleRuntime<Locale>", documentation: "", jsdoc_templates: "@template {string} [Locale=string] @template [Linguini=unknown]", name: "LinguiniRuntime", parents: "LinguiniLocaleRuntime<Locale>", alias: None, members: &[
        WebMember { name: "createLinguini", optional: false, parameters: Some("locale: Locale"), value_type: "Linguini" },
        WebMember { name: "prepareLinguini", optional: true, parameters: Some("locale: Locale"), value_type: "Promise<Linguini>" },
    ] },
    WebType { header: "export interface LinguiniLocaleRequestContext<Locale extends string = string>", documentation: "/**\n * Localization and delocalization leave external and non-HTTP URLs unchanged. Explicit URL\n * operations throw `TypeError(\"Linguini: invalid URL\")` for malformed input; link-safety helpers\n * fail closed and preserve the original href instead.\n */\n", jsdoc_templates: "@template {string} [Locale=string]", name: "LinguiniLocaleRequestContext", parents: "", alias: None, members: &[
        WebMember { name: "locale", optional: false, parameters: None, value_type: "Locale" },
        WebMember { name: "baseLocale", optional: false, parameters: None, value_type: "Locale" },
        WebMember { name: "locales", optional: false, parameters: None, value_type: "readonly Locale[]" },
        WebMember { name: "direction", optional: false, parameters: None, value_type: "TextDirection" },
        WebMember { name: "textDirection", optional: false, parameters: None, value_type: "TextDirection" },
        WebMember { name: "lang", optional: false, parameters: None, value_type: "Locale" },
        WebMember { name: "htmlAttrs", optional: false, parameters: None, value_type: "{ lang: Locale; dir: TextDirection }" },
        WebMember { name: "localizeHref", optional: false, parameters: Some("href: string, locale?: Locale, input?: Record<string, unknown>"), value_type: "string" },
        WebMember { name: "localizeUrl", optional: false, parameters: Some("url: string | URL, locale?: Locale, input?: Record<string, unknown>"), value_type: "URL" },
        WebMember { name: "shouldLocalizeHref", optional: false, parameters: Some("href: string, input?: Record<string, unknown>"), value_type: "boolean" },
        WebMember { name: "shouldLocalizeLink", optional: false, parameters: Some("href: string, attributes?: LinkLocalizationAttributes, input?: Record<string, unknown>"), value_type: "boolean" },
        WebMember { name: "localizeHrefAttribute", optional: false, parameters: Some("href: string, locale?: Locale, input?: Record<string, unknown>"), value_type: "string" },
        WebMember { name: "delocalizeUrl", optional: false, parameters: Some("url: string | URL, input?: Record<string, unknown>"), value_type: "URL" },
        WebMember { name: "alternateLinks", optional: false, parameters: Some("url: string | URL, input?: Record<string, unknown>"), value_type: "AlternateLink[]" },
    ] },
    WebType { header: "export interface LinguiniRequestContext<Locale extends string = string, Linguini = unknown>\n  extends LinguiniLocaleRequestContext<Locale>", documentation: "", jsdoc_templates: "@template {string} [Locale=string] @template [Linguini=unknown]", name: "LinguiniRequestContext", parents: "LinguiniLocaleRequestContext<Locale>", alias: None, members: &[
        WebMember { name: "messages", optional: false, parameters: None, value_type: "Linguini" },
        WebMember { name: "l", optional: false, parameters: None, value_type: "Linguini" },
    ] },
    WebType { header: "export interface LinguiniWebLocale<Locale extends string = string> extends LinguiniLocaleRuntime<Locale>", documentation: "/**\n * Localization and delocalization leave external and non-HTTP URLs unchanged. Explicit URL\n * operations throw `TypeError(\"Linguini: invalid URL\")` for malformed input; link-safety helpers\n * fail closed and preserve the original href instead.\n */\n", jsdoc_templates: "@template {string} [Locale=string]", name: "LinguiniWebLocale", parents: "LinguiniLocaleRuntime<Locale>", alias: None, members: &[
        WebMember { name: "options", optional: false, parameters: None, value_type: "{\n    routing: { localePrefix: LocalePrefixMode; canonical: \"redirect\" | \"preserve\" };\n    locale: { sources: readonly LocaleSource[]; switch: LocaleSwitchPlan };\n    cookie: { name: string; path: string; domain?: string; maxAge: number; sameSite: \"lax\" | \"strict\" | \"none\"; secure: boolean | \"auto\"; httpOnly: boolean };\n    localStorage: { key: string };\n    links: { mode: \"transform\" | \"runtime\" | \"manual\" };\n    routes: { exclude: readonly (string | RegExp | ((url: URL) => boolean))[] };\n    environment: { base: string; origin?: string | URL };\n    baseLocale: string;\n  }" },
        WebMember { name: "matchLocale", optional: false, parameters: Some("locale: unknown"), value_type: "Locale | undefined" },
        WebMember { name: "resolveLocale", optional: false, parameters: Some("input?: Record<string, unknown>"), value_type: "Promise<Locale>" },
        WebMember { name: "resolveLocaleSync", optional: false, parameters: Some("input?: Record<string, unknown>"), value_type: "Locale" },
        WebMember { name: "localizeUrl", optional: false, parameters: Some("url: string | URL, locale: Locale, input?: Record<string, unknown>"), value_type: "URL" },
        WebMember { name: "localizeHref", optional: false, parameters: Some("href: string, locale: Locale, input?: Record<string, unknown>"), value_type: "string" },
        WebMember { name: "shouldLocalizeHref", optional: false, parameters: Some("href: string, input?: Record<string, unknown>"), value_type: "boolean" },
        WebMember { name: "shouldLocalizeLink", optional: false, parameters: Some("href: string, attributes?: LinkLocalizationAttributes, input?: Record<string, unknown>"), value_type: "boolean" },
        WebMember { name: "localizeHrefAttribute", optional: false, parameters: Some("href: string, locale: Locale, input?: Record<string, unknown>"), value_type: "string" },
        WebMember { name: "delocalizeUrl", optional: false, parameters: Some("url: string | URL, input?: Record<string, unknown>"), value_type: "URL" },
        WebMember { name: "delocalizePathname", optional: false, parameters: Some("pathname: string, input?: Record<string, unknown>"), value_type: "string" },
        WebMember { name: "alternateLinks", optional: false, parameters: Some("url: string | URL, input?: Record<string, unknown>"), value_type: "AlternateLink[]" },
        WebMember { name: "htmlAttrs", optional: false, parameters: Some("locale: Locale"), value_type: "{ lang: Locale; dir: TextDirection }" },
        WebMember { name: "getTextDirection", optional: false, parameters: Some("locale: Locale"), value_type: "TextDirection" },
        WebMember { name: "getCanonicalRedirect", optional: false, parameters: Some("url: string | URL, locale: Locale, input?: Record<string, unknown>"), value_type: "string | undefined" },
        WebMember { name: "shouldExclude", optional: false, parameters: Some("url: string | URL, input?: Record<string, unknown>"), value_type: "boolean" },
        WebMember { name: "setLocaleCookie", optional: false, parameters: Some("target: unknown, locale: Locale, input?: Record<string, unknown>"), value_type: "void" },
        WebMember { name: "serializeLocaleCookie", optional: false, parameters: Some("locale: Locale, input?: Record<string, unknown>"), value_type: "string" },
    ] },
    WebType { header: "export interface LinguiniWeb<Locale extends string = string, Linguini = unknown> extends LinguiniWebLocale<Locale>", documentation: "", jsdoc_templates: "@template {string} [Locale=string] @template [Linguini=unknown]", name: "LinguiniWeb", parents: "LinguiniWebLocale<Locale>", alias: None, members: &[
        WebMember { name: "createLinguini", optional: false, parameters: Some("locale: Locale"), value_type: "Linguini" },
        WebMember { name: "resolveRequest", optional: false, parameters: Some("request: Request, input?: Record<string, unknown>"), value_type: "Promise<LinguiniRequestContext<Locale, Linguini>>" },
        WebMember { name: "createRequestContext", optional: false, parameters: Some("locale: Locale, input?: Record<string, unknown>"), value_type: "LinguiniRequestContext<Locale, Linguini>" },
    ] },
];

impl WebType {
    fn context(&self) -> minijinja::Value {
        let members = self.members.iter().map(|member| minijinja::context! { name => member.name, optional => member.optional, parameters => member.parameters, value_type => member.value_type }).collect::<Vec<_>>();
        minijinja::context! {
            header => self.header, documentation => self.documentation,
            jsdoc_templates => self.jsdoc_templates, name => self.name,
            parents => self.parents, alias => self.alias, members,
        }
    }
    fn declaration(&self) -> String {
        crate::templates::render("web/type_declaration", self.context())
    }
    fn jsdoc(&self) -> String {
        crate::templates::render("web/type_jsdoc", self.context())
    }
}

pub(super) fn append(module: &mut EcmaModule) {
    for item in TYPES {
        module.push_statement(EcmaStatement::documentation(item.jsdoc(), None));
        if item.alias.is_none() {
            module.push_statement(EcmaStatement::documentation("", None));
        }
    }
}

pub fn generate_web_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaModuleKind::Declaration,
        "web.d.ts",
        None,
    ));
    for (position, index) in [0, 1, 2, 3, 8, 9, 6, 7, 4, 5, 10, 11, 12, 13]
        .into_iter()
        .enumerate()
    {
        module.push_statement(EcmaStatement::type_declaration(
            TYPES[index].declaration(),
            None,
        ));
        if position >= 2 {
            module.push_statement(EcmaStatement::documentation("", None));
        }
    }
    module.push_statement(EcmaStatement::type_declaration(
        crate::templates::render(
            "web/factory_declaration",
            minijinja::context! { locale_only => true },
        ),
        None,
    ));
    module.push_statement(EcmaStatement::documentation("", None));
    module.push_statement(EcmaStatement::type_declaration(
        crate::templates::render(
            "web/factory_declaration",
            minijinja::context! { locale_only => false },
        ),
        None,
    ));
    module.push_statement(EcmaStatement::documentation("", None));
    module.render_code().trim_end().to_owned() + "\n"
}
