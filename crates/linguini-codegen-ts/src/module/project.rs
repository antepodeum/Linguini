use super::names::{escape_string, property_key, safe_identifier};
use super::TypeScriptLocaleModule;

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
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str("type LinguiniLanguageInput = LinguiniLanguage;\n\n");
    push_runtime_types(&mut output);
    output
        .push_str("export function createLinguini(language: LinguiniLanguageInput): Linguini {\n");
    output.push_str("  return localeModules[language as LinguiniLanguage];\n");
    output.push_str("}\n\n");
    output.push_str("export function getLocale(): Locale {\n");
    output.push_str("  return activeLocaleStore?.getStore() ?? activeLocale ?? baseLocale;\n");
    output.push_str("}\n\n");
    output.push_str("export function createLinguiniProvider(options: LinguiniProviderOptions = {}): Linguini {\n");
    output
        .push_str("  const resolve = options.getLocale ?? options.resolveLanguage ?? getLocale;\n");
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
    output.push_str("export const lgl: Linguini = createLinguini(");
    output.push_str("baseLocale");
    output.push_str(");\n");
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
    output.push_str("export type Linguini = (typeof localeModules)[LinguiniLanguage];\n\n");
    output.push_str("type LinguiniLanguageInput = LinguiniLanguage;\n\n");
    push_runtime_declarations(&mut output);
    output.push_str(
        "export declare function createLinguini(language: LinguiniLanguageInput): Linguini;\n\n",
    );
    output.push_str("export declare function getLocale(): Locale;\n\n");
    output.push_str(
        "export declare function createLinguiniProvider(options?: LinguiniProviderOptions): Linguini;\n\n",
    );
    output.push_str("export declare function configureLinguini(options: {\n");
    output.push_str("  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);\n");
    output.push_str("}): Linguini;\n\n");
    output.push_str("export declare const lgl: Linguini;\n");
    output.push_str("\n");
    push_runtime_function_declarations(&mut output);
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
    output.push_str("export type LocaleDetector = (context: LocaleDetectionContext) => unknown;\n");
    output.push_str("export type HeaderReader = { get(name: string): string | null } | Record<string, string | string[] | undefined>;\n");
    output.push_str("export type CookieReader = string | Record<string, string | undefined>;\n");
    output.push_str("export type LocaleDetectionContext = {\n");
    output.push_str("  url?: string | URL;\n");
    output.push_str("  headers?: HeaderReader;\n");
    output.push_str("  cookies?: CookieReader;\n");
    output.push_str("  cookieName?: string;\n");
    output.push_str("  navigator?: { language?: string; languages?: readonly string[] };\n");
    output.push_str("  localStorage?: { getItem(key: string): string | null };\n");
    output.push_str("  localStorageKey?: string;\n");
    output.push_str("};\n");
    output
        .push_str("export type LocaleResolveOptions = { strategy?: readonly LocaleDetector[] };\n");
    output.push_str("export type LinguiniProviderOptions = {\n");
    output.push_str("  getLocale?: () => LinguiniLanguageInput;\n");
    output.push_str("  resolveLanguage?: () => LinguiniLanguageInput;\n");
    output.push_str("};\n");
    output.push_str("export type LocaleHrefOptions = { stripBaseLocale?: boolean };\n");
    output.push_str("export type LinguiniMiddlewareOptions = {\n");
    output.push_str("  strategy?: readonly LocaleDetector[];\n");
    output.push_str("  disableAsyncLocalStorage?: boolean;\n");
    output.push_str("};\n");
    output.push_str("export type LinguiniMiddlewareContext = LocaleDetectionContext & { locale?: LinguiniLanguageInput };\n");
    output.push_str("export type TextDirection = \"ltr\" | \"rtl\";\n\n");
    output.push_str("type AsyncLocalStorageLike<T> = {\n");
    output.push_str("  getStore(): T | undefined;\n");
    output.push_str("  run<R>(store: T, callback: () => R): R;\n");
    output.push_str("};\n\n");
    output.push_str("let activeLocale: Locale = baseLocale;\n");
    output.push_str("let activeLocaleStore: AsyncLocalStorageLike<Locale> | undefined;\n\n");
}

fn push_runtime_functions(output: &mut String) {
    output.push_str("\nexport function resolveLocale(context: LocaleDetectionContext = {}, options: LocaleResolveOptions = {}): Locale {\n");
    output.push_str("  for (const detector of options.strategy ?? defaultLocaleStrategy) {\n");
    output.push_str("    const locale = normalizeLocale(detector(context));\n");
    output.push_str("    if (locale) return locale;\n");
    output.push_str("  }\n");
    output.push_str("  return baseLocale;\n");
    output.push_str("}\n\n");
    output.push_str("export const defaultLocaleStrategy = [\n");
    output.push_str("  detectUrlLocale,\n");
    output.push_str("  detectCookieLocale,\n");
    output.push_str("  detectPreferredLanguage,\n");
    output.push_str("  detectLocalStorageLocale,\n");
    output.push_str("  detectBaseLocale,\n");
    output.push_str("] as const;\n\n");
    output.push_str(
        "export function detectUrlLocale(context: LocaleDetectionContext): string | undefined {\n",
    );
    output.push_str("  const pathname = toUrl(context.url)?.pathname ?? (typeof context.url === \"string\" ? context.url : \"\");\n");
    output.push_str("  return pathname.split(\"/\").find(Boolean);\n");
    output.push_str("}\n\n");
    output.push_str("export function detectCookieLocale(context: LocaleDetectionContext): string | undefined {\n");
    output.push_str("  const name = context.cookieName ?? \"linguini_locale\";\n");
    output.push_str("  if (typeof context.cookies === \"string\") return readCookieString(context.cookies, name);\n");
    output.push_str("  if (context.cookies) return context.cookies[name];\n");
    output.push_str("  const cookie = readHeader(context.headers, \"cookie\");\n");
    output.push_str("  return cookie ? readCookieString(cookie, name) : undefined;\n");
    output.push_str("}\n\n");
    output.push_str("export function detectPreferredLanguage(context: LocaleDetectionContext): string | undefined {\n");
    output.push_str("  const header = readHeader(context.headers, \"accept-language\");\n");
    output.push_str("  const candidates = header ? parseAcceptLanguage(header) : context.navigator?.languages ?? [context.navigator?.language ?? \"\"];\n");
    output.push_str("  return candidates.map(normalizeLocale).find(Boolean);\n");
    output.push_str("}\n\n");
    output.push_str("export function detectLocalStorageLocale(context: LocaleDetectionContext): string | undefined {\n");
    output.push_str("  return context.localStorage?.getItem(context.localStorageKey ?? \"linguini_locale\") ?? undefined;\n");
    output.push_str("}\n\n");
    output.push_str("export function detectBaseLocale(): LinguiniLanguageInput {\n");
    output.push_str("  return baseLocale;\n");
    output.push_str("}\n\n");
    output.push_str("export function localizeHref(href: string | URL, locale: LinguiniLanguageInput, options: LocaleHrefOptions = {}): string {\n");
    output.push_str("  const resolved = normalizeLocale(locale) ?? baseLocale;\n");
    output.push_str("  const url = toUrl(href);\n");
    output.push_str("  const pathname = url?.pathname ?? String(href);\n");
    output.push_str("  const suffix = url ? `${url.search}${url.hash}` : \"\";\n");
    output.push_str("  const parts = pathname.split(\"/\").filter(Boolean);\n");
    output.push_str("  if (parts.length > 0 && normalizeLocale(parts[0])) parts.shift();\n");
    output.push_str(
        "  if (!(options.stripBaseLocale && resolved === baseLocale)) parts.unshift(resolved);\n",
    );
    output.push_str("  const nextPath = `/${parts.join(\"/\")}`;\n");
    output.push_str("  return url && url.origin !== \"http://linguini.local\" && url.origin !== \"null\" ? `${url.origin}${nextPath}${suffix}` : `${nextPath}${suffix}`;\n");
    output.push_str("}\n\n");
    output.push_str("export function shouldRedirect(href: string | URL, locale: LinguiniLanguageInput, options: LocaleHrefOptions = {}): boolean {\n");
    output.push_str("  const current = toComparableHref(href);\n");
    output.push_str("  return current !== localizeHref(href, locale, options);\n");
    output.push_str("}\n\n");
    output.push_str("export function runWithLocale<R>(locale: LinguiniLanguageInput, callback: () => R, options: LinguiniMiddlewareOptions = {}): R {\n");
    output.push_str("  const resolved = normalizeLocale(locale) ?? baseLocale;\n");
    output.push_str(
        "  const store = options.disableAsyncLocalStorage ? undefined : getAsyncLocalStorage();\n",
    );
    output.push_str("  if (store) return store.run(resolved, callback);\n");
    output.push_str("  const previous = activeLocale;\n");
    output.push_str("  activeLocale = resolved;\n");
    output.push_str("  try {\n");
    output.push_str("    const result = callback();\n");
    output.push_str("    if (isPromiseLike(result)) return result.finally(() => { activeLocale = previous; }) as R;\n");
    output.push_str("    return result;\n");
    output.push_str("  } finally {\n");
    output.push_str("    activeLocale = previous;\n");
    output.push_str("  }\n");
    output.push_str("}\n\n");
    output.push_str(
        "export function createLinguiniMiddleware(options: LinguiniMiddlewareOptions = {}) {\n",
    );
    output.push_str("  return function linguiniMiddleware<R>(context: LinguiniMiddlewareContext, next: () => R): R {\n");
    output.push_str("    return runWithLocale(context.locale ?? resolveLocale(context, options), next, options);\n");
    output.push_str("  };\n");
    output.push_str("}\n\n");
    output.push_str("export function injectLangAndDir(template: string, locale: LinguiniLanguageInput = getLocale()): string {\n");
    output.push_str("  const resolved = normalizeLocale(locale) ?? baseLocale;\n");
    output.push_str("  return template.split(\"%lang%\").join(resolved).split(\"%dir%\").join(getTextDirection(resolved));\n");
    output.push_str("}\n\n");
    output.push_str("export function getTextDirection(locale: LinguiniLanguageInput = getLocale()): TextDirection {\n");
    output.push_str("  const language = String(locale).toLowerCase().split(\"-\")[0];\n");
    output.push_str("  return [\"ar\", \"fa\", \"he\", \"ps\", \"ur\", \"yi\"].includes(language) ? \"rtl\" : \"ltr\";\n");
    output.push_str("}\n\n");
    push_private_runtime_helpers(output);
}

fn push_private_runtime_helpers(output: &mut String) {
    output.push_str("function normalizeLocale(locale: unknown): Locale | undefined {\n");
    output.push_str("  if (typeof locale !== \"string\") return undefined;\n");
    output.push_str("  if (locales.includes(locale as Locale)) return locale as Locale;\n");
    output.push_str("  const language = locale.toLowerCase().split(\"-\")[0];\n");
    output.push_str("  return locales.find((candidate) => candidate.toLowerCase() === language || candidate.toLowerCase().startsWith(`${language}-`));\n");
    output.push_str("}\n\n");
    output.push_str("function readHeader(headers: HeaderReader | undefined, name: string): string | undefined {\n");
    output.push_str("  if (!headers) return undefined;\n");
    output.push_str("  const get = (headers as { get?: unknown }).get;\n");
    output.push_str("  if (typeof get === \"function\") return (get.call(headers, name) as string | null) ?? undefined;\n");
    output.push_str("  const record = headers as Record<string, string | string[] | undefined>;\n");
    output.push_str("  const value = record[name] ?? record[name.toLowerCase()];\n");
    output.push_str("  return Array.isArray(value) ? value.join(\",\") : value;\n");
    output.push_str("}\n\n");
    output.push_str(
        "function readCookieString(source: string, name: string): string | undefined {\n",
    );
    output.push_str("  return source.split(\";\").map((part) => part.trim()).find((part) => part.startsWith(`${name}=`))?.slice(name.length + 1);\n");
    output.push_str("}\n\n");
    output.push_str("function parseAcceptLanguage(header: string): string[] {\n");
    output.push_str("  return header.split(\",\").map((part) => part.trim().split(\";\")[0]).filter(Boolean);\n");
    output.push_str("}\n\n");
    output.push_str("function toUrl(value: string | URL | undefined): URL | undefined {\n");
    output.push_str("  if (!value) return undefined;\n");
    output.push_str("  if (value instanceof URL) return value;\n");
    output.push_str(
        "  try { return new URL(value, \"http://linguini.local\"); } catch { return undefined; }\n",
    );
    output.push_str("}\n\n");
    output.push_str("function toComparableHref(value: string | URL): string {\n");
    output.push_str("  const url = toUrl(value);\n");
    output.push_str("  if (!url) return String(value);\n");
    output.push_str("  const suffix = `${url.search}${url.hash}`;\n");
    output.push_str("  return url.origin === \"http://linguini.local\" ? `${url.pathname}${suffix}` : `${url.origin}${url.pathname}${suffix}`;\n");
    output.push_str("}\n\n");
    output.push_str("function isPromiseLike(value: unknown): value is Promise<unknown> {\n");
    output.push_str(
        "  return typeof value === \"object\" && value !== null && \"finally\" in value;\n",
    );
    output.push_str("}\n\n");
    output
        .push_str("function getAsyncLocalStorage(): AsyncLocalStorageLike<Locale> | undefined {\n");
    output.push_str("  if (activeLocaleStore) return activeLocaleStore;\n");
    output.push_str("  try {\n");
    output.push_str("    const req = Function(\"return typeof require === 'function' && require\")() as ((name: string) => { AsyncLocalStorage?: new () => AsyncLocalStorageLike<Locale> }) | false;\n");
    output.push_str("    const Storage = req && req(\"node:async_hooks\").AsyncLocalStorage;\n");
    output.push_str("    activeLocaleStore = Storage ? new Storage() : undefined;\n");
    output.push_str("  } catch {\n");
    output.push_str("    activeLocaleStore = undefined;\n");
    output.push_str("  }\n");
    output.push_str("  return activeLocaleStore;\n");
    output.push_str("}\n");
}

fn push_runtime_declarations(output: &mut String) {
    output.push_str("export type LocaleDetector = (context: LocaleDetectionContext) => unknown;\n");
    output.push_str("export type HeaderReader = { get(name: string): string | null } | Record<string, string | string[] | undefined>;\n");
    output.push_str("export type CookieReader = string | Record<string, string | undefined>;\n");
    output.push_str("export type LocaleDetectionContext = {\n");
    output
        .push_str("  url?: string | URL;\n  headers?: HeaderReader;\n  cookies?: CookieReader;\n");
    output.push_str("  cookieName?: string;\n  navigator?: { language?: string; languages?: readonly string[] };\n");
    output.push_str("  localStorage?: { getItem(key: string): string | null };\n  localStorageKey?: string;\n};\n");
    output
        .push_str("export type LocaleResolveOptions = { strategy?: readonly LocaleDetector[] };\n");
    output.push_str("export type LinguiniProviderOptions = {\n");
    output.push_str("  getLocale?: () => LinguiniLanguageInput;\n  resolveLanguage?: () => LinguiniLanguageInput;\n};\n");
    output.push_str("export type LocaleHrefOptions = { stripBaseLocale?: boolean };\n");
    output.push_str("export type LinguiniMiddlewareOptions = {\n");
    output.push_str(
        "  strategy?: readonly LocaleDetector[];\n  disableAsyncLocalStorage?: boolean;\n};\n",
    );
    output.push_str("export type LinguiniMiddlewareContext = LocaleDetectionContext & { locale?: LinguiniLanguageInput };\n");
    output.push_str("export type TextDirection = \"ltr\" | \"rtl\";\n\n");
}

fn push_runtime_function_declarations(output: &mut String) {
    output.push_str("export declare function resolveLocale(context?: LocaleDetectionContext, options?: LocaleResolveOptions): Locale;\n");
    output.push_str("export declare const defaultLocaleStrategy: readonly [typeof detectUrlLocale, typeof detectCookieLocale, typeof detectPreferredLanguage, typeof detectLocalStorageLocale, typeof detectBaseLocale];\n");
    output.push_str("export declare function detectUrlLocale(context: LocaleDetectionContext): string | undefined;\n");
    output.push_str("export declare function detectCookieLocale(context: LocaleDetectionContext): string | undefined;\n");
    output.push_str("export declare function detectPreferredLanguage(context: LocaleDetectionContext): string | undefined;\n");
    output.push_str("export declare function detectLocalStorageLocale(context: LocaleDetectionContext): string | undefined;\n");
    output.push_str("export declare function detectBaseLocale(): LinguiniLanguageInput;\n");
    output.push_str("export declare function localizeHref(href: string | URL, locale: LinguiniLanguageInput, options?: LocaleHrefOptions): string;\n");
    output.push_str("export declare function shouldRedirect(href: string | URL, locale: LinguiniLanguageInput, options?: LocaleHrefOptions): boolean;\n");
    output.push_str("export declare function runWithLocale<R>(locale: LinguiniLanguageInput, callback: () => R, options?: LinguiniMiddlewareOptions): R;\n");
    output.push_str("export declare function createLinguiniMiddleware(options?: LinguiniMiddlewareOptions): <R>(context: LinguiniMiddlewareContext, next: () => R) => R;\n");
    output.push_str("export declare function injectLangAndDir(template: string, locale?: LinguiniLanguageInput): string;\n");
    output.push_str("export declare function getTextDirection(locale?: LinguiniLanguageInput): TextDirection;\n");
}
