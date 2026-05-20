import locale_en from "./locales/en";
import locale_ru from "./locales/ru";

export const locales = ["en", "ru"] as const;
export const baseLocale = "en";

export const localeDirections = {
  en: "ltr",
  ru: "ltr",
} as const;

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
export type TextDirection = "ltr" | "rtl";
export type Linguini = (typeof localeModules)[LinguiniLanguage];

type LinguiniLanguageInput = LinguiniLanguage;

export type LinguiniProviderOptions = {
  getLocale?: () => LinguiniLanguageInput;
  resolveLanguage?: () => LinguiniLanguageInput;
};

function localeFallbackTags(locale: string): string[] {
const tags: string[] = [];
let tag = locale;
while (tag) {
tags.push(tag);
const dash = tag.lastIndexOf("-");
if (dash <= 0) break;
tag = tag.slice(0, dash);
}
return tags;
}

function localeFallbackChain(locale: Locale): Locale[] {
const chain: Locale[] = [];
for (const tag of localeFallbackTags(locale)) {
const exact = locales.find((entry) => entry.toLowerCase() === tag.toLowerCase());
if (exact && !chain.includes(exact)) chain.push(exact);
}
if (!chain.includes(baseLocale)) chain.push(baseLocale);
return chain;
}

function mergeLocaleChain(chain: Locale[]): Linguini {
let merged = {} as Linguini;
for (const locale of [...chain].reverse()) {
merged = mergeLocaleModule(merged, localeModules[locale as LinguiniLanguage]);
}
return merged;
}

function mergeLocaleModule(target: Linguini, source: Linguini): Linguini {
const result = { ...target } as Linguini;
for (const key of Object.keys(source) as (keyof Linguini)[]) {
const value = source[key];
const existing = target[key];
if (
value &&
typeof value === "object" &&
!Array.isArray(value) &&
typeof (value as { call?: unknown }).call !== "function" &&
existing &&
typeof existing === "object" &&
!Array.isArray(existing) &&
typeof (existing as { call?: unknown }).call !== "function"
) {
result[key] = mergeLocaleModule(existing as Linguini, value as Linguini) as Linguini[keyof Linguini];
} else {
result[key] = value;
}
}
return result;
}

export function createLinguini(language: LinguiniLanguageInput): Linguini {
  const locale = normalizeLocale(language) ?? baseLocale;
  return mergeLocaleChain(localeFallbackChain(locale));
}

export function createLinguiniProvider(options: LinguiniProviderOptions = {}): Linguini {
  const resolve = options.getLocale ?? options.resolveLanguage ?? (() => baseLocale);
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

export function isLocale(locale: unknown): locale is Locale {
  return normalizeLocale(locale) !== undefined;
}

export function normalizeLocale(locale: unknown): Locale | undefined {
  if (typeof locale !== "string") return undefined;
  for (const tag of localeFallbackTags(locale)) {
    const exact = locales.find((entry) => entry.toLowerCase() === tag.toLowerCase());
    if (exact) return exact;
  }
  return undefined;
}

export function getTextDirection(locale: Locale): TextDirection {
  return localeDirections[normalizeLocale(locale) ?? baseLocale];
}
