const DEFAULT_STRATEGY = ["url", "cookie", "localStorage", "preferredLanguage", "baseLocale"] as const;
const DEFAULT_COOKIE_MAX_AGE = 60 * 60 * 24 * 365;

export type TextDirection = "ltr" | "rtl";
export type LocaleStrategy = "url" | "cookie" | "localStorage" | "header" | "navigator" | "preferredLanguage" | "globalVariable" | "baseLocale" | `custom-${string}`;

export interface AlternateLink {
  rel: "alternate";
  hreflang: string;
  href: string;
}

export interface LinguiniWebOptions {
  strategy?: readonly LocaleStrategy[];
  cookieName?: string;
  localStorageKey?: string;
  prefixDefaultLocale?: boolean;
  basePath?: string;
  trailingSlash?: "ignore" | "always" | "never" | "directory";
  cookiePath?: string;
  cookieDomain?: string;
  cookieMaxAge?: number;
  cookieSameSite?: "lax" | "strict" | "none";
  cookieSecure?: boolean;
  cookieHttpOnly?: boolean;
  globalVariableName?: string;
  exclude?: readonly (string | RegExp | ((url: URL) => boolean))[];
  redirect?: boolean;
  origin?: string;
  localizeLinks?: boolean;
}

export interface LinguiniRuntime<Locale extends string = string, Linguini = unknown> {
  locales: readonly Locale[];
  baseLocale: Locale;
  localeDirections?: Readonly<Record<Locale, TextDirection>>;
  createLinguini(locale: Locale): Linguini;
  normalizeLocale?(locale: unknown): Locale | undefined;
  getTextDirection?(locale: Locale): TextDirection;
}

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
  localizeHrefAttribute(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  localizeMarkupLinks(html: string, locale?: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
}

export interface LinguiniWeb<Locale extends string = string, Linguini = unknown> extends LinguiniRuntime<Locale, Linguini> {
  options: Required<Pick<LinguiniWebOptions, "strategy" | "cookieName" | "localStorageKey" | "prefixDefaultLocale" | "basePath" | "trailingSlash" | "cookiePath" | "cookieMaxAge" | "cookieSameSite" | "cookieSecure" | "cookieHttpOnly" | "exclude" | "redirect" | "localizeLinks">> & LinguiniWebOptions;
  matchLocale(locale: unknown): Locale | undefined;
  resolveLocale(input?: Record<string, unknown>): Promise<Locale>;
  resolveLocaleSync(input?: Record<string, unknown>): Locale;
  resolveRequest(request: Request, input?: Record<string, unknown>): Promise<LinguiniRequestContext<Locale, Linguini>>;
  createRequestContext(locale: Locale, input?: Record<string, unknown>): LinguiniRequestContext<Locale, Linguini>;
  localizeUrl(url: string | URL, locale: Locale, input?: Record<string, unknown>): URL;
  localizeHref(href: string, locale: Locale, input?: Record<string, unknown>): string;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale: Locale, input?: Record<string, unknown>): string;
  localizeMarkupLinks(html: string, locale: Locale, input?: Record<string, unknown>): string;
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

export function createWebI18n<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions = {}): LinguiniWeb<Locale, Linguini> {
  const normalized = normalizeOptions({ baseLocale: runtime.baseLocale, ...options });
  const matchLocale = (locale: unknown) => runtime.normalizeLocale?.(locale) ?? matchLocaleValue(runtime.locales, locale);
  const getTextDirection = (locale: Locale) => runtime.getTextDirection?.(locale) ?? runtime.localeDirections?.[locale] ?? "ltr";

  function createRequestContext(locale: Locale, contextInput: Record<string, unknown> = {}): LinguiniRequestContext<Locale, Linguini> {
    const resolved = matchLocale(locale) ?? runtime.baseLocale;
    const messages = runtime.createLinguini(resolved);
    return {
      locale: resolved,
      baseLocale: runtime.baseLocale,
      locales: runtime.locales,
      direction: getTextDirection(resolved),
      textDirection: getTextDirection(resolved),
      lang: resolved,
      messages,
      l: messages,
      htmlAttrs: htmlAttrs(resolved),
      localizeHref: (href, nextLocale = resolved, input = contextInput) => localizeHref(href, nextLocale, input),
      localizeUrl: (url, nextLocale = resolved, input = contextInput) => localizeUrl(url, nextLocale, input),
      shouldLocalizeHref: (href, input = contextInput) => shouldLocalizeHref(href, input),
      localizeHrefAttribute: (href, nextLocale = resolved, input = contextInput) => localizeHrefAttribute(href, nextLocale, input),
      localizeMarkupLinks: (html, nextLocale = resolved, input = contextInput) => localizeMarkupLinks(html, nextLocale, input),
      delocalizeUrl: (url, input = contextInput) => delocalizeUrl(url, input),
      alternateLinks: (url, input = contextInput) => alternateLinks(url, input),
    };
  }

  function resolveLocaleSync(input: Record<string, unknown> = {}) {
    for (const strategy of normalized.strategy) {
      const locale = readStrategy(strategy, input, normalized);
      const resolved = matchLocale(locale);
      if (resolved) return resolved;
    }
    return runtime.baseLocale;
  }

  function localizeUrl(url: string | URL, locale: Locale, input: Record<string, unknown> = {}) {
    const resolved = matchLocale(locale) ?? runtime.baseLocale;
    const copy = new URL(String(url), String(input.currentUrl ?? input.url ?? input.origin ?? normalized.origin ?? "http://localhost"));
    const path = stripBasePath(copy.pathname, normalized.basePath);
    const withoutLocale = stripLeadingLocale(path, runtime.locales, matchLocale);
    copy.pathname = applyTrailingSlash(joinPath(normalized.basePath, shouldPrefixLocale(normalized, resolved) ? joinPath("/", resolved, withoutLocale) : withoutLocale), normalized.trailingSlash);
    return copy;
  }

  function localizeHref(href: string, locale: Locale, input: Record<string, unknown> = {}) {
    const url = localizeUrl(href, locale, input);
    if (/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(String(href))) return url.toString();
    return `${url.pathname}${url.search}${url.hash}`;
  }

  function shouldLocalizeHref(href: string, input: Record<string, unknown> = {}) {
    if (normalized.localizeLinks === false) return false;
    const value = String(href ?? "").trim();
    if (!value || value.startsWith("#") || value.startsWith("//")) return false;
    const scheme = value.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);
    if (scheme && !["http", "https"].includes(scheme[1].toLowerCase())) return false;
    let parsed: URL;
    try {
      parsed = new URL(value, String(input.currentUrl ?? input.url ?? input.origin ?? normalized.origin ?? "http://localhost"));
    } catch {
      return false;
    }
    if (!["http:", "https:"].includes(parsed.protocol)) return false;
    if (parsed.origin !== currentOrigin(normalized, input)) return false;
    return !shouldExclude(parsed, input);
  }

  function localizeHrefAttribute(href: string, locale: Locale, input: Record<string, unknown> = {}) {
    return shouldLocalizeHref(href, input) ? localizeHref(href, locale, input) : href;
  }

  function localizeMarkupLinks(html: string, locale: Locale, input: Record<string, unknown> = {}) {
    if (normalized.localizeLinks === false) return String(html);
    return String(html).replace(/<a\b[^>]*>/gi, (tag) => {
      if (/\sdownload(?:\s|=|>)/i.test(tag) || /\sdata-linguini-(?:ignore|no-localize)(?:\s|=|>)/i.test(tag)) return tag;
      return tag.replace(/\bhref\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/i, (attribute, rawValue) => {
        const quote = rawValue[0] === "\"" || rawValue[0] === "'" ? rawValue[0] : "";
        const href = quote ? rawValue.slice(1, -1) : rawValue;
        const localized = localizeHrefAttribute(href, locale, input);
        return localized === href ? attribute : `href=${quote}${localized}${quote}`;
      });
    });
  }

  function delocalizeUrl(url: string | URL, input: Record<string, unknown> = {}) {
    const copy = new URL(String(url), String(input.currentUrl ?? input.url ?? input.origin ?? normalized.origin ?? "http://localhost"));
    copy.pathname = joinPath(normalized.basePath, stripLeadingLocale(stripBasePath(copy.pathname, normalized.basePath), runtime.locales, matchLocale));
    return copy;
  }

  function delocalizePathname(pathname: string, input: Record<string, unknown> = {}) {
    return delocalizeUrl(pathname, input).pathname;
  }

  function alternateLinks(url: string | URL, input: Record<string, unknown> = {}) {
    const parsed = new URL(String(url), String(input.currentUrl ?? input.url ?? input.origin ?? normalized.origin ?? "http://localhost"));
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
    if (normalized.redirect === false || shouldExclude(url, input)) return undefined;
    const parsed = new URL(String(url), String(input.origin ?? normalized.origin ?? "http://localhost"));
    const canonical = localizeUrl(parsed, locale, input);
    return canonical.pathname === parsed.pathname ? undefined : `${canonical.pathname}${canonical.search}${canonical.hash}`;
  }

  function shouldExclude(url: string | URL, input: Record<string, unknown> = {}) {
    const parsed = new URL(String(url), String(input.origin ?? normalized.origin ?? "http://localhost"));
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
    const sink = target as { headers?: Headers; cookies?: { set(name: string, value: string, options?: Record<string, unknown>): void }; setHeaders?: (headers: Record<string, string>) => void };
    if (sink.setHeaders) sink.setHeaders({ "set-cookie": cookie });
    else if (sink.headers?.append) sink.headers.append("set-cookie", cookie);
    else if (sink.cookies?.set) {
      sink.cookies.set(normalized.cookieName, locale, {
        path: normalized.cookiePath,
        domain: normalized.cookieDomain,
        maxAge: input.maxAge ?? normalized.cookieMaxAge,
        sameSite: normalized.cookieSameSite,
        secure: input.secure ?? normalized.cookieSecure,
        httpOnly: input.httpOnly ?? normalized.cookieHttpOnly,
      });
    }
  }

  return {
    ...runtime,
    options: normalized,
    matchLocale,
    resolveLocale: async (input = {}) => resolveLocaleSync(input),
    resolveLocaleSync,
    resolveRequest: async (request, input = {}) => {
      const requestInput = inputFromRequest(request, input);
      return createRequestContext(resolveLocaleSync(requestInput), requestInput);
    },
    createRequestContext,
    localizeUrl,
    localizeHref,
    shouldLocalizeHref,
    localizeHrefAttribute,
    localizeMarkupLinks,
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
  if (typeof getter === "function") return getter.call(headers, name) ?? undefined;
  const record = headers as Record<string, unknown>;
  const value = record[name] ?? record[name.toLowerCase()];
  if (Array.isArray(value)) return value.join(", ");
  return typeof value === "string" ? value : undefined;
}

function normalizeOptions(options: LinguiniWebOptions & { baseLocale: string }) {
  return {
    strategy: options.strategy ?? DEFAULT_STRATEGY,
    cookieName: options.cookieName ?? "LINGUINI_LOCALE",
    localStorageKey: options.localStorageKey ?? "LINGUINI_LOCALE",
    prefixDefaultLocale: Boolean(options.prefixDefaultLocale ?? false),
    basePath: options.basePath ?? "",
    trailingSlash: options.trailingSlash ?? "ignore",
    cookiePath: options.cookiePath ?? "/",
    cookieDomain: options.cookieDomain,
    cookieMaxAge: options.cookieMaxAge ?? DEFAULT_COOKIE_MAX_AGE,
    cookieSameSite: options.cookieSameSite ?? "lax",
    cookieSecure: Boolean(options.cookieSecure ?? false),
    cookieHttpOnly: Boolean(options.cookieHttpOnly ?? false),
    globalVariableName: options.globalVariableName,
    exclude: options.exclude ?? [],
    redirect: options.redirect ?? true,
    origin: options.origin,
    localizeLinks: options.localizeLinks ?? true,
    baseLocale: options.baseLocale,
  };
}

function readStrategy(strategy: LocaleStrategy, input: Record<string, unknown>, options: ReturnType<typeof normalizeOptions>) {
  if (strategy === "baseLocale") return options.baseLocale;
  if (strategy === "url") return firstPathSegment(input.url as URL | string | undefined, options.basePath);
  if (strategy === "cookie") return readCookie(input.cookie as string | undefined, options.cookieName);
  if (strategy === "localStorage") {
    try {
      return (input.localStorage as Storage | undefined)?.getItem(options.localStorageKey) ?? undefined;
    } catch {
      return undefined;
    }
  }
  if (strategy === "header") return parseAcceptLanguage((input.headers as Headers | undefined)?.get("accept-language"))[0];
  if (strategy === "navigator") return (input.navigator as Navigator | undefined)?.languages?.[0] ?? (input.navigator as Navigator | undefined)?.language;
  return undefined;
}

function parseAcceptLanguage(header: string | null | undefined): string[] {
  if (!header) return [];
  return String(header)
    .split(",")
    .map((part) => part.trim().split(";")[0])
    .filter(Boolean);
}

function readCookie(source: string | undefined, name: string) {
  for (const cookie of String(source ?? "").split(";")) {
    const [rawName, ...rawValue] = cookie.trim().split("=");
    if (rawName === name) return decodeURIComponent(rawValue.join("="));
  }
  return undefined;
}

function matchLocaleValue<Locale extends string>(locales: readonly Locale[], value: unknown): Locale | undefined {
  const candidates = Array.isArray(value) ? value : [value];
  for (const candidate of candidates) {
    if (typeof candidate !== "string") continue;
    let tag = candidate;
    while (tag) {
      const exact = locales.find((locale) => locale.toLowerCase() === tag.toLowerCase());
      if (exact) return exact;
      const dash = tag.lastIndexOf("-");
      tag = dash > 0 ? tag.slice(0, dash) : "";
    }
  }
  return undefined;
}

function firstPathSegment(url: URL | string | undefined, basePath = "") {
  if (!url) return undefined;
  const parsed = new URL(String(url), "http://localhost");
  return stripBasePath(parsed.pathname, basePath).split("/").filter(Boolean)[0];
}

function stripLeadingLocale<Locale extends string>(pathname: string, locales: readonly Locale[], matchLocale: (locale: unknown) => Locale | undefined) {
  const parts = pathname.split("/").filter(Boolean);
  return matchLocale(parts[0]) ? ensureSlash(parts.slice(1).join("/")) : ensureSlash(parts.join("/"));
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
  return options.prefixDefaultLocale || locale !== options.baseLocale;
}

function matchesRoute(pattern: string | RegExp | ((url: URL) => boolean), url: URL) {
  if (typeof pattern === "function") return Boolean(pattern(url));
  if (pattern instanceof RegExp) return pattern.test(url.pathname);
  if (pattern.endsWith("/**")) return url.pathname.startsWith(pattern.slice(0, -3));
  return url.pathname === pattern;
}

function currentOrigin(options: ReturnType<typeof normalizeOptions>, input: Record<string, unknown> = {}) {
  return new URL(String(input.currentUrl ?? input.url ?? input.origin ?? options.origin ?? "http://localhost")).origin;
}
