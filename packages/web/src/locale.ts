import type { LinguiniLocaleRuntime, LinguiniWebEnvironment, LinguiniWebLocale, LinguiniWebOptions, TextDirection } from "./types.js";
import { normalizeOptions } from "./options.js";
import { matchesRoute } from "./routes.js";
import { matchLocaleValue, resolveBaseUrl, parseUrl, isInternalHttpUrl, stripBasePath, stripLeadingLocale, joinPath, localizeParsedUrl, parseRuntimeUrl } from "./url.js";

export function createWebLocaleI18n<Locale extends string>(runtime: LinguiniLocaleRuntime<Locale>, options: LinguiniWebOptions = {}, environment: LinguiniWebEnvironment = {}): LinguiniWebLocale<Locale> {
    const sources = options.locale?.sources ?? [];
    const normalized = normalizeOptions(runtime.baseLocale, options, environment);

    const matchLocale = (locale: unknown): Locale | undefined => runtime.normalizeLocale?.(locale) ?? matchLocaleValue(runtime.locales, locale);

    const getTextDirection = (locale: Locale): TextDirection => runtime.getTextDirection?.(locale) ?? runtime.localeDirections?.[locale] ?? "ltr";

    function resolveLocaleSync(input: Record<string, unknown> = {}): Locale {
        for (const source of sources) {
            const resolved = source.resolve(input, { locales: runtime.locales, baseLocale: runtime.baseLocale, matchLocale, options: normalized });
            if (resolved !== undefined)
                return resolved;
        }
        return runtime.baseLocale;
    }

    function localizeUrl(url: string | URL, locale: Locale, input: Record<string, unknown> = {}): URL {
        const resolved = matchLocale(locale) ?? runtime.baseLocale;
        const base = resolveBaseUrl(normalized, input);
        const copy = parseUrl(url, base);
        if (!isInternalHttpUrl(copy, base.origin))
            return copy;
        return localizeParsedUrl(copy, resolved, normalized, runtime);
    }

    function localizeHref(href: string, locale: Locale, input: Record<string, unknown> = {}): string {
        const value = String(href);
        const trimmed = value.trim();
        if (!trimmed || trimmed.startsWith("#") || trimmed.startsWith("//"))
            return value;
        const scheme = trimmed.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);
        if (scheme && !["http", "https"].includes(scheme[1].toLowerCase()))
            return value;
        const base = resolveBaseUrl(normalized, input);
        const url = parseUrl(value, base);
        if (!isInternalHttpUrl(url, base.origin))
            return value;
        const resolved = matchLocale(locale) ?? runtime.baseLocale;
        localizeParsedUrl(url, resolved, normalized, runtime);
        if (scheme)
            return url.toString();
        return `${url.pathname}${url.search}${url.hash}`;
    }

    function delocalizeUrl(url: string | URL, input: Record<string, unknown> = {}): URL {
        const base = resolveBaseUrl(normalized, input);
        const copy = parseUrl(url, base);
        if (!isInternalHttpUrl(copy, base.origin))
            return copy;
        if (normalized.routing.localePrefix === "never")
            return copy;
        copy.pathname = joinPath(normalized.environment.base, stripLeadingLocale(stripBasePath(copy.pathname, normalized.environment.base), runtime.locales));
        return copy;
    }

    function delocalizePathname(pathname: string, input: Record<string, unknown> = {}): string {
        return delocalizeUrl(pathname, input).pathname;
    }

    function htmlAttrs(locale: Locale): {
        lang: Locale;
        dir: TextDirection;
    } {
        const resolved = matchLocale(locale) ?? runtime.baseLocale;
        return { lang: resolved, dir: getTextDirection(resolved) };
    }

    function getCanonicalRedirect(url: string | URL, locale: Locale, input: Record<string, unknown> = {}): string | undefined {
        if (normalized.routing.localePrefix === "never" || normalized.routing.canonical === "preserve" || shouldExclude(url, input))
            return undefined;
        const parsed = parseRuntimeUrl(url, normalized, input);
        const canonical = localizeUrl(parsed, locale, input);
        return canonical.pathname === parsed.pathname ? undefined : `${canonical.pathname}${canonical.search}${canonical.hash}`;
    }

    function shouldExclude(url: string | URL, input: Record<string, unknown> = {}): boolean {
        if (normalized.routes.exclude.length === 0) return false;
        const parsed = parseRuntimeUrl(url, normalized, input);
        return normalized.routes.exclude.some((matcher) => matchesRoute(matcher, parsed));
    }

    return {
        locales: runtime.locales,
        baseLocale: runtime.baseLocale,
        localeDirections: runtime.localeDirections,
        normalizeLocale: runtime.normalizeLocale,
        options: normalized,
        matchLocale,
        resolveLocale: async (input = {}) => resolveLocaleSync(input),
        resolveLocaleSync,
        localizeUrl,
        localizeHref,
        delocalizeUrl,
        delocalizePathname,
        htmlAttrs,
        getCanonicalRedirect,
        shouldExclude,
        getTextDirection,
    };
}
