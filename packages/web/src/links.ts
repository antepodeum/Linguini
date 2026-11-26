import type { LinguiniWebLocale, LinkLocalizationAttributes } from './types.js';
import { matchesRoute } from './routes.js';
import { resolveBaseUrl, parseUrl, isInternalHttpUrl, localizeParsedUrl } from './url.js';

/** Check href safety and configured route exclusions without changing the href. */
export function shouldLocalizeHref<Locale extends string>(web: LinguiniWebLocale<Locale>, href: string, input: Record<string, unknown> = {}): boolean {
    const normalized = web.options;
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
    return !web.shouldExclude(parsed, input);
}

/** Localize a plain href through the same link-safety checks as attributed links. */
export function localizeHrefAttribute<Locale extends string>(web: LinguiniWebLocale<Locale>, href: string, locale: NoInfer<Locale>, input: Record<string, unknown> = {}): string {
    return localizeLinkHref(web, href, locale, {}, input);
}

/** Validate attributes and localize from one URL/environment snapshot. */
export function localizeLinkHref<Locale extends string>(web: LinguiniWebLocale<Locale>, href: string, locale: NoInfer<Locale>, attributes: LinkLocalizationAttributes = {}, input: Record<string, unknown> = {}): string {
    const normalized = web.options;
    if (normalized.links.mode === 'manual' || !localizableAttributes(attributes)) return href;
    const value = String(href ?? '').trim();
    if (!value || value.startsWith('#') || value.startsWith('//')) return href;
    const scheme = /^([a-zA-Z][a-zA-Z0-9+.-]*):/.exec(value);
    if (scheme && !['http', 'https'].includes(scheme[1].toLowerCase())) return href;
    let url: URL;
    try {
        const base = resolveBaseUrl(normalized, input);
        url = parseUrl(value, base);
        if (!isInternalHttpUrl(url, base.origin)) return href;
    } catch {
        return href;
    }
    // Predicates receive an isolated URL. Their mutations must not change the
    // URL being localized, but later predicates still observe earlier mutations.
    const exclusions = normalized.routes.exclude;
    const checked = exclusions.some(matcher => typeof matcher === 'function') ? new URL(url) : url;
    if (exclusions.some(matcher => matchesRoute(matcher, checked))) return href;
    localizeParsedUrl(url, web.matchLocale(locale) ?? web.baseLocale, normalized, web);
    return scheme ? url.toString() : `${url.pathname}${url.search}${url.hash}`;
}

/** Check attributes before resolving the href or reading the URL environment. */
export function shouldLocalizeLink<Locale extends string>(web: LinguiniWebLocale<Locale>, href: string, attributes: LinkLocalizationAttributes = {}, input: Record<string, unknown> = {}): boolean {
    return localizableAttributes(attributes) && shouldLocalizeHref(web, href, input);
}

function localizableAttributes(attributes: LinkLocalizationAttributes): boolean {
    if (attributes.download || attributes.ignored)
        return false;
    const relations = String(attributes.rel ?? "").toLowerCase().split(/\s+/);
    return !relations.includes("external");
}
