import type { LinguiniWebLocale } from "./types.js";
const FALLBACK_URL = "http://localhost";
const INVALID_URL_MESSAGE = "Linguini: invalid URL";

export function matchLocaleValue<Locale extends string>(locales: readonly Locale[], value: unknown): Locale | undefined {
    const candidates = Array.isArray(value) ? value : [value];
    for (const candidate of candidates) {
        if (typeof candidate !== "string")
            continue;
        const exact = locales.find((locale) => locale.toLowerCase() === candidate.toLowerCase());
        if (exact)
            return exact;
    }
    return undefined;
}

export function matchExactLocale<Locale extends string>(locales: readonly Locale[], value: unknown): Locale | undefined {
    if (typeof value !== "string")
        return undefined;
    return locales.find((locale) => locale.toLowerCase() === value.toLowerCase());
}

export function stripLeadingLocale<Locale extends string>(pathname: string, locales: readonly Locale[]): string {
    const parts = pathname.split("/").filter(Boolean);
    return matchExactLocale(locales, parts[0]) ? ensureSlash(parts.slice(1).join("/")) : ensureSlash(parts.join("/"));
}

export function stripBasePath(pathname: string, basePath: string): string {
    const normalizedPath = ensureSlash(pathname);
    const normalizedBase = basePath && basePath !== "/" ? ensureSlash(basePath).replace(/\/$/, "") : "";
    if (!normalizedBase)
        return normalizedPath;
    if (normalizedPath === normalizedBase)
        return "/";
    return normalizedPath.startsWith(`${normalizedBase}/`) ? normalizedPath.slice(normalizedBase.length) || "/" : normalizedPath;
}

export function joinPath(...parts: (string | undefined)[]): string {
    const joined = parts
        .filter((part) => part !== undefined && part !== "")
        .map((part) => String(part).replace(/^\/+|\/+$/g, ""))
        .filter(Boolean)
        .join("/");
    return `/${joined}`.replace(/\/+/g, "/");
}

export function ensureSlash(pathname: string): string {
    const value = String(pathname || "/");
    return value.startsWith("/") ? value : `/${value}`;
}

export function shouldPrefixLocale(options: LinguiniWebLocale<string>["options"], locale: string): boolean {
    switch (options.routing.localePrefix) {
        case "always":
            return true;
        case "never":
            return false;
        case "except-default":
        default:
            return locale !== options.baseLocale;
    }
}

export function resolveBaseUrl(options: LinguiniWebLocale<string>["options"], input: Record<string, unknown> = {}): URL {
    const origin = parseUrl(input.origin ?? options.environment.origin ?? browserLocationHref() ?? FALLBACK_URL, FALLBACK_URL);
    const current = input.currentUrl ?? input.url;
    return current === undefined ? origin : parseUrl(current, origin);
}

export function normalizeBasePath(value: string): string {
    if (!value || value === "/")
        return "";
    return ensureSlash(value).replace(/\/+$/, "");
}

export function resolveEnvironmentProtocol(origin: string | URL | undefined): string {
    try {
        return parseUrl(origin ?? browserLocationHref() ?? FALLBACK_URL, FALLBACK_URL).protocol;
    }
    catch {
        return "http:";
    }
}

export function resolveCookieSecure(policy: boolean | "auto", input: Record<string, unknown>): boolean {
    if (typeof input.secure === "boolean")
        return input.secure;
    if (policy !== "auto")
        return policy;
    return resolveEnvironmentProtocol(input.origin as string | URL | undefined) === "https:";
}

export function parseRuntimeUrl(url: string | URL, options: LinguiniWebLocale<string>["options"], input: Record<string, unknown> = {}): URL {
    return parseUrl(url, resolveBaseUrl(options, input));
}

export function parseUrl(value: unknown, base: string | URL): URL {
    try {
        return new URL(String(value), String(base));
    }
    catch {
        throw new TypeError(INVALID_URL_MESSAGE);
    }
}

export function isInternalHttpUrl(url: URL, origin: string): boolean {
    return ["http:", "https:"].includes(url.protocol) && url.origin === origin;
}
export function browserLocationHref() {
    const href = ( globalThis as {
        location?: {
            href?: unknown;
        };
    }).location?.href;
    return typeof href === "string" ? href : undefined;
}
