import locale_ru from "./locales/ru";

export declare const locales: readonly ["ru"];
export declare const baseLocale: "ru";

export declare const localeModules: {
  readonly ru: typeof locale_ru;
};

export declare const localeLoaders: {
  readonly ru: () => Promise<typeof locale_ru>;
};

type LinguiniLanguage = keyof typeof localeModules;
export type Locale = (typeof locales)[number];
export type Linguini = (typeof localeModules)[LinguiniLanguage];

type LinguiniLanguageInput = LinguiniLanguage;

export type LocaleDetector = (context: LocaleDetectionContext) => unknown;
export type HeaderReader = { get(name: string): string | null } | Record<string, string | string[] | undefined>;
export type CookieReader = string | Record<string, string | undefined>;
export type LocaleDetectionContext = {
  url?: string | URL;
  headers?: HeaderReader;
  cookies?: CookieReader;
  cookieName?: string;
  navigator?: { language?: string; languages?: readonly string[] };
  localStorage?: { getItem(key: string): string | null };
  localStorageKey?: string;
};
export type LocaleResolveOptions = { strategy?: readonly LocaleDetector[] };
export type LinguiniProviderOptions = {
  getLocale?: () => LinguiniLanguageInput;
  resolveLanguage?: () => LinguiniLanguageInput;
};
export type LocaleHrefOptions = { stripBaseLocale?: boolean };
export type LinguiniMiddlewareOptions = {
  strategy?: readonly LocaleDetector[];
  disableAsyncLocalStorage?: boolean;
};
export type LinguiniMiddlewareContext = LocaleDetectionContext & { locale?: LinguiniLanguageInput };
export type TextDirection = "ltr" | "rtl";

export declare function createLinguini(language: LinguiniLanguageInput): Linguini;

export declare function getLocale(): Locale;

export declare function createLinguiniProvider(options?: LinguiniProviderOptions): Linguini;

export declare function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini;

export declare const lgl: Linguini;

export declare function resolveLocale(context?: LocaleDetectionContext, options?: LocaleResolveOptions): Locale;
export declare const defaultLocaleStrategy: readonly [typeof detectUrlLocale, typeof detectCookieLocale, typeof detectPreferredLanguage, typeof detectLocalStorageLocale, typeof detectBaseLocale];
export declare function detectUrlLocale(context: LocaleDetectionContext): string | undefined;
export declare function detectCookieLocale(context: LocaleDetectionContext): string | undefined;
export declare function detectPreferredLanguage(context: LocaleDetectionContext): string | undefined;
export declare function detectLocalStorageLocale(context: LocaleDetectionContext): string | undefined;
export declare function detectBaseLocale(): LinguiniLanguageInput;
export declare function localizeHref(href: string | URL, locale: LinguiniLanguageInput, options?: LocaleHrefOptions): string;
export declare function shouldRedirect(href: string | URL, locale: LinguiniLanguageInput, options?: LocaleHrefOptions): boolean;
export declare function runWithLocale<R>(locale: LinguiniLanguageInput, callback: () => R, options?: LinguiniMiddlewareOptions): R;
export declare function createLinguiniMiddleware(options?: LinguiniMiddlewareOptions): <R>(context: LinguiniMiddlewareContext, next: () => R) => R;
export declare function injectLangAndDir(template: string, locale?: LinguiniLanguageInput): string;
export declare function getTextDirection(locale?: LinguiniLanguageInput): TextDirection;
