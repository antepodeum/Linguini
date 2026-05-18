export type TextDirection = "ltr" | "rtl";
export type BuiltInLocaleStrategy = "url" | "cookie" | "localStorage" | "header" | "navigator" | "preferredLanguage" | "globalVariable" | "baseLocale";
export type LocaleStrategy = BuiltInLocaleStrategy | `custom-${string}`;

export interface LinguiniRuntime<Locale extends string = string, Linguini = unknown> {
  locales: readonly Locale[];
  baseLocale: Locale;
  localeDirections?: Readonly<Record<Locale, TextDirection>>;
  createLinguini(locale: Locale): Linguini;
  normalizeLocale?(locale: unknown): Locale | undefined;
  getTextDirection?(locale: Locale): TextDirection;
}

export interface RouteStrategy {
  match: string | RegExp | ((url: URL) => boolean);
  strategy?: readonly LocaleStrategy[];
  exclude?: boolean;
}

export interface LinguiniWebOptions {
  strategy?: readonly LocaleStrategy[];
  cookieName?: string;
  cookie_name?: string;
  localStorageKey?: string;
  local_storage_key?: string;
  prefixDefaultLocale?: boolean;
  prefix_default_locale?: boolean;
  basePath?: string;
  base_path?: string;
  trailingSlash?: "ignore" | "always" | "never" | "directory";
  trailing_slash?: "ignore" | "always" | "never" | "directory";
  cookiePath?: string;
  cookie_path?: string;
  cookieDomain?: string;
  cookie_domain?: string;
  cookieMaxAge?: number;
  cookie_max_age?: number;
  cookieSameSite?: "lax" | "strict" | "none";
  cookie_same_site?: "lax" | "strict" | "none";
  cookieSecure?: boolean;
  cookie_secure?: boolean;
  cookieHttpOnly?: boolean;
  cookie_http_only?: boolean;
  globalVariableName?: string;
  global_variable_name?: string;
  routeStrategies?: readonly RouteStrategy[];
  route_strategies?: readonly RouteStrategy[];
  exclude?: readonly (string | RegExp | ((url: URL) => boolean))[];
  redirect?: boolean;
  origin?: string;
  localizeLinks?: boolean;
  localize_links?: boolean;
}

export interface ResolveLocaleInput {
  url?: string | URL;
  request?: Request;
  headers?: Headers;
  cookies?: { get(name: string): string | undefined | null };
  cookie?: string;
  localStorage?: Storage;
  storage?: Pick<Storage, "getItem">;
  navigator?: { language?: string; languages?: readonly string[] };
  origin?: string;
  currentUrl?: string | URL;
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

export interface AlternateLink {
  rel: "alternate";
  hreflang: string;
  href: string;
}

export interface LinguiniWeb<Locale extends string = string, Linguini = unknown> extends LinguiniRuntime<Locale, Linguini> {
  options: Required<Omit<LinguiniWebOptions, "routeStrategies" | "route_strategies" | "exclude" | "origin" | "cookieDomain" | "cookie_domain" | "globalVariableName" | "global_variable_name">> & LinguiniWebOptions;
  resolveLocale(input?: ResolveLocaleInput): Promise<Locale>;
  resolveLocaleSync(input?: ResolveLocaleInput): Locale;
  resolveRequest(request: Request, input?: ResolveLocaleInput): Promise<LinguiniRequestContext<Locale, Linguini>>;
  createRequestContext(locale: Locale, input?: Record<string, unknown>): LinguiniRequestContext<Locale, Linguini>;
  localizeUrl(url: string | URL, locale: Locale, input?: Record<string, unknown>): URL;
  localizeHref(href: string, locale: Locale, input?: Record<string, unknown>): string;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale: Locale, input?: Record<string, unknown>): string;
  localizeMarkupLinks(html: string, locale: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  delocalizePathname(pathname: string, input?: Record<string, unknown>): string;
  extractLocaleFromUrl(url: string | URL, input?: Record<string, unknown>): Locale | undefined;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
  htmlAttrs(locale: Locale): { lang: Locale; dir: TextDirection };
  getCanonicalRedirect(url: string | URL, locale: Locale, input?: Record<string, unknown>): string | undefined;
  shouldExclude(url: string | URL, input?: Record<string, unknown>): boolean;
  setLocaleCookie(target: unknown, locale: Locale, input?: Record<string, unknown>): void;
  serializeLocaleCookie(locale: Locale, input?: Record<string, unknown>): string;
}

export interface ClientLocaleStrategy<Locale extends string = string> {
  getLocale(): Locale | string | undefined;
  setLocale(locale: Locale | string): void | Promise<void>;
}

export interface ServerLocaleStrategy<Locale extends string = string> {
  getLocale(request?: Request, input?: ResolveLocaleInput): Locale | string | undefined | Promise<Locale | string | undefined>;
}

export declare function defineCustomClientStrategy<Locale extends string = string>(name: `custom-${string}`, strategy: ClientLocaleStrategy<Locale>): ClientLocaleStrategy<Locale>;
export declare function defineCustomServerStrategy<Locale extends string = string>(name: `custom-${string}`, strategy: ServerLocaleStrategy<Locale>): ServerLocaleStrategy<Locale>;
export declare function createWebI18n<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options?: LinguiniWebOptions): LinguiniWeb<Locale, Linguini>;
export declare function createRequestContext<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, locale: Locale): LinguiniRequestContext<Locale, Linguini>;
export declare function createRequestContext<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions, locale: Locale): LinguiniRequestContext<Locale, Linguini>;
export declare function normalizeLocale<Locale extends string>(runtime: LinguiniRuntime<Locale, unknown>, locale: unknown): Locale | undefined;
export declare function parseAcceptLanguage(header: string | null | undefined): string[];
export declare function getCookie(source: string | Request | { headers?: Headers; cookie?: string }, name: string): string | undefined;
export declare function serializeCookie(name: string, value: string, options?: Record<string, unknown>): string;
export declare function localizeUrl<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions, url: string | URL, locale: Locale, input?: Record<string, unknown>): URL;
export declare function localizeHref<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions, href: string, locale: Locale, input?: Record<string, unknown>): string;
export declare function shouldLocalizeHref<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions, href: string, input?: Record<string, unknown>): boolean;
export declare function localizeHrefAttribute<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions, href: string, locale: Locale, input?: Record<string, unknown>): string;
export declare function localizeMarkupLinks<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions, html: string, locale: Locale, input?: Record<string, unknown>): string;
export declare function delocalizeUrl<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions, url: string | URL, input?: Record<string, unknown>): URL;
export declare function alternateLinks<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options: LinguiniWebOptions, url: string | URL, input?: Record<string, unknown>): AlternateLink[];
