use super::names::{escape_string, property_key, safe_identifier};
use super::{TypeScriptLocaleModule, TypeScriptWebOptions};
use linguini_cldr::built_in_text_direction;

pub fn generate_project_index(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    let mut output = String::new();

    for locale in locales {
        output.push_str(&format!(
            "import {} from \"./locales/{}\";\n",
            locale_identifier(&locale.locale),
            escape_string(&locale.locale)
        ));
    }

    output.push('\n');
    output.push_str("export const locales = [");
    output.push_str(&locale_literals(locales).join(", "));
    output.push_str("] as const;\n");
    output.push_str("export const baseLocale = ");
    output.push_str(&base_locale_literal(locales, base_locale));
    output.push_str(";\n\n");
    output.push_str("export const localeDirections = {\n");
    for locale in locales {
        output.push_str(&format!(
            "  {}: \"{}\",\n",
            property_key(&locale.locale),
            locale_direction(&locale.locale)
        ));
    }
    output.push_str("} as const;\n\n");
    output.push_str("export const localeModules = {\n");
    for locale in locales {
        output.push_str(&format!(
            "  {}: {},\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("} as const;\n\n");
    output.push_str("export const localeLoaders = {\n");
    for locale in locales {
        output.push_str(&format!(
            "  {}: () => Promise.resolve({}),\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("} as const;\n\n");
    output.push_str("type LinguiniLanguage = keyof typeof localeModules;\n");
    output.push_str("export type Locale = (typeof locales)[number];\n");
    output.push_str("export type TextDirection = \"ltr\" | \"rtl\";\n");
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str("type LinguiniLanguageInput = LinguiniLanguage;\n\n");
    push_runtime_types(&mut output);
    output
        .push_str("export function createLinguini(language: LinguiniLanguageInput): Linguini {\n");
    output.push_str("  const locale = normalizeLocale(language) ?? baseLocale;\n");
    output.push_str("  return localeModules[locale as LinguiniLanguage];\n");
    output.push_str("}\n\n");
    output.push_str("export function createLinguiniProvider(options: LinguiniProviderOptions = {}): Linguini {\n");
    output.push_str(
        "  const resolve = options.getLocale ?? options.resolveLanguage ?? (() => baseLocale);\n",
    );
    output.push_str("  return new Proxy({} as Linguini, {\n");
    output.push_str("    get(_target, property) {\n");
    output.push_str("      return createLinguini(resolve())[property as keyof Linguini];\n");
    output.push_str("    },\n");
    output.push_str("  });\n");
    output.push_str("}\n\n");
    output.push_str("export function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);\n");
    output.push_str("}): Linguini {\n");
    output.push_str("  if (typeof options.language === \"function\") {\n");
    output.push_str("    return createLinguiniProvider({ resolveLanguage: options.language });\n");
    output.push_str("  }\n");
    output.push_str("  return createLinguini(options.language);\n");
    output.push_str("}\n\n");
    output.push_str("export const lgl: Linguini = createLinguini(baseLocale);\n");
    push_runtime_functions(&mut output);
    output
}

pub fn generate_project_index_declaration(
    locales: &[TypeScriptLocaleModule],
    base_locale: Option<&str>,
) -> String {
    let mut output = String::new();

    for locale in locales {
        output.push_str(&format!(
            "import {} from \"./locales/{}\";\n",
            locale_identifier(&locale.locale),
            escape_string(&locale.locale)
        ));
    }

    output.push('\n');
    output.push_str("export declare const locales: readonly [");
    output.push_str(&locale_literals(locales).join(", "));
    output.push_str("];\n");
    output.push_str("export declare const baseLocale: ");
    output.push_str(&base_locale_literal(locales, base_locale));
    output.push_str(";\n\n");
    output.push_str("export declare const localeDirections: {\n");
    for locale in locales {
        output.push_str(&format!(
            "  readonly {}: \"{}\";\n",
            property_key(&locale.locale),
            locale_direction(&locale.locale)
        ));
    }
    output.push_str("};\n\n");
    output.push_str("export declare const localeModules: {\n");
    for locale in locales {
        output.push_str(&format!(
            "  readonly {}: typeof {};\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("};\n\n");
    output.push_str("export declare const localeLoaders: {\n");
    for locale in locales {
        output.push_str(&format!(
            "  readonly {}: () => Promise<typeof {}>;\n",
            property_key(&locale.locale),
            locale_identifier(&locale.locale)
        ));
    }
    output.push_str("};\n\n");
    output.push_str("type LinguiniLanguage = keyof typeof localeModules;\n");
    output.push_str("export type Locale = (typeof locales)[number];\n");
    output.push_str("export type TextDirection = \"ltr\" | \"rtl\";\n");
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str("type LinguiniLanguageInput = LinguiniLanguage;\n\n");
    push_runtime_declarations(&mut output);
    output.push_str(
        "export declare function createLinguini(language: LinguiniLanguageInput): Linguini;\n\n",
    );
    output.push_str(
        "export declare function createLinguiniProvider(options?: LinguiniProviderOptions): Linguini;\n\n",
    );
    output.push_str("export declare function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);\n");
    output.push_str("}): Linguini;\n\n");
    output.push_str("export declare const lgl: Linguini;\n");
    output.push('\n');
    push_runtime_function_declarations(&mut output);
    output
}

pub fn generate_project_svelte_module(options: &TypeScriptWebOptions) -> String {
    let mut output = String::new();
    output.push_str("import { createLinguiniRune } from \"@antepod/linguini-sveltekit/client\";\n");
    output.push_str("import * as runtime from \"./index\";\n\n");
    output.push_str("export const linguini = createLinguiniRune(runtime, ");
    output.push_str(&web_options_literal(options));
    output.push_str(");\n");
    output.push_str("export const l = linguini.l;\n");
    output.push_str("export const messages = linguini.messages;\n");
    output.push_str("export const setLocale = linguini.setLocale;\n");
    output.push_str("export const localizeHref = linguini.localizeHref;\n");
    output.push_str("export const localizeUrl = linguini.localizeUrl;\n");
    output.push_str("export const delocalizeUrl = linguini.delocalizeUrl;\n");
    output.push_str("export const alternateLinks = linguini.alternateLinks;\n");
    output
}

pub fn generate_project_svelte_declaration() -> String {
    let mut output = String::new();
    output.push_str("import type { LinguiniRune } from \"@antepod/linguini-sveltekit/client\";\n");
    output.push_str("import type { Locale, Linguini, TextDirection } from \"./index\";\n\n");
    output.push_str("export declare const linguini: LinguiniRune<Locale, Linguini>;\n");
    output.push_str("export declare const l: Linguini;\n");
    output.push_str("export declare const messages: Linguini;\n");
    output.push_str(
        "export declare const setLocale: LinguiniRune<Locale, Linguini>[\"setLocale\"];\n",
    );
    output.push_str(
        "export declare const localizeHref: LinguiniRune<Locale, Linguini>[\"localizeHref\"];\n",
    );
    output.push_str(
        "export declare const localizeUrl: LinguiniRune<Locale, Linguini>[\"localizeUrl\"];\n",
    );
    output.push_str(
        "export declare const delocalizeUrl: LinguiniRune<Locale, Linguini>[\"delocalizeUrl\"];\n",
    );
    output.push_str("export declare const alternateLinks: LinguiniRune<Locale, Linguini>[\"alternateLinks\"];\n");
    output
}

pub fn generate_project_sveltekit_module(options: &TypeScriptWebOptions) -> String {
    let mut output = String::new();
    output.push_str("import { createHandle, createLoad, createReroute } from \"@antepod/linguini-sveltekit/server\";\n");
    output.push_str("import * as runtime from \"./index\";\n\n");
    output.push_str("const options = ");
    output.push_str(&web_options_literal(options));
    output.push_str(";\n\n");
    output.push_str("export const handle = createHandle(runtime, options);\n");
    output.push_str("export const reroute = createReroute(runtime, options);\n");
    output.push_str("export const load = createLoad();\n");
    output
}

pub fn generate_project_sveltekit_declaration() -> String {
    let mut output = String::new();
    output.push_str("import type { Handle, Reroute, ServerLoad } from \"@sveltejs/kit\";\n");
    output.push_str("import type { LinguiniRequestContext } from \"@antepod/linguini-web\";\n");
    output.push_str(
        "import type { SerializedLinguiniContext } from \"@antepod/linguini-sveltekit/server\";\n",
    );
    output.push_str("import type { Locale, Linguini, TextDirection } from \"./index\";\n\n");
    output.push_str("export declare const handle: Handle;\n");
    output.push_str("export declare const reroute: Reroute;\n");
    output.push_str("export declare const load: ServerLoad;\n\n");
    output.push_str("declare global {\n");
    output.push_str("  namespace App {\n");
    output.push_str("    interface Locals {\n");
    output.push_str("      linguini: LinguiniRequestContext<Locale, Linguini>;\n");
    output.push_str("      locale: Locale;\n");
    output.push_str("      direction: TextDirection;\n");
    output.push_str("      l: Linguini;\n");
    output.push_str("    }\n");
    output.push_str("    interface PageData {\n");
    output.push_str("      linguini?: SerializedLinguiniContext<Locale>;\n");
    output.push_str("    }\n");
    output.push_str("  }\n");
    output.push_str("}\n");
    output
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
    let selected = base_locale
        .filter(|base_locale| locales.iter().any(|locale| locale.locale == *base_locale))
        .or_else(|| locales.first().map(|locale| locale.locale.as_str()));

    selected
        .map(|locale| format!("\"{}\"", escape_string(locale)))
        .unwrap_or_else(|| "\"\" as LinguiniLanguageInput".to_owned())
}

fn push_runtime_types(output: &mut String) {
    output.push_str("export type LinguiniProviderOptions = {\n");
    output.push_str("  getLocale?: () => LinguiniLanguageInput;\n");
    output.push_str("  resolveLanguage?: () => LinguiniLanguageInput;\n");
    output.push_str("};\n\n");
}

fn push_runtime_functions(output: &mut String) {
    output.push_str("\nexport function isLocale(locale: unknown): locale is Locale {\n");
    output.push_str("  return normalizeLocale(locale) !== undefined;\n");
    output.push_str("}\n\n");
    output.push_str("export function normalizeLocale(locale: unknown): Locale | undefined {\n");
    output.push_str("  if (typeof locale !== \"string\") return undefined;\n");
    output.push_str("  if (locales.includes(locale as Locale)) return locale as Locale;\n");
    output.push_str("  const language = locale.toLowerCase().split(\"-\")[0];\n");
    output.push_str("  return locales.find((candidate) => candidate.toLowerCase() === language || candidate.toLowerCase().startsWith(`${language}-`));\n");
    output.push_str("}\n\n");
    output.push_str("export function getTextDirection(locale: Locale): TextDirection {\n");
    output.push_str("  return localeDirections[normalizeLocale(locale) ?? baseLocale];\n");
    output.push_str("}\n");
}

fn push_runtime_declarations(output: &mut String) {
    output.push_str("export type LinguiniProviderOptions = {\n");
    output.push_str("  getLocale?: () => LinguiniLanguageInput;\n  resolveLanguage?: () => LinguiniLanguageInput;\n};\n");
    output.push('\n');
}

fn push_runtime_function_declarations(output: &mut String) {
    output.push_str("export declare function isLocale(locale: unknown): locale is Locale;\n");
    output.push_str(
        "export declare function normalizeLocale(locale: unknown): Locale | undefined;\n",
    );
    output.push_str("export declare function getTextDirection(locale: Locale): TextDirection;\n");
}

fn web_options_literal(options: &TypeScriptWebOptions) -> String {
    let strategy = js_string_array(&options.strategy);
    let exclude = js_string_array(&options.exclude);
    let mut fields = vec![
        format!("strategy: [{strategy}]"),
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
        format!(
            "trailingSlash: \"{}\"",
            escape_string(&options.trailing_slash)
        ),
        format!("redirect: {}", js_bool(options.redirect)),
        format!("exclude: [{exclude}]"),
        format!("localizeLinks: {}", js_bool(options.localize_links)),
    ];

    if let Some(cookie_domain) = &options.cookie_domain {
        fields.push(format!(
            "cookieDomain: \"{}\"",
            escape_string(cookie_domain)
        ));
    }
    if let Some(global_variable_name) = &options.global_variable_name {
        fields.push(format!(
            "globalVariableName: \"{}\"",
            escape_string(global_variable_name)
        ));
    }
    if let Some(origin) = &options.origin {
        fields.push(format!("origin: \"{}\"", escape_string(origin)));
    }

    format!("{{ {} }}", fields.join(", "))
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
