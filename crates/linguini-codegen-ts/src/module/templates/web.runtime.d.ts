export type TextDirection = "ltr" | "rtl";
export type LocaleStrategy = "url" | "cookie" | "localStorage" | "header" | "navigator" | "preferredLanguage" | "globalVariable" | "baseLocale" | `custom-${string}`;

export interface LinguiniRuntime<Locale extends string = string, Linguini = unknown> {
  locales: readonly Locale[];
  baseLocale: Locale;
  localeDirections?: Readonly<Record<Locale, TextDirection>>;
  createLinguini(locale: Locale): Linguini;
  normalizeLocale?(locale: unknown): Locale | undefined;
  getTextDirection?(locale: Locale): TextDirection;
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

export interface AlternateLink {
  rel: "alternate";
  hreflang: string;
  href: string;
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

export declare function createWebI18n<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options?: LinguiniWebOptions): LinguiniWeb<Locale, Linguini>;
