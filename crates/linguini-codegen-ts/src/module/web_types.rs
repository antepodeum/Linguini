//! Public web API model shared by TypeScript, JSDoc, and declaration output.

use crate::ecmascript::{EcmaModule, EcmaModuleOutput, EcmaScriptTarget, EcmaStatement};

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
    source_type: Option<&'static str>,
}

const TYPES: &[WebType] = &[
    WebType { header: "export type TextDirection =", documentation: "", jsdoc_templates: "", name: "TextDirection", parents: "", alias: Some("\"ltr\" | \"rtl\""), members: &[
    ] },
    WebType { header: "export type LocaleSource =", documentation: "", jsdoc_templates: "", name: "LocaleSource", parents: "", alias: Some("\"path\" | \"cookie\" | \"local-storage\" | \"accept-language\""), members: &[
    ] },
    WebType { header: "export type LocalePrefixMode =", documentation: "", jsdoc_templates: "", name: "LocalePrefixMode", parents: "", alias: Some("\"always\" | \"except-default\" | \"never\""), members: &[
    ] },
    WebType { header: "export interface LocaleSwitchPlan", documentation: "", jsdoc_templates: "", name: "LocaleSwitchPlan", parents: "", alias: None, members: &[
        WebMember { name: "writesPath", optional: false, parameters: None, value_type: "boolean", source_type: None },
        WebMember { name: "writesCookie", optional: false, parameters: None, value_type: "boolean", source_type: None },
        WebMember { name: "writesLocalStorage", optional: false, parameters: None, value_type: "boolean", source_type: None },
    ] },
    WebType { header: "export interface AlternateLink", documentation: "", jsdoc_templates: "", name: "AlternateLink", parents: "", alias: None, members: &[
        WebMember { name: "rel", optional: false, parameters: None, value_type: "\"alternate\"", source_type: None },
        WebMember { name: "hreflang", optional: false, parameters: None, value_type: "string", source_type: None },
        WebMember { name: "href", optional: false, parameters: None, value_type: "string", source_type: None },
    ] },
    WebType { header: "export interface LinkLocalizationAttributes", documentation: "", jsdoc_templates: "", name: "LinkLocalizationAttributes", parents: "", alias: None, members: &[
        WebMember { name: "download", optional: true, parameters: None, value_type: "boolean", source_type: None },
        WebMember { name: "ignored", optional: true, parameters: None, value_type: "boolean", source_type: None },
        WebMember { name: "rel", optional: true, parameters: None, value_type: "string | null", source_type: None },
    ] },
    WebType { header: "export interface LinguiniWebOptions", documentation: "", jsdoc_templates: "", name: "LinguiniWebOptions", parents: "", alias: None, members: &[
        WebMember { name: "routing", optional: true, parameters: None, value_type: "{\n    localePrefix?: LocalePrefixMode;\n    canonical?: \"redirect\" | \"preserve\";\n  }", source_type: None },
        WebMember { name: "locale", optional: true, parameters: None, value_type: "{\n    sources?: readonly LocaleSource[];\n    switch?: LocaleSwitchPlan;\n  }", source_type: None },
        WebMember { name: "cookie", optional: true, parameters: None, value_type: "{\n    name?: string;\n    path?: string | \"auto\";\n    domain?: string;\n    maxAge?: number;\n    sameSite?: \"lax\" | \"strict\" | \"none\";\n    secure?: boolean | \"auto\";\n    httpOnly?: boolean;\n  }", source_type: None },
        WebMember { name: "localStorage", optional: true, parameters: None, value_type: "{ key?: string }", source_type: None },
        WebMember { name: "links", optional: true, parameters: None, value_type: "{ mode?: \"transform\" | \"runtime\" | \"manual\" }", source_type: None },
        WebMember { name: "routes", optional: true, parameters: None, value_type: "{ exclude?: readonly (string | RegExp | ((url: URL) => boolean))[] }", source_type: None },
    ] },
    WebType { header: "export interface LinguiniWebEnvironment", documentation: "", jsdoc_templates: "", name: "LinguiniWebEnvironment", parents: "", alias: None, members: &[
        WebMember { name: "base", optional: true, parameters: None, value_type: "string", source_type: None },
        WebMember { name: "origin", optional: true, parameters: None, value_type: "string | URL", source_type: None },
    ] },
    WebType { header: "export interface LinguiniLocaleRuntime<Locale extends string = string>", documentation: "", jsdoc_templates: "@template {string} [Locale=string]", name: "LinguiniLocaleRuntime", parents: "", alias: None, members: &[
        WebMember { name: "locales", optional: false, parameters: None, value_type: "readonly Locale[]", source_type: None },
        WebMember { name: "baseLocale", optional: false, parameters: None, value_type: "Locale", source_type: None },
        WebMember { name: "localeDirections", optional: true, parameters: None, value_type: "Readonly<Record<Locale, TextDirection>>", source_type: None },
        WebMember { name: "normalizeLocale", optional: true, parameters: Some("locale: unknown"), value_type: "Locale | undefined", source_type: None },
        WebMember { name: "getTextDirection", optional: true, parameters: Some("locale: Locale"), value_type: "TextDirection", source_type: None },
    ] },
    WebType { header: "export interface LinguiniRuntime<Locale extends string = string, Linguini = unknown> extends LinguiniLocaleRuntime<Locale>", documentation: "", jsdoc_templates: "@template {string} [Locale=string] @template [Linguini=unknown]", name: "LinguiniRuntime", parents: "LinguiniLocaleRuntime<Locale>", alias: None, members: &[
        WebMember { name: "createLinguini", optional: false, parameters: Some("locale: Locale"), value_type: "Linguini", source_type: None },
        WebMember { name: "prepareLinguini", optional: true, parameters: Some("locale: Locale"), value_type: "Promise<Linguini>", source_type: None },
    ] },
    WebType { header: "export interface LinguiniLocaleRequestContext<Locale extends string = string>", documentation: "/**\n * Localization and delocalization leave external and non-HTTP URLs unchanged. Explicit URL\n * operations throw `TypeError(\"Linguini: invalid URL\")` for malformed input; link-safety helpers\n * fail closed and preserve the original href instead.\n */\n", jsdoc_templates: "@template {string} [Locale=string]", name: "LinguiniLocaleRequestContext", parents: "", alias: None, members: &[
        WebMember { name: "locale", optional: false, parameters: None, value_type: "Locale", source_type: None },
        WebMember { name: "baseLocale", optional: false, parameters: None, value_type: "Locale", source_type: None },
        WebMember { name: "locales", optional: false, parameters: None, value_type: "readonly Locale[]", source_type: None },
        WebMember { name: "direction", optional: false, parameters: None, value_type: "TextDirection", source_type: None },
        WebMember { name: "textDirection", optional: false, parameters: None, value_type: "TextDirection", source_type: None },
        WebMember { name: "lang", optional: false, parameters: None, value_type: "Locale", source_type: None },
        WebMember { name: "htmlAttrs", optional: false, parameters: None, value_type: "{ lang: Locale; dir: TextDirection }", source_type: None },
        WebMember { name: "localizeHref", optional: false, parameters: Some("href: string, locale?: Locale, input?: Record<string, unknown>"), value_type: "string", source_type: None },
        WebMember { name: "localizeUrl", optional: false, parameters: Some("url: string | URL, locale?: Locale, input?: Record<string, unknown>"), value_type: "URL", source_type: None },
        WebMember { name: "shouldLocalizeHref", optional: false, parameters: Some("href: string, input?: Record<string, unknown>"), value_type: "boolean", source_type: None },
        WebMember { name: "shouldLocalizeLink", optional: false, parameters: Some("href: string, attributes?: LinkLocalizationAttributes, input?: Record<string, unknown>"), value_type: "boolean", source_type: None },
        WebMember { name: "localizeHrefAttribute", optional: false, parameters: Some("href: string, locale?: Locale, input?: Record<string, unknown>"), value_type: "string", source_type: None },
        WebMember { name: "delocalizeUrl", optional: false, parameters: Some("url: string | URL, input?: Record<string, unknown>"), value_type: "URL", source_type: None },
        WebMember { name: "alternateLinks", optional: false, parameters: Some("url: string | URL, input?: Record<string, unknown>"), value_type: "AlternateLink[]", source_type: None },
    ] },
    WebType { header: "export interface LinguiniRequestContext<Locale extends string = string, Linguini = unknown>\n  extends LinguiniLocaleRequestContext<Locale>", documentation: "", jsdoc_templates: "@template {string} [Locale=string] @template [Linguini=unknown]", name: "LinguiniRequestContext", parents: "LinguiniLocaleRequestContext<Locale>", alias: None, members: &[
        WebMember { name: "messages", optional: false, parameters: None, value_type: "Linguini", source_type: None },
        WebMember { name: "l", optional: false, parameters: None, value_type: "Linguini", source_type: None },
    ] },
    WebType { header: "export interface LinguiniWebLocale<Locale extends string = string> extends LinguiniLocaleRuntime<Locale>", documentation: "/**\n * Localization and delocalization leave external and non-HTTP URLs unchanged. Explicit URL\n * operations throw `TypeError(\"Linguini: invalid URL\")` for malformed input; link-safety helpers\n * fail closed and preserve the original href instead.\n */\n", jsdoc_templates: "@template {string} [Locale=string]", name: "LinguiniWebLocale", parents: "LinguiniLocaleRuntime<Locale>", alias: None, members: &[
        WebMember { name: "options", optional: false, parameters: None, value_type: "{\n    routing: { localePrefix: LocalePrefixMode; canonical: \"redirect\" | \"preserve\" };\n    locale: { sources: readonly LocaleSource[]; switch: LocaleSwitchPlan };\n    cookie: { name: string; path: string; domain?: string; maxAge: number; sameSite: \"lax\" | \"strict\" | \"none\"; secure: boolean | \"auto\"; httpOnly: boolean };\n    localStorage: { key: string };\n    links: { mode: \"transform\" | \"runtime\" | \"manual\" };\n    routes: { exclude: readonly (string | RegExp | ((url: URL) => boolean))[] };\n    environment: { base: string; origin?: string | URL };\n    baseLocale: string;\n  }", source_type: Some("ReturnType<typeof normalizeOptions>") },
        WebMember { name: "matchLocale", optional: false, parameters: Some("locale: unknown"), value_type: "Locale | undefined", source_type: None },
        WebMember { name: "resolveLocale", optional: false, parameters: Some("input?: Record<string, unknown>"), value_type: "Promise<Locale>", source_type: None },
        WebMember { name: "resolveLocaleSync", optional: false, parameters: Some("input?: Record<string, unknown>"), value_type: "Locale", source_type: None },
        WebMember { name: "localizeUrl", optional: false, parameters: Some("url: string | URL, locale: Locale, input?: Record<string, unknown>"), value_type: "URL", source_type: None },
        WebMember { name: "localizeHref", optional: false, parameters: Some("href: string, locale: Locale, input?: Record<string, unknown>"), value_type: "string", source_type: None },
        WebMember { name: "shouldLocalizeHref", optional: false, parameters: Some("href: string, input?: Record<string, unknown>"), value_type: "boolean", source_type: None },
        WebMember { name: "shouldLocalizeLink", optional: false, parameters: Some("href: string, attributes?: LinkLocalizationAttributes, input?: Record<string, unknown>"), value_type: "boolean", source_type: None },
        WebMember { name: "localizeHrefAttribute", optional: false, parameters: Some("href: string, locale: Locale, input?: Record<string, unknown>"), value_type: "string", source_type: None },
        WebMember { name: "delocalizeUrl", optional: false, parameters: Some("url: string | URL, input?: Record<string, unknown>"), value_type: "URL", source_type: None },
        WebMember { name: "delocalizePathname", optional: false, parameters: Some("pathname: string, input?: Record<string, unknown>"), value_type: "string", source_type: None },
        WebMember { name: "alternateLinks", optional: false, parameters: Some("url: string | URL, input?: Record<string, unknown>"), value_type: "AlternateLink[]", source_type: None },
        WebMember { name: "htmlAttrs", optional: false, parameters: Some("locale: Locale"), value_type: "{ lang: Locale; dir: TextDirection }", source_type: None },
        WebMember { name: "getTextDirection", optional: false, parameters: Some("locale: Locale"), value_type: "TextDirection", source_type: None },
        WebMember { name: "getCanonicalRedirect", optional: false, parameters: Some("url: string | URL, locale: Locale, input?: Record<string, unknown>"), value_type: "string | undefined", source_type: None },
        WebMember { name: "shouldExclude", optional: false, parameters: Some("url: string | URL, input?: Record<string, unknown>"), value_type: "boolean", source_type: None },
        WebMember { name: "setLocaleCookie", optional: false, parameters: Some("target: unknown, locale: Locale, input?: Record<string, unknown>"), value_type: "void", source_type: None },
        WebMember { name: "serializeLocaleCookie", optional: false, parameters: Some("locale: Locale, input?: Record<string, unknown>"), value_type: "string", source_type: None },
    ] },
    WebType { header: "export interface LinguiniWeb<Locale extends string = string, Linguini = unknown> extends LinguiniWebLocale<Locale>", documentation: "", jsdoc_templates: "@template {string} [Locale=string] @template [Linguini=unknown]", name: "LinguiniWeb", parents: "LinguiniWebLocale<Locale>", alias: None, members: &[
        WebMember { name: "createLinguini", optional: false, parameters: Some("locale: Locale"), value_type: "Linguini", source_type: None },
        WebMember { name: "resolveRequest", optional: false, parameters: Some("request: Request, input?: Record<string, unknown>"), value_type: "Promise<LinguiniRequestContext<Locale, Linguini>>", source_type: None },
        WebMember { name: "createRequestContext", optional: false, parameters: Some("locale: Locale, input?: Record<string, unknown>"), value_type: "LinguiniRequestContext<Locale, Linguini>", source_type: None },
    ] },
];

impl WebType {
    fn typescript(&self, source: bool) -> String {
        let mut output = self.documentation.to_owned();
        output.push_str(self.header);
        if let Some(alias) = self.alias {
            output.push(' ');
            output.push_str(alias);
            output.push_str(";\n");
        } else {
            output.push_str(" {\n");
            for member in self.members {
                output.push_str("  ");
                output.push_str(member.name);
                if member.optional {
                    output.push('?');
                }
                if let Some(parameters) = member.parameters {
                    output.push('(');
                    output.push_str(parameters);
                    output.push(')');
                }
                output.push_str(": ");
                output.push_str(if source {
                    member.source_type.unwrap_or(member.value_type)
                } else {
                    member.value_type
                });
                output.push_str(";\n");
            }
            output.push_str("}\n");
        }
        output
    }

    fn jsdoc(&self) -> String {
        let mut body = if let Some(alias) = self.alias {
            alias.to_owned()
        } else {
            "{ ".to_owned()
        };
        if self.alias.is_none() {
            for (index, member) in self.members.iter().enumerate() {
                if index > 0 {
                    body.push_str("; ");
                }
                body.push_str(member.name);
                if member.optional {
                    body.push('?');
                }
                body.push_str(": ");
                if let Some(parameters) = member.parameters {
                    body.push('(');
                    body.push_str(parameters);
                    body.push_str(") => ");
                }
                body.push_str(member.value_type);
            }
            body.push_str(" }");
        }
        if !self.parents.is_empty() {
            body = format!("{} & {body}", self.parents);
        }
        format!(
            "{}/** {} @typedef {{{body}}} {} */\n",
            self.documentation, self.jsdoc_templates, self.name
        )
    }
}

pub(super) fn append(module: &mut EcmaModule, target: EcmaScriptTarget) {
    for item in TYPES {
        module.push_statement(if target.is_typescript() {
            EcmaStatement::type_declaration(item.typescript(true), None)
        } else {
            EcmaStatement::documentation(item.jsdoc(), None)
        });
        if item.alias.is_none() {
            module.push_statement(EcmaStatement::documentation("", None));
        }
    }
}

pub(super) fn generate_web_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web.d.ts",
        None,
    ));
    for (position, index) in [0, 1, 2, 3, 8, 9, 6, 7, 4, 5, 10, 11, 12, 13]
        .into_iter()
        .enumerate()
    {
        module.push_statement(EcmaStatement::type_declaration(
            TYPES[index].typescript(false),
            None,
        ));
        if position >= 2 {
            module.push_statement(EcmaStatement::documentation("", None));
        }
    }
    module.push_statement(EcmaStatement::type_declaration("export declare function createWebLocaleI18n<Locale extends string>(runtime: LinguiniLocaleRuntime<Locale>, options?: LinguiniWebOptions, environment?: LinguiniWebEnvironment): LinguiniWebLocale<Locale>;\n", None));
    module.push_statement(EcmaStatement::documentation("", None));
    module.push_statement(EcmaStatement::type_declaration("export declare function createWebI18n<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options?: LinguiniWebOptions, environment?: LinguiniWebEnvironment): LinguiniWeb<Locale, Linguini>;\n", None));
    module.push_statement(EcmaStatement::documentation("", None));
    module.render_code().trim_end().to_owned() + "\n"
}
