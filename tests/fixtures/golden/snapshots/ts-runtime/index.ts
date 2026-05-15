import locale_en from "./locales/en";
import locale_ru from "./locales/ru";

export const locales = ["en", "ru"] as const;
export const baseLocale = "en";

export const localeModules = {
  en: locale_en,
  ru: locale_ru,
} as const;

export const localeLoaders = {
  en: () => Promise.resolve(locale_en),
  ru: () => Promise.resolve(locale_ru),
} as const;

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

type AsyncLocalStorageLike<T> = {
  getStore(): T | undefined;
  run<R>(store: T, callback: () => R): R;
};

let activeLocale: Locale = baseLocale;
let activeLocaleStore: AsyncLocalStorageLike<Locale> | undefined;

export function createLinguini(language: LinguiniLanguageInput): Linguini {
  return localeModules[language as LinguiniLanguage];
}

export function getLocale(): Locale {
  return activeLocaleStore?.getStore() ?? activeLocale ?? baseLocale;
}

export function createLinguiniProvider(options: LinguiniProviderOptions = {}): Linguini {
  const resolve = options.getLocale ?? options.resolveLanguage ?? getLocale;
  return new Proxy({} as Linguini, {
    get(_target, property) {
      return createLinguini(resolve())[property as keyof Linguini];
    },
  });
}

export function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini {
  if (typeof options.language === "function") {
    return createLinguiniProvider({ resolveLanguage: options.language });
  }
  return createLinguini(options.language);
}

export const lgl: Linguini = createLinguini(baseLocale);

export function resolveLocale(context: LocaleDetectionContext = {}, options: LocaleResolveOptions = {}): Locale {
  for (const detector of options.strategy ?? defaultLocaleStrategy) {
    const locale = normalizeLocale(detector(context));
    if (locale) return locale;
  }
  return baseLocale;
}

export const defaultLocaleStrategy = [
  detectUrlLocale,
  detectCookieLocale,
  detectPreferredLanguage,
  detectLocalStorageLocale,
  detectBaseLocale,
] as const;

export function detectUrlLocale(context: LocaleDetectionContext): string | undefined {
  const pathname = toUrl(context.url)?.pathname ?? (typeof context.url === "string" ? context.url : "");
  return pathname.split("/").find(Boolean);
}

export function detectCookieLocale(context: LocaleDetectionContext): string | undefined {
  const name = context.cookieName ?? "linguini_locale";
  if (typeof context.cookies === "string") return readCookieString(context.cookies, name);
  if (context.cookies) return context.cookies[name];
  const cookie = readHeader(context.headers, "cookie");
  return cookie ? readCookieString(cookie, name) : undefined;
}

export function detectPreferredLanguage(context: LocaleDetectionContext): string | undefined {
  const header = readHeader(context.headers, "accept-language");
  const candidates = header ? parseAcceptLanguage(header) : context.navigator?.languages ?? [context.navigator?.language ?? ""];
  return candidates.map(normalizeLocale).find(Boolean);
}

export function detectLocalStorageLocale(context: LocaleDetectionContext): string | undefined {
  return context.localStorage?.getItem(context.localStorageKey ?? "linguini_locale") ?? undefined;
}

export function detectBaseLocale(): LinguiniLanguageInput {
  return baseLocale;
}

export function localizeHref(href: string | URL, locale: LinguiniLanguageInput, options: LocaleHrefOptions = {}): string {
  const resolved = normalizeLocale(locale) ?? baseLocale;
  const url = toUrl(href);
  const pathname = url?.pathname ?? String(href);
  const suffix = url ? `${url.search}${url.hash}` : "";
  const parts = pathname.split("/").filter(Boolean);
  if (parts.length > 0 && normalizeLocale(parts[0])) parts.shift();
  if (!(options.stripBaseLocale && resolved === baseLocale)) parts.unshift(resolved);
  const nextPath = `/${parts.join("/")}`;
  return url && url.origin !== "http://linguini.local" && url.origin !== "null" ? `${url.origin}${nextPath}${suffix}` : `${nextPath}${suffix}`;
}

export function shouldRedirect(href: string | URL, locale: LinguiniLanguageInput, options: LocaleHrefOptions = {}): boolean {
  const current = toComparableHref(href);
  return current !== localizeHref(href, locale, options);
}

export function runWithLocale<R>(locale: LinguiniLanguageInput, callback: () => R, options: LinguiniMiddlewareOptions = {}): R {
  const resolved = normalizeLocale(locale) ?? baseLocale;
  const store = options.disableAsyncLocalStorage ? undefined : getAsyncLocalStorage();
  if (store) return store.run(resolved, callback);
  const previous = activeLocale;
  activeLocale = resolved;
  try {
    const result = callback();
    if (isPromiseLike(result)) return result.finally(() => { activeLocale = previous; }) as R;
    return result;
  } finally {
    activeLocale = previous;
  }
}

export function createLinguiniMiddleware(options: LinguiniMiddlewareOptions = {}) {
  return function linguiniMiddleware<R>(context: LinguiniMiddlewareContext, next: () => R): R {
    return runWithLocale(context.locale ?? resolveLocale(context, options), next, options);
  };
}

export function injectLangAndDir(template: string, locale: LinguiniLanguageInput = getLocale()): string {
  const resolved = normalizeLocale(locale) ?? baseLocale;
  return template.split("%lang%").join(resolved).split("%dir%").join(getTextDirection(resolved));
}

export function getTextDirection(locale: LinguiniLanguageInput = getLocale()): TextDirection {
  const language = String(locale).toLowerCase().split("-")[0];
  return ["ar", "fa", "he", "ps", "ur", "yi"].includes(language) ? "rtl" : "ltr";
}

function normalizeLocale(locale: unknown): Locale | undefined {
  if (typeof locale !== "string") return undefined;
  if (locales.includes(locale as Locale)) return locale as Locale;
  const language = locale.toLowerCase().split("-")[0];
  return locales.find((candidate) => candidate.toLowerCase() === language || candidate.toLowerCase().startsWith(`${language}-`));
}

function readHeader(headers: HeaderReader | undefined, name: string): string | undefined {
  if (!headers) return undefined;
  const get = (headers as { get?: unknown }).get;
  if (typeof get === "function") return (get.call(headers, name) as string | null) ?? undefined;
  const record = headers as Record<string, string | string[] | undefined>;
  const value = record[name] ?? record[name.toLowerCase()];
  return Array.isArray(value) ? value.join(",") : value;
}

function readCookieString(source: string, name: string): string | undefined {
  return source.split(";").map((part) => part.trim()).find((part) => part.startsWith(`${name}=`))?.slice(name.length + 1);
}

function parseAcceptLanguage(header: string): string[] {
  return header.split(",").map((part) => part.trim().split(";")[0]).filter(Boolean);
}

function toUrl(value: string | URL | undefined): URL | undefined {
  if (!value) return undefined;
  if (value instanceof URL) return value;
  try { return new URL(value, "http://linguini.local"); } catch { return undefined; }
}

function toComparableHref(value: string | URL): string {
  const url = toUrl(value);
  if (!url) return String(value);
  const suffix = `${url.search}${url.hash}`;
  return url.origin === "http://linguini.local" ? `${url.pathname}${suffix}` : `${url.origin}${url.pathname}${suffix}`;
}

function isPromiseLike(value: unknown): value is Promise<unknown> {
  return typeof value === "object" && value !== null && "finally" in value;
}

function getAsyncLocalStorage(): AsyncLocalStorageLike<Locale> | undefined {
  if (activeLocaleStore) return activeLocaleStore;
  try {
    const req = Function("return typeof require === 'function' && require")() as ((name: string) => { AsyncLocalStorage?: new () => AsyncLocalStorageLike<Locale> }) | false;
    const Storage = req && req("node:async_hooks").AsyncLocalStorage;
    activeLocaleStore = Storage ? new Storage() : undefined;
  } catch {
    activeLocaleStore = undefined;
  }
  return activeLocaleStore;
}
