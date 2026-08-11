const DEFAULT_SOURCES = ["path", "cookie", "accept-language"] as const;
const DEFAULT_COOKIE_MAX_AGE = 60 * 60 * 24 * 365;
const FALLBACK_URL = "http://localhost";
const INVALID_URL_MESSAGE = "Linguini: invalid URL";

export type TextDirection = "ltr" | "rtl";
export type LocaleSource = "path" | "cookie" | "local-storage" | "accept-language";
export type LocalePrefixMode = "always" | "except-default" | "never";

export interface LocaleSwitchPlan {
  writesPath: boolean;
  writesCookie: boolean;
  writesLocalStorage: boolean;
}

export interface AlternateLink {
  rel: "alternate";
  hreflang: string;
  href: string;
}

export interface LinkLocalizationAttributes {
  download?: boolean;
  ignored?: boolean;
  rel?: string | null;
}

export interface LinguiniWebOptions {
  sources?: readonly LocaleSource[];
  localeSwitch?: LocaleSwitchPlan;
  cookieName?: string;
  localStorageKey?: string;
  localePrefix?: LocalePrefixMode;
  /** @deprecated Use localePrefix for the closed three-mode policy. */
  prefixDefaultLocale?: boolean;
  basePath?: string;
  trailingSlash?: "ignore" | "always" | "never";
  cookiePath?: string;
  cookieDomain?: string;
  cookieMaxAge?: number;
  cookieSameSite?: "lax" | "strict" | "none";
  cookieSecure?: boolean;
  cookieHttpOnly?: boolean;
  exclude?: readonly (string | RegExp | ((url: URL) => boolean))[];
  redirect?: boolean;
  origin?: string;
  localizeLinks?: boolean;
}

export interface LinguiniLocaleRuntime<Locale extends string = string> {
  locales: readonly Locale[];
  baseLocale: Locale;
  localeDirections?: Readonly<Record<Locale, TextDirection>>;
  normalizeLocale?(locale: unknown): Locale | undefined;
  getTextDirection?(locale: Locale): TextDirection;
}

export interface LinguiniRuntime<Locale extends string = string, Linguini = unknown> extends LinguiniLocaleRuntime<Locale> {
  createLinguini(locale: Locale): Linguini;
}

/**
 * Localization and delocalization leave external and non-HTTP URLs unchanged. Explicit URL
 * operations throw `TypeError("Linguini: invalid URL")` for malformed input; link-safety helpers
 * fail closed and preserve the original href instead.
 */
export interface LinguiniRequestContext<Locale extends string = string, Linguini = unknown> {
  locale: Locale;
  baseLocale: Locale;
  locales: readonly Locale[];
  direction: TextDirection;
  textDirection: TextDirection;
  lang: Locale;
  messages: Linguini;
  l: Linguini;
  htmlAttrs: { lang: Locale; dir: TextDirection };
  localizeHref(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  localizeUrl(url: string | URL, locale?: Locale, input?: Record<string, unknown>): URL;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  shouldLocalizeLink(href: string, attributes?: LinkLocalizationAttributes, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
}

/**
 * Localization and delocalization leave external and non-HTTP URLs unchanged. Explicit URL
 * operations throw `TypeError("Linguini: invalid URL")` for malformed input; link-safety helpers
 * fail closed and preserve the original href instead.
 */
export interface LinguiniWebLocale<Locale extends string = string> extends LinguiniLocaleRuntime<Locale> {
  options: Required<Pick<LinguiniWebOptions, "sources" | "localeSwitch" | "cookieName" | "localStorageKey" | "localePrefix" | "prefixDefaultLocale" | "basePath" | "trailingSlash" | "cookiePath" | "cookieMaxAge" | "cookieSameSite" | "cookieSecure" | "cookieHttpOnly" | "exclude" | "redirect" | "localizeLinks">> & LinguiniWebOptions;
  matchLocale(locale: unknown): Locale | undefined;
  resolveLocale(input?: Record<string, unknown>): Promise<Locale>;
  resolveLocaleSync(input?: Record<string, unknown>): Locale;
  localizeUrl(url: string | URL, locale: Locale, input?: Record<string, unknown>): URL;
  localizeHref(href: string, locale: Locale, input?: Record<string, unknown>): string;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  shouldLocalizeLink(href: string, attributes?: LinkLocalizationAttributes, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  delocalizePathname(pathname: string, input?: Record<string, unknown>): string;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
  htmlAttrs(locale: Locale): { lang: Locale; dir: TextDirection };
  getTextDirection(locale: Locale): TextDirection;
  getCanonicalRedirect(url: string | URL, locale: Locale, input?: Record<string, unknown>): string | undefined;
  shouldExclude(url: string | URL, input?: Record<string, unknown>): boolean;
  setLocaleCookie(target: unknown, locale: Locale, input?: Record<string, unknown>): void;
  serializeLocaleCookie(locale: Locale, input?: Record<string, unknown>): string;
}

export interface LinguiniWeb<Locale extends string = string, Linguini = unknown> extends LinguiniWebLocale<Locale> {
  createLinguini(locale: Locale): Linguini;
  resolveRequest(request: Request, input?: Record<string, unknown>): Promise<LinguiniRequestContext<Locale, Linguini>>;
  createRequestContext(locale: Locale, input?: Record<string, unknown>): LinguiniRequestContext<Locale, Linguini>;
}

export function createWebLocaleI18n<Locale extends string>(runtime: LinguiniLocaleRuntime<Locale>, options: LinguiniWebOptions = {}): LinguiniWebLocale<Locale> {
  const normalized = normalizeOptions({ baseLocale: runtime.baseLocale, ...options });
  const matchLocale = (locale: unknown) => runtime.normalizeLocale?.(locale) ?? matchLocaleValue(runtime.locales, locale);
  const getTextDirection = (locale: Locale) => runtime.getTextDirection?.(locale) ?? runtime.localeDirections?.[locale] ?? "ltr";

  function resolveLocaleSync(input: Record<string, unknown> = {}) {
    for (const source of normalized.sources) {
      const resolved = resolveLocaleSource(
        source,
        input,
        normalized,
        runtime.locales,
        runtime.baseLocale,
        matchLocale,
      );
      if (resolved) return resolved;
    }
    return runtime.baseLocale;
  }

  function localizeUrl(url: string | URL, locale: Locale, input: Record<string, unknown> = {}) {
    const resolved = matchLocale(locale) ?? runtime.baseLocale;
    const base = resolveBaseUrl(normalized, input);
    const copy = parseUrl(url, base);
    if (!isInternalHttpUrl(copy, base.origin)) return copy;
    return localizeParsedUrl(copy, resolved);
  }

  function localizeParsedUrl(copy: URL, locale: Locale) {
    const path = stripBasePath(copy.pathname, normalized.basePath);
    // In `never` mode locale-looking segments are ordinary application paths.
    // Do not strip them while normalizing a URL that should remain unprefixed.
    const withoutLocale = normalized.localePrefix === "never"
      ? path
      : stripLeadingLocale(path, runtime.locales);
    copy.pathname = applyTrailingSlash(joinPath(normalized.basePath, shouldPrefixLocale(normalized, locale) ? joinPath("/", locale, withoutLocale) : withoutLocale), normalized.trailingSlash);
    return copy;
  }

  function localizeHref(href: string, locale: Locale, input: Record<string, unknown> = {}) {
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

  function shouldLocalizeHref(href: string, input: Record<string, unknown> = {}) {
    if (normalized.localizeLinks === false) return false;
    const value = String(href ?? "").trim();
    if (!value || value.startsWith("#") || value.startsWith("//")) return false;
    const scheme = value.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);
    if (scheme && !["http", "https"].includes(scheme[1].toLowerCase())) return false;
    let parsed: URL;
    let base: URL;
    try {
      base = resolveBaseUrl(normalized, input);
      parsed = parseUrl(value, base);
    } catch {
      return false;
    }
    if (!isInternalHttpUrl(parsed, base.origin)) return false;
    return !shouldExclude(parsed, input);
  }

  function localizeHrefAttribute(href: string, locale: Locale, input: Record<string, unknown> = {}) {
    return shouldLocalizeHref(href, input) ? localizeHref(href, locale, input) : href;
  }

  function shouldLocalizeLink(
    href: string,
    attributes: LinkLocalizationAttributes = {},
    input: Record<string, unknown> = {},
  ) {
    if (attributes.download || attributes.ignored) return false;
    const relations = String(attributes.rel ?? "").toLowerCase().split(/\s+/);
    if (relations.includes("external")) return false;
    return shouldLocalizeHref(href, input);
  }

  function delocalizeUrl(url: string | URL, input: Record<string, unknown> = {}) {
    const base = resolveBaseUrl(normalized, input);
    const copy = parseUrl(url, base);
    if (!isInternalHttpUrl(copy, base.origin)) return copy;
    if (normalized.localePrefix === "never") return copy;
    copy.pathname = joinPath(normalized.basePath, stripLeadingLocale(stripBasePath(copy.pathname, normalized.basePath), runtime.locales));
    return copy;
  }

  function delocalizePathname(pathname: string, input: Record<string, unknown> = {}) {
    return delocalizeUrl(pathname, input).pathname;
  }

  function alternateLinks(url: string | URL, input: Record<string, unknown> = {}) {
    const parsed = parseRuntimeUrl(url, normalized, input);
    const links: AlternateLink[] = runtime.locales.map((locale) => ({
      rel: "alternate" as const,
      hreflang: locale,
      href: localizeUrl(parsed, locale, input).toString(),
    }));
    links.push({ rel: "alternate", hreflang: "x-default", href: localizeUrl(parsed, runtime.baseLocale, input).toString() });
    return links;
  }

  function htmlAttrs(locale: Locale) {
    const resolved = matchLocale(locale) ?? runtime.baseLocale;
    return { lang: resolved, dir: getTextDirection(resolved) };
  }

  function getCanonicalRedirect(url: string | URL, locale: Locale, input: Record<string, unknown> = {}) {
    if (normalized.localePrefix === "never" || normalized.redirect === false || shouldExclude(url, input)) return undefined;
    const parsed = parseRuntimeUrl(url, normalized, input);
    const canonical = localizeUrl(parsed, locale, input);
    return canonical.pathname === parsed.pathname ? undefined : `${canonical.pathname}${canonical.search}${canonical.hash}`;
  }

  function shouldExclude(url: string | URL, input: Record<string, unknown> = {}) {
    const parsed = parseRuntimeUrl(url, normalized, input);
    return normalized.exclude.some((matcher) => matchesRoute(matcher, parsed));
  }

  function serializeLocaleCookie(locale: Locale, input: Record<string, unknown> = {}) {
    const parts = [`${normalized.cookieName}=${encodeURIComponent(locale)}`];
    parts.push(`Max-Age=${input.maxAge ?? normalized.cookieMaxAge}`);
    parts.push(`Path=${normalized.cookiePath}`);
    if (normalized.cookieDomain) parts.push(`Domain=${normalized.cookieDomain}`);
    parts.push(`SameSite=${normalized.cookieSameSite}`);
    if (input.secure ?? normalized.cookieSecure) parts.push("Secure");
    if (input.httpOnly ?? normalized.cookieHttpOnly) parts.push("HttpOnly");
    return parts.join("; ");
  }

  function setLocaleCookie(target: unknown, locale: Locale, input: Record<string, unknown> = {}) {
    const cookie = serializeLocaleCookie(locale, input);
    const sink = target as { headers?: Pick<Headers, "append">; cookies?: { set(name: string, value: string, options?: Record<string, unknown>): void } };
    if (sink.cookies?.set) {
      sink.cookies.set(normalized.cookieName, locale, {
        path: normalized.cookiePath,
        domain: normalized.cookieDomain,
        maxAge: input.maxAge ?? normalized.cookieMaxAge,
        sameSite: normalized.cookieSameSite,
        secure: input.secure ?? normalized.cookieSecure,
        httpOnly: input.httpOnly ?? normalized.cookieHttpOnly,
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

export function createWebI18n<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions = {}): LinguiniWeb<Locale, Linguini> {
  const web = createWebLocaleI18n(runtime, options);

  function createRequestContext(locale: Locale, contextInput: Record<string, unknown> = {}): LinguiniRequestContext<Locale, Linguini> {
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
      return createRequestContext(web.resolveLocaleSync(requestInput), requestInput);
    },
    createRequestContext,
  };
}

function inputFromRequest(request: Request, input: Record<string, unknown> = {}) {
  const requestInput: Record<string, unknown> = { ...input };
  requestInput.url ??= request.url;
  requestInput.currentUrl ??= requestInput.url;
  requestInput.headers ??= request.headers;
  requestInput.cookie ??= readHeader(requestInput.headers, "cookie") ?? readHeader(request.headers, "cookie") ?? undefined;
  return requestInput;
}

function readHeader(headers: unknown, name: string) {
  if (!headers) return undefined;
  const getter = (headers as { get?: (header: string) => string | null | undefined }).get;
  if (typeof getter !== "function") return undefined;
  try {
    return getter.call(headers, name) ?? undefined;
  } catch {
    return undefined;
  }
}

function normalizeOptions(options: LinguiniWebOptions & { baseLocale: string }) {
  const sources = options.sources ?? DEFAULT_SOURCES;
  const localePrefix = options.localePrefix ?? (options.prefixDefaultLocale ? "always" : "except-default");
  return {
    sources,
    localeSwitch: options.localeSwitch ?? localeSwitchFromSources(sources),
    cookieName: options.cookieName ?? "LINGUINI_LOCALE",
    localStorageKey: options.localStorageKey ?? "LINGUINI_LOCALE",
    localePrefix,
    prefixDefaultLocale: localePrefix === "always",
    basePath: options.basePath ?? "",
    trailingSlash: options.trailingSlash ?? "ignore",
    cookiePath: options.cookiePath ?? "/",
    cookieDomain: options.cookieDomain,
    cookieMaxAge: options.cookieMaxAge ?? DEFAULT_COOKIE_MAX_AGE,
    cookieSameSite: options.cookieSameSite ?? "lax",
    cookieSecure: Boolean(options.cookieSecure ?? false),
    cookieHttpOnly: Boolean(options.cookieHttpOnly ?? false),
    exclude: options.exclude ?? [],
    redirect: options.redirect ?? true,
    origin: options.origin,
    localizeLinks: options.localizeLinks ?? true,
    baseLocale: options.baseLocale,
  };
}

function localeSwitchFromSources(sources: readonly LocaleSource[]): LocaleSwitchPlan {
  return {
    writesPath: sources.includes("path"),
    writesCookie: sources.includes("cookie"),
    writesLocalStorage: sources.includes("local-storage"),
  };
}

function resolveLocaleSource<Locale extends string>(
  source: LocaleSource,
  input: Record<string, unknown>,
  options: ReturnType<typeof normalizeOptions>,
  locales: readonly Locale[],
  baseLocale: Locale,
  matchLocale: (locale: unknown) => Locale | undefined,
): Locale | undefined {
  if (source === "path") {
    return matchExactLocale(
      locales,
      firstPathSegment(input.url as URL | string | undefined, options.basePath),
    );
  }
  if (source === "cookie") {
    return matchLocale(readCookie(input.cookie as string | undefined, options.cookieName));
  }
  if (source === "local-storage") {
    try {
      return matchLocale(
        (input.localStorage as Storage | undefined)?.getItem(options.localStorageKey) ?? undefined,
      );
    } catch {
      return undefined;
    }
  }
  const header = readHeader(input.headers, "accept-language");
  return header !== undefined
    ? resolveAcceptLanguage(locales, baseLocale, header)
    : resolveNavigatorLanguage(locales, baseLocale, input.navigator);
}

type LanguagePreference = {
  range: string;
  quality: number;
  index: number;
  specificity: number;
};

function parseAcceptLanguage(header: string | null | undefined): LanguagePreference[] {
  if (!header) return [];
  return String(header)
    .split(",")
    .flatMap((part, index) => {
      const [rawRange, ...parameters] = part.split(";");
      const range = rawRange.trim();
      if (!/^(?:\*|[A-Za-z]{1,8}(?:-[A-Za-z0-9]{1,8})*)$/.test(range)) return [];

      let quality = 1;
      for (const parameter of parameters) {
        const [rawName, ...rawValue] = parameter.split("=");
        if (rawName.trim().toLowerCase() !== "q") continue;
        const value = rawValue.join("=").trim();
        if (!/^(?:0(?:\.\d{0,3})?|1(?:\.0{0,3})?)$/.test(value)) return [];
        quality = Number(value);
        break;
      }

      return [{
        range,
        quality,
        index,
        specificity: range === "*" ? 0 : range.split("-").length,
      }];
    });
}

function resolveAcceptLanguage<Locale extends string>(
  locales: readonly Locale[],
  baseLocale: Locale,
  header: string | null | undefined,
): Locale | undefined {
  const preferences = parseAcceptLanguage(header);
  let best:
    | { locale: Locale; quality: number; preferenceIndex: number; base: boolean; localeIndex: number }
    | undefined;

  for (const [localeIndex, locale] of locales.entries()) {
    let preference: LanguagePreference | undefined;
    for (const candidate of preferences) {
      if (!languageRangeMatches(candidate.range, locale)) continue;
      if (
        !preference
        || candidate.specificity > preference.specificity
        || (
          candidate.specificity === preference.specificity
          && candidate.index < preference.index
        )
      ) {
        preference = candidate;
      }
    }
    if (!preference || preference.quality <= 0) continue;

    const candidate = {
      locale,
      quality: preference.quality,
      preferenceIndex: preference.index,
      base: locale.toLowerCase() === baseLocale.toLowerCase(),
      localeIndex,
    };
    if (
      !best
      || candidate.quality > best.quality
      || (
        candidate.quality === best.quality
        && candidate.preferenceIndex < best.preferenceIndex
      )
      || (
        candidate.quality === best.quality
        && candidate.preferenceIndex === best.preferenceIndex
        && candidate.base
        && !best.base
      )
      || (
        candidate.quality === best.quality
        && candidate.preferenceIndex === best.preferenceIndex
        && candidate.base === best.base
        && candidate.localeIndex < best.localeIndex
      )
    ) {
      best = candidate;
    }
  }

  return best?.locale;
}

function resolveNavigatorLanguage<Locale extends string>(
  locales: readonly Locale[],
  baseLocale: Locale,
  navigator: unknown,
): Locale | undefined {
  const preferences = readNavigatorLanguages(navigator);
  return preferences.length === 0
    ? undefined
    : resolveAcceptLanguage(locales, baseLocale, preferences.join(","));
}

function readNavigatorLanguages(value: unknown): string[] {
  if (!value || (typeof value !== "object" && typeof value !== "function")) return [];

  const preferences: string[] = [];
  try {
    const languages = (value as { languages?: unknown }).languages;
    if (Array.isArray(languages)) {
      for (const language of languages) {
        if (typeof language === "string") preferences.push(language);
      }
    }
  } catch {
    // Some browser capability proxies throw while reading navigator.languages.
  }

  try {
    const language = (value as { language?: unknown }).language;
    if (typeof language === "string") preferences.push(language);
  } catch {
    // Some browser capability proxies throw while reading navigator.language.
  }

  return preferences;
}

function languageRangeMatches(range: string, locale: string) {
  if (range === "*") return true;
  const normalizedRange = range.toLowerCase();
  const normalizedLocale = locale.toLowerCase();
  return normalizedRange === normalizedLocale
    || normalizedLocale.startsWith(`${normalizedRange}-`)
    || normalizedRange.startsWith(`${normalizedLocale}-`);
}

function readCookie(source: string | undefined, name: string) {
  for (const cookie of String(source ?? "").split(";")) {
    const [rawName, ...rawValue] = cookie.trim().split("=");
    if (rawName !== name) continue;
    try {
      return decodeURIComponent(rawValue.join("="));
    } catch {
      return undefined;
    }
  }
  return undefined;
}

function matchLocaleValue<Locale extends string>(locales: readonly Locale[], value: unknown): Locale | undefined {
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

function matchExactLocale<Locale extends string>(locales: readonly Locale[], value: unknown): Locale | undefined {
  if (typeof value !== "string") return undefined;
  return locales.find((locale) => locale.toLowerCase() === value.toLowerCase());
}

function firstPathSegment(url: URL | string | undefined, basePath = "") {
  if (!url) return undefined;
  let parsed: URL;
  try {
    parsed = parseUrl(url, FALLBACK_URL);
  } catch {
    return undefined;
  }
  return stripBasePath(parsed.pathname, basePath).split("/").filter(Boolean)[0];
}

function stripLeadingLocale<Locale extends string>(pathname: string, locales: readonly Locale[]) {
  const parts = pathname.split("/").filter(Boolean);
  return matchExactLocale(locales, parts[0]) ? ensureSlash(parts.slice(1).join("/")) : ensureSlash(parts.join("/"));
}

function stripBasePath(pathname: string, basePath: string) {
  const normalizedPath = ensureSlash(pathname);
  const normalizedBase = basePath && basePath !== "/" ? ensureSlash(basePath).replace(/\/$/, "") : "";
  if (!normalizedBase) return normalizedPath;
  if (normalizedPath === normalizedBase) return "/";
  return normalizedPath.startsWith(`${normalizedBase}/`) ? normalizedPath.slice(normalizedBase.length) || "/" : normalizedPath;
}

function joinPath(...parts: Array<string | undefined>) {
  const joined = parts
    .filter((part) => part !== undefined && part !== "")
    .map((part) => String(part).replace(/^\/+|\/+$/g, ""))
    .filter(Boolean)
    .join("/");
  return `/${joined}`.replace(/\/+/g, "/");
}

function ensureSlash(pathname: string) {
  const value = String(pathname || "/");
  return value.startsWith("/") ? value : `/${value}`;
}

function applyTrailingSlash(pathname: string, mode: string) {
  if (pathname === "/") return pathname;
  if (mode === "always") return pathname.endsWith("/") ? pathname : `${pathname}/`;
  if (mode === "never") return pathname.replace(/\/+$/, "");
  return pathname;
}

function shouldPrefixLocale(options: ReturnType<typeof normalizeOptions>, locale: string) {
  switch (options.localePrefix) {
    case "always":
      return true;
    case "never":
      return false;
    case "except-default":
    default:
      return locale !== options.baseLocale;
  }
}

function matchesRoute(pattern: string | RegExp | ((url: URL) => boolean), url: URL) {
  if (typeof pattern === "function") return Boolean(pattern(url));
  if (pattern instanceof RegExp) {
    if (!pattern.global && !pattern.sticky) return pattern.test(url.pathname);
    pattern.lastIndex = 0;
    try {
      return pattern.test(url.pathname);
    } finally {
      pattern.lastIndex = 0;
    }
  }
  if (pattern.endsWith("/**")) {
    const prefix = pattern.slice(0, -3).replace(/\/+$/, "");
    return !prefix || prefix === "/" || url.pathname === prefix || url.pathname.startsWith(`${prefix}/`);
  }
  return url.pathname === pattern;
}

function resolveBaseUrl(options: ReturnType<typeof normalizeOptions>, input: Record<string, unknown> = {}) {
  const origin = parseUrl(input.origin ?? options.origin ?? browserLocationHref() ?? FALLBACK_URL, FALLBACK_URL);
  const current = input.currentUrl ?? input.url;
  return current === undefined ? origin : parseUrl(current, origin);
}

function parseRuntimeUrl(url: string | URL, options: ReturnType<typeof normalizeOptions>, input: Record<string, unknown> = {}) {
  return parseUrl(url, resolveBaseUrl(options, input));
}

function parseUrl(value: unknown, base: string | URL) {
  try {
    return new URL(String(value), String(base));
  } catch {
    throw new TypeError(INVALID_URL_MESSAGE);
  }
}

function isInternalHttpUrl(url: URL, origin: string) {
  return ["http:", "https:"].includes(url.protocol) && url.origin === origin;
}

function browserLocationHref() {
  const href = (globalThis as { location?: { href?: unknown } }).location?.href;
  return typeof href === "string" ? href : undefined;
}
