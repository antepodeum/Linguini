export type TextDirection = "ltr" | "rtl";
export type LocaleSource = "path" | "cookie" | "local-storage" | "accept-language";
export type LocalePrefixMode = "always" | "except-default" | "never";

export interface LocaleSwitchPlan {
  writesPath: boolean;
  writesCookie: boolean;
  writesLocalStorage: boolean;
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
  prepareLinguini?(locale: Locale): Promise<Linguini>;
}

export interface LinguiniWebOptions {
  routing?: {
    localePrefix?: LocalePrefixMode;
    canonical?: "redirect" | "preserve";
  };
  locale?: {
    sources?: readonly LocaleSource[];
    switch?: LocaleSwitchPlan;
  };
  cookie?: {
    name?: string;
    path?: string | "auto";
    domain?: string;
    maxAge?: number;
    sameSite?: "lax" | "strict" | "none";
    secure?: boolean | "auto";
    httpOnly?: boolean;
  };
  localStorage?: { key?: string };
  links?: { mode?: "transform" | "runtime" | "manual" };
  routes?: { exclude?: readonly (string | RegExp | ((url: URL) => boolean))[] };
}

export interface LinguiniWebEnvironment {
  base?: string;
  origin?: string | URL;
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

/**
 * Localization and delocalization leave external and non-HTTP URLs unchanged. Explicit URL
 * operations throw `TypeError("Linguini: invalid URL")` for malformed input; link-safety helpers
 * fail closed and preserve the original href instead.
 */
export interface LinguiniLocaleRequestContext<Locale extends string = string> {
  locale: Locale;
  baseLocale: Locale;
  locales: readonly Locale[];
  direction: TextDirection;
  textDirection: TextDirection;
  lang: Locale;
  htmlAttrs: { lang: Locale; dir: TextDirection };
  localizeHref(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  localizeUrl(url: string | URL, locale?: Locale, input?: Record<string, unknown>): URL;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  shouldLocalizeLink(href: string, attributes?: LinkLocalizationAttributes, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
}

export interface LinguiniRequestContext<Locale extends string = string, Linguini = unknown>
  extends LinguiniLocaleRequestContext<Locale> {
  messages: Linguini;
  l: Linguini;
}

/**
 * Localization and delocalization leave external and non-HTTP URLs unchanged. Explicit URL
 * operations throw `TypeError("Linguini: invalid URL")` for malformed input; link-safety helpers
 * fail closed and preserve the original href instead.
 */
export interface LinguiniWebLocale<Locale extends string = string> extends LinguiniLocaleRuntime<Locale> {
  options: {
    routing: { localePrefix: LocalePrefixMode; canonical: "redirect" | "preserve" };
    locale: { sources: readonly LocaleSource[]; switch: LocaleSwitchPlan };
    cookie: { name: string; path: string; domain?: string; maxAge: number; sameSite: "lax" | "strict" | "none"; secure: boolean | "auto"; httpOnly: boolean };
    localStorage: { key: string };
    links: { mode: "transform" | "runtime" | "manual" };
    routes: { exclude: readonly (string | RegExp | ((url: URL) => boolean))[] };
    environment: { base: string; origin?: string | URL };
    baseLocale: string;
  };
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

export declare function createWebLocaleI18n<Locale extends string>(runtime: LinguiniLocaleRuntime<Locale>, options?: LinguiniWebOptions, environment?: LinguiniWebEnvironment): LinguiniWebLocale<Locale>;

export declare function createWebI18n<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options?: LinguiniWebOptions, environment?: LinguiniWebEnvironment): LinguiniWeb<Locale, Linguini>;
