import type { AlternateLink, LinguiniLocaleRuntime, LinguiniWebEnvironment, LinguiniWebLocale, LinguiniWebOptions, LinkLocalizationAttributes, TextDirection } from "./types.js";
import { normalizeOptions } from "./options.js";
import { matchesRoute } from "./routes.js";
import { matchLocaleValue, resolveBaseUrl, parseUrl, isInternalHttpUrl, stripBasePath, stripLeadingLocale, joinPath, shouldPrefixLocale, parseRuntimeUrl, resolveCookieSecure } from "./url.js";

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
        return localizeParsedUrl(copy, resolved);
    }

    function localizeParsedUrl(copy: URL, locale: Locale): URL {
        const path = stripBasePath(copy.pathname, normalized.environment.base);
        // In `never` mode locale-looking segments are ordinary application paths.
        // Do not strip them while normalizing a URL that should remain unprefixed.
        const withoutLocale = normalized.routing.localePrefix === "never"
            ? path
            : stripLeadingLocale(path, runtime.locales);
        copy.pathname = joinPath(normalized.environment.base, shouldPrefixLocale(normalized, locale) ? joinPath("/", locale, withoutLocale) : withoutLocale);
        return copy;
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
        localizeParsedUrl(url, resolved);
        if (scheme)
            return url.toString();
        return `${url.pathname}${url.search}${url.hash}`;
    }

    function shouldLocalizeHref(href: string, input: Record<string, unknown> = {}): boolean {
        if (normalized.links.mode === "manual")
            return false;
        const value = String(href ?? "").trim();
        if (!value || value.startsWith("#") || value.startsWith("//"))
            return false;
        const scheme = value.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);
        if (scheme && !["http", "https"].includes(scheme[1].toLowerCase()))
            return false;

        let parsed: URL;

        let base: URL;
        try {
            base = resolveBaseUrl(normalized, input);
            parsed = parseUrl(value, base);
        }
        catch {
            return false;
        }
        if (!isInternalHttpUrl(parsed, base.origin))
            return false;
        return !shouldExclude(parsed, input);
    }

    function localizeHrefAttribute(href: string, locale: Locale, input: Record<string, unknown> = {}): string {
        return shouldLocalizeHref(href, input) ? localizeHref(href, locale, input) : href;
    }

    function shouldLocalizeLink(href: string, attributes: LinkLocalizationAttributes = {}, input: Record<string, unknown> = {}): boolean {
        if (attributes.download || attributes.ignored)
            return false;
        const relations = String(attributes.rel ?? "").toLowerCase().split(/\s+/);
        if (relations.includes("external"))
            return false;
        return shouldLocalizeHref(href, input);
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

    function alternateLinks(url: string | URL, input: Record<string, unknown> = {}): AlternateLink[] {
        const parsed = parseRuntimeUrl(url, normalized, input);

        const links: AlternateLink[] = runtime.locales.map((locale) => ({
            rel: "alternate",
            hreflang: locale,
            href: localizeUrl(parsed, locale, input).toString(),
        }));
        links.push({ rel: "alternate", hreflang: "x-default", href: localizeUrl(parsed, runtime.baseLocale, input).toString() });
        return links;
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

    function serializeLocaleCookie(locale: Locale, input: Record<string, unknown> = {}): string {
        const parts = [`${normalized.cookie.name}=${encodeURIComponent(locale)}`];
        parts.push(`Max-Age=${input.maxAge ?? normalized.cookie.maxAge}`);
        parts.push(`Path=${normalized.cookie.path}`);
        if (normalized.cookie.domain)
            parts.push(`Domain=${normalized.cookie.domain}`);
        parts.push(`SameSite=${normalized.cookie.sameSite}`);
        if (resolveCookieSecure(normalized.cookie.secure, input))
            parts.push("Secure");
        if (input.httpOnly ?? normalized.cookie.httpOnly)
            parts.push("HttpOnly");
        return parts.join("; ");
    }

    function setLocaleCookie(target: unknown, locale: Locale, input: Record<string, unknown> = {}): void {
        const cookie = serializeLocaleCookie(locale, input);
        const sink =  target as {
            headers?: Pick<Headers, "append">;
            cookies?: {
                set(name: string, value: string, options?: Record<string, unknown>): void;
            };
        };
        if (sink.cookies?.set) {
            sink.cookies.set(normalized.cookie.name, locale, {
                path: normalized.cookie.path,
                domain: normalized.cookie.domain,
                maxAge: input.maxAge ?? normalized.cookie.maxAge,
                sameSite: normalized.cookie.sameSite,
                secure: resolveCookieSecure(normalized.cookie.secure, input),
                httpOnly: input.httpOnly ?? normalized.cookie.httpOnly,
            });
        }
        else {
            sink.headers?.append?.("set-cookie", cookie);
        }
    }
    return {
        ...runtime,
        options: normalized,
        matchLocale,
        resolveLocale: async (input = {}) => resolveLocaleSync(input),
        resolveLocaleSync,
        localizeUrl,
        localizeHref,
        shouldLocalizeHref,
        shouldLocalizeLink,
        localizeHrefAttribute,
        delocalizeUrl,
        delocalizePathname,
        alternateLinks,
        htmlAttrs,
        getCanonicalRedirect,
        shouldExclude,
        setLocaleCookie,
        serializeLocaleCookie,
        getTextDirection,
    };
}
