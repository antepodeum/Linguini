const DEFAULT_COOKIE_NAME = "LINGUINI_LOCALE";
const DEFAULT_LOCAL_STORAGE_KEY = "LINGUINI_LOCALE";
const DEFAULT_STRATEGY = ["url", "cookie", "localStorage", "preferredLanguage", "baseLocale"];
const DEFAULT_COOKIE_MAX_AGE = 60 * 60 * 24 * 365;

const clientStrategies = new Map();
const serverStrategies = new Map();

export function defineCustomClientStrategy(name, strategy) {
  assertCustomStrategyName(name);
  if (!strategy || typeof strategy.getLocale !== "function" || typeof strategy.setLocale !== "function") {
    throw new TypeError("client locale strategies need getLocale() and setLocale() functions");
  }
  clientStrategies.set(name, strategy);
  return strategy;
}

export function defineCustomServerStrategy(name, strategy) {
  assertCustomStrategyName(name);
  if (!strategy || typeof strategy.getLocale !== "function") {
    throw new TypeError("server locale strategies need a getLocale() function");
  }
  serverStrategies.set(name, strategy);
  return strategy;
}

export function createWebI18n(runtime, options = {}) {
  const runtimeAdapter = createRuntimeAdapter(runtime);
  const normalizedOptions = normalizeOptions({ baseLocale: runtimeAdapter.baseLocale, ...options });

  return {
    ...runtimeAdapter,
    options: normalizedOptions,
    defineCustomClientStrategy,
    defineCustomServerStrategy,
    matchLocale: runtimeAdapter.matchLocale,
    resolveLocale: (input = {}) => resolveLocale(runtimeAdapter, normalizedOptions, input),
    resolveLocaleSync: (input = {}) => resolveLocaleSync(runtimeAdapter, normalizedOptions, input),
    resolveRequest: (request, input = {}) => resolveRequest(runtimeAdapter, normalizedOptions, request, input),
    createRequestContext: (locale, input = {}) => createRequestContext(runtimeAdapter, normalizedOptions, locale, input),
    localizeUrl: (url, locale, input = {}) => localizeUrl(runtimeAdapter, normalizedOptions, url, locale, input),
    localizeHref: (href, locale, input = {}) => localizeHref(runtimeAdapter, normalizedOptions, href, locale, input),
    shouldLocalizeHref: (href, input = {}) => shouldLocalizeHrefInternal(runtimeAdapter, normalizedOptions, href, input),
    localizeHrefAttribute: (href, locale, input = {}) => localizeHrefAttributeInternal(runtimeAdapter, normalizedOptions, href, locale, input),
    localizeMarkupLinks: (html, locale, input = {}) => localizeMarkupLinksInternal(runtimeAdapter, normalizedOptions, html, locale, input),
    delocalizeUrl: (url, input = {}) => delocalizeUrl(runtimeAdapter, normalizedOptions, url, input),
    delocalizePathname: (pathname, input = {}) => delocalizePathname(runtimeAdapter, normalizedOptions, pathname, input),
    extractLocaleFromUrl: (url, input = {}) => extractLocaleFromUrl(runtimeAdapter, normalizedOptions, url, input),
    alternateLinks: (url, input = {}) => alternateLinks(runtimeAdapter, normalizedOptions, url, input),
    htmlAttrs: (locale) => htmlAttrs(runtimeAdapter, locale),
    getCanonicalRedirect: (url, locale, input = {}) => getCanonicalRedirect(runtimeAdapter, normalizedOptions, url, locale, input),
    shouldExclude: (url, input = {}) => shouldExclude(normalizedOptions, url, input),
    setLocaleCookie: (target, locale, input = {}) => setLocaleCookie(normalizedOptions, target, runtimeAdapter.matchLocale(locale), input),
    serializeLocaleCookie: (locale, input = {}) => serializeLocaleCookie(normalizedOptions, runtimeAdapter.matchLocale(locale), input),
  };
}

export function createRequestContext(runtime, optionsOrLocale, maybeLocale) {
  const adapter = createRuntimeAdapter(runtime);
  const options = typeof optionsOrLocale === "string" ? normalizeOptions({ baseLocale: adapter.baseLocale }) : normalizeOptions({ baseLocale: adapter.baseLocale, ...(optionsOrLocale ?? {}) });
  const locale = typeof optionsOrLocale === "string" ? optionsOrLocale : maybeLocale;
  const resolvedLocale = adapter.matchLocale(locale) ?? adapter.baseLocale;
  const messages = adapter.createLinguini(resolvedLocale);
  return {
    locale: resolvedLocale,
    baseLocale: adapter.baseLocale,
    locales: adapter.locales,
    direction: adapter.getTextDirection(resolvedLocale),
    textDirection: adapter.getTextDirection(resolvedLocale),
    lang: resolvedLocale,
    messages,
    l: messages,
    htmlAttrs: htmlAttrs(adapter, resolvedLocale),
    localizeHref: (href, nextLocale = resolvedLocale, input = {}) => localizeHref(adapter, options, href, nextLocale, input),
    localizeUrl: (url, nextLocale = resolvedLocale, input = {}) => localizeUrl(adapter, options, url, nextLocale, input),
    shouldLocalizeHref: (href, input = {}) => shouldLocalizeHrefInternal(adapter, options, href, input),
    localizeHrefAttribute: (href, nextLocale = resolvedLocale, input = {}) => localizeHrefAttributeInternal(adapter, options, href, nextLocale, input),
    localizeMarkupLinks: (html, nextLocale = resolvedLocale, input = {}) => localizeMarkupLinksInternal(adapter, options, html, nextLocale, input),
    delocalizeUrl: (url, input = {}) => delocalizeUrl(adapter, options, url, input),
    alternateLinks: (url, input = {}) => alternateLinks(adapter, options, url, input),
  };
}

export function normalizeLocale(runtime, locale) {
  return createRuntimeAdapter(runtime).matchLocale(locale);
}

export function parseAcceptLanguage(header) {
  if (!header) return [];
  return String(header)
    .split(",")
    .map((part, index) => {
      const [rawTag, ...params] = part.trim().split(";");
      let q = 1;
      for (const param of params) {
        const [key, value] = param.trim().split("=");
        if (key === "q") q = Number(value);
      }
      return { tag: rawTag.trim(), q: Number.isFinite(q) ? q : 0, index };
    })
    .filter((entry) => entry.tag)
    .sort((left, right) => right.q - left.q || left.index - right.index)
    .map((entry) => entry.tag);
}

export function getCookie(source, name) {
  const cookieHeader = typeof source === "string" ? source : source?.headers?.get?.("cookie") ?? source?.cookie ?? "";
  for (const cookie of String(cookieHeader).split(";")) {
    const [rawName, ...rawValue] = cookie.trim().split("=");
    if (rawName === name) return decodeURIComponent(rawValue.join("="));
  }
  return undefined;
}

export function serializeCookie(name, value, options = {}) {
  const parts = [`${name}=${encodeURIComponent(value)}`];
  if (options.maxAge !== undefined) parts.push(`Max-Age=${options.maxAge}`);
  if (options.path) parts.push(`Path=${options.path}`);
  if (options.domain) parts.push(`Domain=${options.domain}`);
  if (options.sameSite) parts.push(`SameSite=${options.sameSite}`);
  if (options.secure) parts.push("Secure");
  if (options.httpOnly) parts.push("HttpOnly");
  return parts.join("; ");
}

export function localizeUrl(runtime, options, url, locale, input = {}) {
  const adapter = createRuntimeAdapter(runtime);
  const normalizedOptions = normalizeOptions({ baseLocale: adapter.baseLocale, ...options });
  return localizeUrlInternal(adapter, normalizedOptions, toUrl(url, input.currentUrl ?? input.url ?? input.origin ?? normalizedOptions.origin), locale, input);
}

export function localizeHref(runtime, options, href, locale, input = {}) {
  const adapter = createRuntimeAdapter(runtime);
  const normalizedOptions = normalizeOptions({ baseLocale: adapter.baseLocale, ...options });
  return localizeHrefInternal(adapter, normalizedOptions, href, locale, input);
}

export function shouldLocalizeHref(runtime, options, href, input = {}) {
  const adapter = createRuntimeAdapter(runtime);
  const normalizedOptions = normalizeOptions({ baseLocale: adapter.baseLocale, ...options });
  return shouldLocalizeHrefInternal(adapter, normalizedOptions, href, input);
}

export function localizeHrefAttribute(runtime, options, href, locale, input = {}) {
  const adapter = createRuntimeAdapter(runtime);
  const normalizedOptions = normalizeOptions({ baseLocale: adapter.baseLocale, ...options });
  return localizeHrefAttributeInternal(adapter, normalizedOptions, href, locale, input);
}

export function localizeMarkupLinks(runtime, options, html, locale, input = {}) {
  const adapter = createRuntimeAdapter(runtime);
  const normalizedOptions = normalizeOptions({ baseLocale: adapter.baseLocale, ...options });
  return localizeMarkupLinksInternal(adapter, normalizedOptions, html, locale, input);
}

export function delocalizeUrl(runtime, options, url, input = {}) {
  const adapter = createRuntimeAdapter(runtime);
  const normalizedOptions = normalizeOptions({ baseLocale: adapter.baseLocale, ...options });
  return delocalizeUrlInternal(adapter, normalizedOptions, toUrl(url, input.currentUrl ?? input.url ?? input.origin ?? normalizedOptions.origin), input);
}

export function alternateLinks(runtime, options, url, input = {}) {
  const adapter = createRuntimeAdapter(runtime);
  const normalizedOptions = normalizeOptions({ baseLocale: adapter.baseLocale, ...options });
  return alternateLinksInternal(adapter, normalizedOptions, toUrl(url, input.currentUrl ?? input.url ?? input.origin ?? normalizedOptions.origin), input);
}

async function resolveRequest(adapter, options, request, input) {
  const url = input.url ?? request?.url;
  const locale = await resolveLocale(adapter, options, { ...input, request, url });
  return createRequestContext(adapter, options, locale);
}

async function resolveLocale(adapter, options, input) {
  const strategies = routeStrategy(options, input.url) ?? options.strategy;
  for (const strategy of strategies) {
    const locale = await getStrategyLocale(adapter, options, strategy, input, true);
    const resolved = adapter.matchLocale(locale);
    if (resolved) return resolved;
  }
  return adapter.baseLocale;
}

function resolveLocaleSync(adapter, options, input) {
  const strategies = routeStrategy(options, input.url) ?? options.strategy;
  for (const strategy of strategies) {
    const locale = getStrategyLocale(adapter, options, strategy, input, false);
    const resolved = adapter.matchLocale(locale);
    if (resolved) return resolved;
  }
  return adapter.baseLocale;
}

function getStrategyLocale(adapter, options, strategy, input, allowAsync) {
  if (strategy === "baseLocale") return adapter.baseLocale;
  if (strategy === "url") return extractLocaleFromUrl(adapter, options, input.url, input);
  if (strategy === "cookie") return readCookieStrategy(options, input);
  if (strategy === "localStorage") return readLocalStorageStrategy(options, input);
  if (strategy === "header") return readHeaderStrategy(input);
  if (strategy === "navigator") return readNavigatorStrategy(input);
  if (strategy === "preferredLanguage") return readPreferredLanguageStrategy(input);
  if (strategy === "globalVariable") return readGlobalVariableStrategy(options);
  if (strategy.startsWith("custom-")) {
    const registry = input.request || input.headers ? serverStrategies : clientStrategies;
    const custom = registry.get(strategy);
    if (!custom) return undefined;
    const value = custom.getLocale(input.request, input);
    if (!allowAsync && value && typeof value.then === "function") {
      throw new TypeError(`locale strategy ${strategy} is async; use resolveLocale()`);
    }
    return value;
  }
  return undefined;
}

function readCookieStrategy(options, input) {
  if (input.cookies?.get) return input.cookies.get(options.cookieName);
  if (input.cookie) return getCookie(input.cookie, options.cookieName);
  if (input.request) return getCookie(input.request, options.cookieName);
  if (typeof document !== "undefined") return getCookie(document.cookie, options.cookieName);
  return undefined;
}

function readLocalStorageStrategy(options, input) {
  const storage = input.localStorage ?? input.storage ?? (typeof localStorage !== "undefined" ? localStorage : undefined);
  try {
    return storage?.getItem?.(options.localStorageKey) ?? undefined;
  } catch {
    return undefined;
  }
}

function readPreferredLanguageStrategy(input) {
  return readHeaderStrategy(input) ?? readNavigatorStrategy(input);
}

function readHeaderStrategy(input) {
  const header = input.headers?.get?.("accept-language") ?? input.request?.headers?.get?.("accept-language");
  return header ? parseAcceptLanguage(header) : undefined;
}

function readNavigatorStrategy(input) {
  if (input.navigator?.languages) return input.navigator.languages;
  if (input.navigator?.language) return input.navigator.language;
  return typeof navigator !== "undefined" ? navigator.languages ?? navigator.language : undefined;
}

function readGlobalVariableStrategy(options) {
  const globalName = options.globalVariableName;
  if (!globalName) return undefined;
  const root = typeof globalThis !== "undefined" ? globalThis : undefined;
  return root?.[globalName];
}

function extractLocaleFromUrl(adapter, options, url, input = {}) {
  if (!url) return undefined;
  const parsed = toUrl(url, input.origin ?? options.origin);
  const pathLocale = firstPathSegment(parsed.pathname, options.basePath);
  return adapter.matchLocale(pathLocale);
}


function localizeHrefInternal(adapter, options, href, locale, input = {}) {
  const url = localizeUrlInternal(adapter, options, toUrl(href, input.currentUrl ?? input.url ?? input.origin ?? options.origin), locale, input);
  if (/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(String(href))) return url.toString();
  return `${url.pathname}${url.search}${url.hash}`;
}

function shouldLocalizeHrefInternal(adapter, options, href, input = {}) {
  if (options.localizeLinks === false) return false;
  if (href === undefined || href === null) return false;

  const value = String(href).trim();
  if (!value || value.startsWith("#") || value.startsWith("//")) return false;

  const scheme = value.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);
  if (scheme && !["http", "https"].includes(scheme[1].toLowerCase())) return false;

  let parsed;
  try {
    parsed = toUrl(value, input.currentUrl ?? input.url ?? input.origin ?? options.origin);
  } catch {
    return false;
  }

  if (!["http:", "https:"].includes(parsed.protocol)) return false;
  if (parsed.origin !== currentOrigin(options, input)) return false;
  if (shouldExclude(options, parsed, input)) return false;

  const path = stripBasePath(parsed.pathname, options.basePath);
  if (path.startsWith("/_app/") || path === "/_app") return false;

  return true;
}

function localizeHrefAttributeInternal(adapter, options, href, locale, input = {}) {
  if (!shouldLocalizeHrefInternal(adapter, options, href, input)) return href;
  return localizeHrefInternal(adapter, options, href, locale, input);
}

function localizeMarkupLinksInternal(adapter, options, html, locale, input = {}) {
  if (options.localizeLinks === false) return String(html);
  return String(html).replace(/<a\b[^>]*>/gi, (tag) => localizeAnchorTag(adapter, options, tag, locale, input));
}

function localizeAnchorTag(adapter, options, tag, locale, input = {}) {
  if (shouldSkipAnchorTag(tag)) return tag;
  return tag.replace(/\bhref\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/i, (attribute, rawValue) => {
    const quote = rawValue[0] === '"' || rawValue[0] === "'" ? rawValue[0] : "";
    const href = quote ? rawValue.slice(1, -1) : rawValue;
    const localized = localizeHrefAttributeInternal(adapter, options, href, locale, input);
    if (localized === href) return attribute;
    return `href=${quote}${localized}${quote}`;
  });
}

function shouldSkipAnchorTag(tag) {
  if (/\sdownload(?:\s|=|>)/i.test(tag)) return true;
  if (/\sdata-linguini-(?:ignore|no-localize)(?:\s|=|>)/i.test(tag)) return true;
  const rel = tag.match(/\brel\s*=\s*("([^"]*)"|'([^']*)'|([^\s>]+))/i);
  const relValue = rel ? (rel[2] ?? rel[3] ?? rel[4] ?? "").toLowerCase() : "";
  return relValue.split(/\s+/).includes("external");
}

function localizeUrlInternal(adapter, options, url, locale, input = {}) {
  const resolved = adapter.matchLocale(locale) ?? adapter.baseLocale;
  const copy = new URL(url.toString());
  const path = stripBasePath(copy.pathname, options.basePath);
  const withoutLocale = stripLeadingLocale(adapter, path).path;
  const withLocale = shouldPrefixLocale(options, resolved)
    ? joinPath("/", resolved, withoutLocale)
    : withoutLocale;
  copy.pathname = applyTrailingSlash(joinPath(options.basePath, withLocale), options.trailingSlash, input.isDirectory);
  return copy;
}

function delocalizeUrlInternal(adapter, options, url) {
  const copy = new URL(url.toString());
  copy.pathname = joinPath(options.basePath, stripLeadingLocale(adapter, stripBasePath(copy.pathname, options.basePath)).path);
  return copy;
}

function delocalizePathname(adapter, options, pathname, input = {}) {
  const url = toUrl(pathname, input.origin);
  return delocalizeUrlInternal(adapter, options, url).pathname;
}

function alternateLinksInternal(adapter, options, url, input = {}) {
  const links = adapter.locales.map((locale) => ({
    rel: "alternate",
    hreflang: locale,
    href: localizeUrlInternal(adapter, options, url, locale, input).toString(),
  }));
  links.push({
    rel: "alternate",
    hreflang: "x-default",
    href: localizeUrlInternal(adapter, options, url, adapter.baseLocale, input).toString(),
  });
  return links;
}

function getCanonicalRedirect(adapter, options, url, locale, input = {}) {
  if (options.redirect === false || !url || shouldExclude(options, url, input)) return undefined;
  const parsed = toUrl(url, input.origin ?? options.origin);
  const canonical = localizeUrlInternal(adapter, options, parsed, locale, input);
  if (canonical.pathname === parsed.pathname) return undefined;
  return `${canonical.pathname}${canonical.search}${canonical.hash}`;
}

function setLocaleCookie(options, target, locale, input = {}) {
  if (!locale) return;
  const cookie = serializeLocaleCookie(options, locale, input);
  if (target?.setHeaders) {
    target.setHeaders({ "set-cookie": cookie });
  } else if (target?.headers?.append) {
    target.headers.append("set-cookie", cookie);
  } else if (target?.cookies?.set) {
    target.cookies.set(options.cookieName, locale, cookieOptions(options, input));
  }
}

function serializeLocaleCookie(options, locale, input = {}) {
  return serializeCookie(options.cookieName, locale, cookieOptions(options, input));
}

function cookieOptions(options, input = {}) {
  return {
    path: options.cookiePath,
    domain: options.cookieDomain,
    maxAge: input.maxAge ?? options.cookieMaxAge,
    sameSite: options.cookieSameSite,
    secure: input.secure ?? options.cookieSecure,
    httpOnly: input.httpOnly ?? options.cookieHttpOnly,
  };
}

function htmlAttrs(adapter, locale) {
  const resolved = adapter.matchLocale(locale) ?? adapter.baseLocale;
  return { lang: resolved, dir: adapter.getTextDirection(resolved) };
}

function shouldExclude(options, url, input = {}) {
  const parsed = url ? toUrl(url, input.origin ?? options.origin) : undefined;
  if (!parsed) return false;
  if (options.exclude.some((matcher) => matchesRoute(matcher, parsed))) return true;
  const route = options.routeStrategies.find((entry) => matchesRoute(entry.match, parsed));
  return Boolean(route?.exclude);
}

function routeStrategy(options, url) {
  if (!url) return undefined;
  const parsed = toUrl(url, options.origin);
  const route = options.routeStrategies.find((entry) => matchesRoute(entry.match, parsed));
  return route && !route.exclude ? route.strategy : undefined;
}

function matchesRoute(pattern, url) {
  if (!pattern) return false;
  if (typeof pattern === "function") return Boolean(pattern(url));
  if (pattern instanceof RegExp) return pattern.test(url.pathname);
  if (typeof URLPattern !== "undefined") {
    try {
      return new URLPattern({ pathname: pattern }).test(url);
    } catch {
      // fall through to the simple matcher
    }
  }
  if (pattern.endsWith("/**")) return url.pathname.startsWith(pattern.slice(0, -3));
  if (pattern.endsWith("/:path(.*)?")) return url.pathname.startsWith(pattern.slice(0, -11));
  return url.pathname === pattern;
}

function shouldPrefixLocale(options, locale) {
  return options.prefixDefaultLocale || locale !== options.baseLocale;
}

function firstPathSegment(pathname, basePath = "") {
  const stripped = stripBasePath(pathname, basePath);
  return stripped.split("/").filter(Boolean)[0];
}

function stripLeadingLocale(adapter, pathname) {
  const parts = pathname.split("/").filter(Boolean);
  const locale = adapter.matchLocale(parts[0]);
  if (!locale) return { locale: undefined, path: ensureSlash(parts.join("/")) };
  return { locale, path: ensureSlash(parts.slice(1).join("/")) };
}

function stripBasePath(pathname, basePath) {
  const normalizedPath = ensureSlash(pathname);
  const normalizedBase = basePath && basePath !== "/" ? ensureSlash(basePath).replace(/\/$/, "") : "";
  if (!normalizedBase) return normalizedPath;
  if (normalizedPath === normalizedBase) return "/";
  if (normalizedPath.startsWith(`${normalizedBase}/`)) return normalizedPath.slice(normalizedBase.length) || "/";
  return normalizedPath;
}

function joinPath(...parts) {
  const joined = parts
    .filter((part) => part !== undefined && part !== null && String(part) !== "")
    .map((part) => String(part).replace(/^\/+|\/+$/g, ""))
    .filter(Boolean)
    .join("/");
  return `/${joined}`.replace(/\/+/g, "/");
}

function ensureSlash(pathname) {
  const value = String(pathname || "/");
  return value.startsWith("/") ? value : `/${value}`;
}

function applyTrailingSlash(pathname, mode, isDirectory = false) {
  if (pathname === "/") return pathname;
  if (mode === "always" || (mode === "directory" && isDirectory)) return pathname.endsWith("/") ? pathname : `${pathname}/`;
  if (mode === "never") return pathname.replace(/\/+$/, "");
  return pathname;
}


function currentOrigin(options, input = {}) {
  const source = input.currentUrl ?? input.url ?? input.origin ?? options.origin ?? "http://localhost";
  try {
    return toUrl(source, input.origin ?? options.origin ?? "http://localhost").origin;
  } catch {
    return "http://localhost";
  }
}

function toUrl(value, origin = "http://localhost") {
  if (value instanceof URL) return value;
  return new URL(String(value), origin);
}

function normalizeOptions(options = {}) {
  return {
    strategy: options.strategy ?? DEFAULT_STRATEGY,
    cookieName: options.cookieName ?? options.cookie_name ?? DEFAULT_COOKIE_NAME,
    localStorageKey: options.localStorageKey ?? options.local_storage_key ?? DEFAULT_LOCAL_STORAGE_KEY,
    prefixDefaultLocale: Boolean(options.prefixDefaultLocale ?? options.prefix_default_locale ?? false),
    basePath: options.basePath ?? options.base_path ?? "",
    trailingSlash: options.trailingSlash ?? options.trailing_slash ?? "ignore",
    cookiePath: options.cookiePath ?? options.cookie_path ?? "/",
    cookieDomain: options.cookieDomain ?? options.cookie_domain,
    cookieMaxAge: options.cookieMaxAge ?? options.cookie_max_age ?? DEFAULT_COOKIE_MAX_AGE,
    cookieSameSite: options.cookieSameSite ?? options.cookie_same_site ?? "lax",
    cookieSecure: Boolean(options.cookieSecure ?? options.cookie_secure ?? false),
    cookieHttpOnly: Boolean(options.cookieHttpOnly ?? options.cookie_http_only ?? false),
    globalVariableName: options.globalVariableName ?? options.global_variable_name,
    routeStrategies: options.routeStrategies ?? options.route_strategies ?? [],
    exclude: options.exclude ?? [],
    redirect: options.redirect ?? true,
    origin: options.origin,
    localizeLinks: options.localizeLinks ?? options.localize_links ?? true,
    baseLocale: options.baseLocale,
  };
}

function createRuntimeAdapter(runtime) {
  if (runtime && Array.isArray(runtime.locales) && runtime.createLinguini && runtime.baseLocale) {
    const locales = [...runtime.locales];
    const baseLocale = runtime.baseLocale;
    return {
      ...runtime,
      locales,
      baseLocale,
      createLinguini: runtime.createLinguini,
      getTextDirection: runtime.getTextDirection ?? ((locale) => runtime.localeDirections?.[locale] ?? "ltr"),
      matchLocale: runtime.normalizeLocale ?? ((locale) => matchLocale(locales, locale, baseLocale)),
    };
  }
  return runtime;
}

function matchLocale(locales, value, baseLocale) {
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

function assertCustomStrategyName(name) {
  if (typeof name !== "string" || !/^custom-.+/.test(name)) {
    throw new TypeError("custom locale strategy names must start with `custom-`");
  }
}
