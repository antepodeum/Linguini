import locale_ru from "./locales/ru";

export const locales = ["ru"] as const;
export const baseLocale = "ru";

export const localeDirections = {
  ru: "ltr",
} as const;

export const localeModules = {
  ru: locale_ru,
} as const;

export const localeLoaders = {
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

type LinguiniRecord = Record<string, unknown>;

function isPlainLocaleObject(value: unknown): value is LinguiniRecord {
return !!value && typeof value === "object" && !Array.isArray(value) && typeof (value as { call?: unknown }).call !== "function";
}

function mergeLocaleModule(target: Linguini, source: Linguini): Linguini {
const targetRecord = target as unknown as LinguiniRecord;
const sourceRecord = source as unknown as LinguiniRecord;
const result: LinguiniRecord = { ...targetRecord };
for (const key of Object.keys(sourceRecord)) {
const value = sourceRecord[key];
const existing = targetRecord[key];
if (isPlainLocaleObject(value) && isPlainLocaleObject(existing)) {
result[key] = mergeLocaleModule(existing as unknown as Linguini, value as unknown as Linguini);
} else {
result[key] = value;
}
}
return result as unknown as Linguini;
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
