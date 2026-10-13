import { resolveAcceptLanguageLocale } from "./web/accept-language.js";

/** @type {readonly LocaleSource[]} */
const DEFAULT_SOURCES = ["path", "cookie", "accept-language"];
const DEFAULT_COOKIE_MAX_AGE = 60 * 60 * 24 * 365;
const FALLBACK_URL = "http://localhost";
const INVALID_URL_MESSAGE = "Linguini: invalid URL";
/**  @typedef {"ltr" | "rtl"} TextDirection */
/**  @typedef {"path" | "cookie" | "local-storage" | "accept-language"} LocaleSource */
/**  @typedef {"always" | "except-default" | "never"} LocalePrefixMode */
/**  @typedef {{ writesPath: boolean; writesCookie: boolean; writesLocalStorage: boolean }} LocaleSwitchPlan */

/**  @typedef {{ rel: "alternate"; hreflang: string; href: string }} AlternateLink */

/**  @typedef {{ download?: boolean; ignored?: boolean; rel?: string | null }} LinkLocalizationAttributes */

/**  @typedef {{ routing?: {
    localePrefix?: LocalePrefixMode;
    canonical?: "redirect" | "preserve";
  }; locale?: {
    sources?: readonly LocaleSource[];
    switch?: LocaleSwitchPlan;
  }; cookie?: {
    name?: string;
    path?: string | "auto";
    domain?: string;
    maxAge?: number;
    sameSite?: "lax" | "strict" | "none";
    secure?: boolean | "auto";
    httpOnly?: boolean;
  }; localStorage?: { key?: string }; links?: { mode?: "transform" | "runtime" | "manual" }; routes?: { exclude?: readonly (string | RegExp | ((url: URL) => boolean))[] } }} LinguiniWebOptions */

/**  @typedef {{ base?: string; origin?: string | URL }} LinguiniWebEnvironment */

/** @template {string} [Locale=string] @typedef {{ locales: readonly Locale[]; baseLocale: Locale; localeDirections?: Readonly<Record<Locale, TextDirection>>; normalizeLocale?: (locale: unknown) => Locale | undefined; getTextDirection?: (locale: Locale) => TextDirection }} LinguiniLocaleRuntime */

/** @template {string} [Locale=string] @template [Linguini=unknown] @typedef {LinguiniLocaleRuntime<Locale> & { createLinguini: (locale: Locale) => Linguini; prepareLinguini?: (locale: Locale) => Promise<Linguini> }} LinguiniRuntime */

/**
 * Localization and delocalization leave external and non-HTTP URLs unchanged. Explicit URL
 * operations throw `TypeError("Linguini: invalid URL")` for malformed input; link-safety helpers
 * fail closed and preserve the original href instead.
 */
/** @template {string} [Locale=string] @typedef {{ locale: Locale; baseLocale: Locale; locales: readonly Locale[]; direction: TextDirection; textDirection: TextDirection; lang: Locale; htmlAttrs: { lang: Locale; dir: TextDirection }; localizeHref: (href: string, locale?: Locale, input?: Record<string, unknown>) => string; localizeUrl: (url: string | URL, locale?: Locale, input?: Record<string, unknown>) => URL; shouldLocalizeHref: (href: string, input?: Record<string, unknown>) => boolean; shouldLocalizeLink: (href: string, attributes?: LinkLocalizationAttributes, input?: Record<string, unknown>) => boolean; localizeHrefAttribute: (href: string, locale?: Locale, input?: Record<string, unknown>) => string; delocalizeUrl: (url: string | URL, input?: Record<string, unknown>) => URL; alternateLinks: (url: string | URL, input?: Record<string, unknown>) => AlternateLink[] }} LinguiniLocaleRequestContext */

/** @template {string} [Locale=string] @template [Linguini=unknown] @typedef {LinguiniLocaleRequestContext<Locale> & { messages: Linguini; l: Linguini }} LinguiniRequestContext */

/**
 * Localization and delocalization leave external and non-HTTP URLs unchanged. Explicit URL
 * operations throw `TypeError("Linguini: invalid URL")` for malformed input; link-safety helpers
 * fail closed and preserve the original href instead.
 */
/** @template {string} [Locale=string] @typedef {LinguiniLocaleRuntime<Locale> & { options: {
    routing: { localePrefix: LocalePrefixMode; canonical: "redirect" | "preserve" };
    locale: { sources: readonly LocaleSource[]; switch: LocaleSwitchPlan };
    cookie: { name: string; path: string; domain?: string; maxAge: number; sameSite: "lax" | "strict" | "none"; secure: boolean | "auto"; httpOnly: boolean };
    localStorage: { key: string };
    links: { mode: "transform" | "runtime" | "manual" };
    routes: { exclude: readonly (string | RegExp | ((url: URL) => boolean))[] };
    environment: { base: string; origin?: string | URL };
    baseLocale: string;
  }; matchLocale: (locale: unknown) => Locale | undefined; resolveLocale: (input?: Record<string, unknown>) => Promise<Locale>; resolveLocaleSync: (input?: Record<string, unknown>) => Locale; localizeUrl: (url: string | URL, locale: Locale, input?: Record<string, unknown>) => URL; localizeHref: (href: string, locale: Locale, input?: Record<string, unknown>) => string; shouldLocalizeHref: (href: string, input?: Record<string, unknown>) => boolean; shouldLocalizeLink: (href: string, attributes?: LinkLocalizationAttributes, input?: Record<string, unknown>) => boolean; localizeHrefAttribute: (href: string, locale: Locale, input?: Record<string, unknown>) => string; delocalizeUrl: (url: string | URL, input?: Record<string, unknown>) => URL; delocalizePathname: (pathname: string, input?: Record<string, unknown>) => string; alternateLinks: (url: string | URL, input?: Record<string, unknown>) => AlternateLink[]; htmlAttrs: (locale: Locale) => { lang: Locale; dir: TextDirection }; getTextDirection: (locale: Locale) => TextDirection; getCanonicalRedirect: (url: string | URL, locale: Locale, input?: Record<string, unknown>) => string | undefined; shouldExclude: (url: string | URL, input?: Record<string, unknown>) => boolean; setLocaleCookie: (target: unknown, locale: Locale, input?: Record<string, unknown>) => void; serializeLocaleCookie: (locale: Locale, input?: Record<string, unknown>) => string }} LinguiniWebLocale */

/** @template {string} [Locale=string] @template [Linguini=unknown] @typedef {LinguiniWebLocale<Locale> & { createLinguini: (locale: Locale) => Linguini; resolveRequest: (request: Request, input?: Record<string, unknown>) => Promise<LinguiniRequestContext<Locale, Linguini>>; createRequestContext: (locale: Locale, input?: Record<string, unknown>) => LinguiniRequestContext<Locale, Linguini> }} LinguiniWeb */

/** @template {string} Locale @param {LinguiniLocaleRuntime<Locale>} runtime @param {LinguiniWebOptions} [options] @param {LinguiniWebEnvironment} [environment] @returns {LinguiniWebLocale<Locale>} */
export function createWebLocaleI18n(runtime, options = {}, environment = {}) {
  const normalized = normalizeOptions(runtime.baseLocale, options, environment);
  /** @param {unknown} locale @returns {Locale | undefined} */
  const matchLocale = (locale) => runtime.normalizeLocale?.(locale) ?? matchLocaleValue(runtime.locales, locale);
  /** @param {Locale} locale @returns {TextDirection} */
  const getTextDirection = (locale) => runtime.getTextDirection?.(locale) ?? runtime.localeDirections?.[locale] ?? "ltr";

  /** @param {Record<string, unknown>} [input] @returns {Locale} */
  function resolveLocaleSync(input = {}) {
    const resolved_0 = resolveAcceptLanguageLocale(input, runtime.locales, runtime.baseLocale, matchLocale);
    if (resolved_0) return resolved_0;
    return runtime.baseLocale;
  }

  /** @param {string | URL} url @param {Locale} locale @param {Record<string, unknown>} [input] @returns {URL} */
  function localizeUrl(url, locale, input = {}) {
    const resolved = matchLocale(locale) ?? runtime.baseLocale;
    const base = resolveBaseUrl(normalized, input);
    const copy = parseUrl(url, base);
    if (!isInternalHttpUrl(copy, base.origin)) return copy;
    return localizeParsedUrl(copy, resolved);
  }

  /** @param {URL} copy @param {Locale} locale @returns {URL} */
  function localizeParsedUrl(copy, locale) {
    const path = stripBasePath(copy.pathname, normalized.environment.base);
    // In `never` mode locale-looking segments are ordinary application paths.
    // Do not strip them while normalizing a URL that should remain unprefixed.
    const withoutLocale = normalized.routing.localePrefix === "never"
      ? path
      : stripLeadingLocale(path, runtime.locales);
    copy.pathname = joinPath(normalized.environment.base, shouldPrefixLocale(normalized, locale) ? joinPath("/", locale, withoutLocale) : withoutLocale);
    return copy;
  }

  /** @param {string} href @param {Locale} locale @param {Record<string, unknown>} [input] @returns {string} */
  function localizeHref(href, locale, input = {}) {
    const value = String(href);
    const trimmed = value.trim();
    if (!trimmed || trimmed.startsWith("#") || trimmed.startsWith("//")) return value;
    const scheme = trimmed.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);
    if (scheme && !["http", "https"].includes(scheme[1].toLowerCase())) return value;

    const base = resolveBaseUrl(normalized, input);
    const url = parseUrl(value, base);
    if (!isInternalHttpUrl(url, base.origin)) return value;

    const resolved = matchLocale(locale) ?? runtime.baseLocale;
    localizeParsedUrl(url, resolved);
    if (scheme) return url.toString();
    return `${url.pathname}${url.search}${url.hash}`;
  }

  /** @param {string} href @param {Record<string, unknown>} [input] @returns {boolean} */
  function shouldLocalizeHref(href, input = {}) {
    if (normalized.links.mode === "manual") return false;
    const value = String(href ?? "").trim();
    if (!value || value.startsWith("#") || value.startsWith("//")) return false;
    const scheme = value.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);
    if (scheme && !["http", "https"].includes(scheme[1].toLowerCase())) return false;
    /** @type {URL} */
    let parsed;
    /** @type {URL} */
    let base;
    try {
      base = resolveBaseUrl(normalized, input);
      parsed = parseUrl(value, base);
    } catch {
      return false;
    }
    if (!isInternalHttpUrl(parsed, base.origin)) return false;
    return !shouldExclude(parsed, input);
  }

  /** @param {string} href @param {Locale} locale @param {Record<string, unknown>} [input] @returns {string} */
  function localizeHrefAttribute(href, locale, input = {}) {
    return shouldLocalizeHref(href, input) ? localizeHref(href, locale, input) : href;
  }

  /** @param {string} href @param {LinkLocalizationAttributes} [attributes] @param {Record<string, unknown>} [input] @returns {boolean} */
  function shouldLocalizeLink(href, attributes = {}, input = {}) {
    if (attributes.download || attributes.ignored) return false;
    const relations = String(attributes.rel ?? "").toLowerCase().split(/\s+/);
    if (relations.includes("external")) return false;
    return shouldLocalizeHref(href, input);
  }

  /** @param {string | URL} url @param {Record<string, unknown>} [input] @returns {URL} */
  function delocalizeUrl(url, input = {}) {
    const base = resolveBaseUrl(normalized, input);
    const copy = parseUrl(url, base);
    if (!isInternalHttpUrl(copy, base.origin)) return copy;
    if (normalized.routing.localePrefix === "never") return copy;
    copy.pathname = joinPath(normalized.environment.base, stripLeadingLocale(stripBasePath(copy.pathname, normalized.environment.base), runtime.locales));
    return copy;
  }

  /** @param {string} pathname @param {Record<string, unknown>} [input] @returns {string} */
  function delocalizePathname(pathname, input = {}) {
    return delocalizeUrl(pathname, input).pathname;
  }

  /** @param {string | URL} url @param {Record<string, unknown>} [input] @returns {AlternateLink[]} */
  function alternateLinks(url, input = {}) {
    const parsed = parseRuntimeUrl(url, normalized, input);
    /** @type {AlternateLink[]} */
    const links = runtime.locales.map((locale) => ({
      rel: "alternate",
      hreflang: locale,
      href: localizeUrl(parsed, locale, input).toString(),
    }));
    links.push({ rel: "alternate", hreflang: "x-default", href: localizeUrl(parsed, runtime.baseLocale, input).toString() });
    return links;
  }

  /** @param {Locale} locale @returns {{lang: Locale, dir: TextDirection}} */
  function htmlAttrs(locale) {
    const resolved = matchLocale(locale) ?? runtime.baseLocale;
    return { lang: resolved, dir: getTextDirection(resolved) };
  }

  /** @param {string | URL} url @param {Locale} locale @param {Record<string, unknown>} [input] @returns {string | undefined} */
  function getCanonicalRedirect(url, locale, input = {}) {
    if (normalized.routing.localePrefix === "never" || normalized.routing.canonical === "preserve" || shouldExclude(url, input)) return undefined;
    const parsed = parseRuntimeUrl(url, normalized, input);
    const canonical = localizeUrl(parsed, locale, input);
    return canonical.pathname === parsed.pathname ? undefined : `${canonical.pathname}${canonical.search}${canonical.hash}`;
  }

  /** @param {string | URL} _url @param {Record<string, unknown>} [_input] @returns {boolean} */
  function shouldExclude(_url, _input = {}) {
    return false;
  }

  /** @param {Locale} locale @param {Record<string, unknown>} [input] @returns {string} */
  function serializeLocaleCookie(locale, input = {}) {
    const parts = [`${normalized.cookie.name}=${encodeURIComponent(locale)}`];
    parts.push(`Max-Age=${input.maxAge ?? normalized.cookie.maxAge}`);
    parts.push(`Path=${normalized.cookie.path}`);
    if (normalized.cookie.domain) parts.push(`Domain=${normalized.cookie.domain}`);
    parts.push(`SameSite=${normalized.cookie.sameSite}`);
    if (resolveCookieSecure(normalized.cookie.secure, input)) parts.push("Secure");
    if (input.httpOnly ?? normalized.cookie.httpOnly) parts.push("HttpOnly");
    return parts.join("; ");
  }

  /** @param {unknown} target @param {Locale} locale @param {Record<string, unknown>} [input] @returns {void} */
  function setLocaleCookie(target, locale, input = {}) {
    const cookie = serializeLocaleCookie(locale, input);
    const sink = /** @type {{ headers?: Pick<Headers, "append">; cookies?: { set(name: string, value: string, options?: Record<string, unknown>): void } }} */ (target);
    if (sink.cookies?.set) {
      sink.cookies.set(normalized.cookie.name, locale, {
        path: normalized.cookie.path,
        domain: normalized.cookie.domain,
        maxAge: input.maxAge ?? normalized.cookie.maxAge,
        sameSite: normalized.cookie.sameSite,
        secure: resolveCookieSecure(normalized.cookie.secure, input),
        httpOnly: input.httpOnly ?? normalized.cookie.httpOnly,
      });
    } else {
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

/** @template {string} Locale @template Linguini @param {LinguiniRuntime<Locale, Linguini>} runtime @param {LinguiniWebOptions} [options] @param {LinguiniWebEnvironment} [environment] @returns {LinguiniWeb<Locale, Linguini>} */
export function createWebI18n(runtime, options = {}, environment = {}) {
  const web = createWebLocaleI18n(runtime, options, environment);

  /** @param {Locale} locale @param {Record<string, unknown>} [contextInput] @returns {LinguiniRequestContext<Locale, Linguini>} */
  function createRequestContext(locale, contextInput = {}) {
    const resolved = web.matchLocale(locale) ?? runtime.baseLocale;
    const messages = runtime.createLinguini(resolved);
    return {
      locale: resolved,
      baseLocale: runtime.baseLocale,
      locales: runtime.locales,
      direction: web.getTextDirection(resolved),
      textDirection: web.getTextDirection(resolved),
      lang: resolved,
      messages,
      l: messages,
      htmlAttrs: web.htmlAttrs(resolved),
      localizeHref: (href, nextLocale = resolved, input = contextInput) => web.localizeHref(href, nextLocale, input),
      localizeUrl: (url, nextLocale = resolved, input = contextInput) => web.localizeUrl(url, nextLocale, input),
      shouldLocalizeHref: (href, input = contextInput) => web.shouldLocalizeHref(href, input),
      shouldLocalizeLink: (href, attributes = {}, input = contextInput) => web.shouldLocalizeLink(href, attributes, input),
      localizeHrefAttribute: (href, nextLocale = resolved, input = contextInput) => web.localizeHrefAttribute(href, nextLocale, input),
      delocalizeUrl: (url, input = contextInput) => web.delocalizeUrl(url, input),
      alternateLinks: (url, input = contextInput) => web.alternateLinks(url, input),
    };
  }

  return {
    ...web,
    createLinguini: runtime.createLinguini,
    resolveRequest: async (request, input = {}) => {
      const requestInput = inputFromRequest(request, input);
      const locale = web.resolveLocaleSync(requestInput);
      await runtime.prepareLinguini?.(locale);
      return createRequestContext(locale, requestInput);
    },
    createRequestContext,
  };
}


/** @param {Request} request @param {Record<string, unknown>} [input] @returns {Record<string, unknown>} */
function inputFromRequest(request, input = {}) {
  /** @type {Record<string, unknown>} */
  const requestInput = { ...input };
  requestInput.url ??= request.url;
  requestInput.currentUrl ??= requestInput.url;
  requestInput.headers ??= request.headers;
  requestInput.cookie ??= readHeader(requestInput.headers, "cookie") ?? readHeader(request.headers, "cookie") ?? undefined;
  return requestInput;
}

/** @param {unknown} headers @param {string} name @returns {string | undefined} */
function readHeader(headers, name) {
  if (!headers) return undefined;
  const getter = (/** @type {{ get?: (header: string) => string | null | undefined }} */ (headers)).get;
  if (typeof getter !== "function") return undefined;
  try {
    return getter.call(headers, name) ?? undefined;
  } catch {
    return undefined;
  }
}
/** @param {string} baseLocale @param {LinguiniWebOptions} options @param {LinguiniWebEnvironment} environment @returns {LinguiniWebLocale<string>["options"]} */
function normalizeOptions(baseLocale, options, environment) {
  const sources = options.locale?.sources ?? DEFAULT_SOURCES;
  const base = normalizeBasePath(environment.base ?? "");
  const origin = environment.origin;
  const cookiePath = options.cookie?.path === "auto" || options.cookie?.path === undefined
    ? base || "/"
    : options.cookie.path;
  const secure = options.cookie?.secure ?? "auto";
  return {
    routing: {
      localePrefix: options.routing?.localePrefix ?? "except-default",
      canonical: options.routing?.canonical ?? "redirect",
    },
    locale: {
      sources,
      switch: options.locale?.switch ?? localeSwitchFromSources(sources),
    },
    cookie: {
      name: options.cookie?.name ?? "LINGUINI_LOCALE",
      path: cookiePath,
      domain: options.cookie?.domain,
      maxAge: options.cookie?.maxAge ?? DEFAULT_COOKIE_MAX_AGE,
      sameSite: options.cookie?.sameSite ?? "lax",
      secure,
      httpOnly: Boolean(options.cookie?.httpOnly ?? false),
    },
    localStorage: { key: options.localStorage?.key ?? "LINGUINI_LOCALE" },
    links: { mode: options.links?.mode ?? "runtime" },
    routes: { exclude: options.routes?.exclude ?? [] },
    environment: { base, origin },
    baseLocale,
  };
}

/** @param {readonly LocaleSource[]} sources @returns {LocaleSwitchPlan} */
function localeSwitchFromSources(sources) {
  return {
    writesPath: sources.includes("path"),
    writesCookie: sources.includes("cookie"),
    writesLocalStorage: sources.includes("local-storage"),
  };
}


/** @template {string} Locale @param {readonly Locale[]} locales @param {unknown} value @returns {Locale | undefined} */
function matchLocaleValue(locales, value) {
  const candidates = Array.isArray(value) ? value : [value];
  for (const candidate of candidates) {
    if (typeof candidate !== "string") continue;
    const exact = locales.find(
      (locale) => locale.toLowerCase() === candidate.toLowerCase(),
    );
    if (exact) return exact;
  }
  return undefined;
}

/** @template {string} Locale @param {readonly Locale[]} locales @param {unknown} value @returns {Locale | undefined} */
function matchExactLocale(locales, value) {
  if (typeof value !== "string") return undefined;
  return locales.find((locale) => locale.toLowerCase() === value.toLowerCase());
}


/** @template {string} Locale @param {string} pathname @param {readonly Locale[]} locales @returns {string} */
function stripLeadingLocale(pathname, locales) {
  const parts = pathname.split("/").filter(Boolean);
  return matchExactLocale(locales, parts[0]) ? ensureSlash(parts.slice(1).join("/")) : ensureSlash(parts.join("/"));
}

/** @param {string} pathname @param {string} basePath @returns {string} */
function stripBasePath(pathname, basePath) {
  const normalizedPath = ensureSlash(pathname);
  const normalizedBase = basePath && basePath !== "/" ? ensureSlash(basePath).replace(/\/$/, "") : "";
  if (!normalizedBase) return normalizedPath;
  if (normalizedPath === normalizedBase) return "/";
  return normalizedPath.startsWith(`${normalizedBase}/`) ? normalizedPath.slice(normalizedBase.length) || "/" : normalizedPath;
}

/** @param {...(string | undefined)} parts @returns {string} */
function joinPath(...parts) {
  const joined = parts
    .filter((part) => part !== undefined && part !== "")
    .map((part) => String(part).replace(/^\/+|\/+$/g, ""))
    .filter(Boolean)
    .join("/");
  return `/${joined}`.replace(/\/+/g, "/");
}

/** @param {string} pathname @returns {string} */
function ensureSlash(pathname) {
  const value = String(pathname || "/");
  return value.startsWith("/") ? value : `/${value}`;
}

/** @param {ReturnType<typeof normalizeOptions>} options @param {string} locale @returns {boolean} */
function shouldPrefixLocale(options, locale) {
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


/** @param {ReturnType<typeof normalizeOptions>} options @param {Record<string, unknown>} [input] @returns {URL} */
function resolveBaseUrl(options, input = {}) {
  const origin = parseUrl(input.origin ?? options.environment.origin ?? browserLocationHref() ?? FALLBACK_URL, FALLBACK_URL);
  const current = input.currentUrl ?? input.url;
  return current === undefined ? origin : parseUrl(current, origin);
}

/** @param {string} value @returns {string} */
function normalizeBasePath(value) {
  if (!value || value === "/") return "";
  return ensureSlash(value).replace(/\/+$/, "");
}

/** @param {string | URL | undefined} origin @returns {string} */
function resolveEnvironmentProtocol(origin) {
  try {
    return parseUrl(origin ?? browserLocationHref() ?? FALLBACK_URL, FALLBACK_URL).protocol;
  } catch {
    return "http:";
  }
}

/** @param {boolean | "auto"} policy @param {Record<string, unknown>} input @returns {boolean} */
function resolveCookieSecure(policy, input) {
  if (typeof input.secure === "boolean") return input.secure;
  if (policy !== "auto") return policy;
  return resolveEnvironmentProtocol(/** @type {string | URL | undefined} */ (input.origin)) === "https:";
}

/** @param {string | URL} url @param {ReturnType<typeof normalizeOptions>} options @param {Record<string, unknown>} [input] @returns {URL} */
function parseRuntimeUrl(url, options, input = {}) {
  return parseUrl(url, resolveBaseUrl(options, input));
}

/** @param {unknown} value @param {string | URL} base @returns {URL} */
function parseUrl(value, base) {
  try {
    return new URL(String(value), String(base));
  } catch {
    throw new TypeError(INVALID_URL_MESSAGE);
  }
}

/** @param {URL} url @param {string} origin @returns {boolean} */
function isInternalHttpUrl(url, origin) {
  return ["http:", "https:"].includes(url.protocol) && url.origin === origin;
}

function browserLocationHref() {
  const href = (/** @type {{ location?: { href?: unknown } }} */ (globalThis)).location?.href;
  return typeof href === "string" ? href : undefined;
}
//# sourceMappingURL=web.js.map
