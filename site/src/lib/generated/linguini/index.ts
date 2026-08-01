import locale_en from "./locales/en";
import locale_ru from "./locales/ru";
import locale_es from "./locales/es";
import locale_fr from "./locales/fr";
import locale_de from "./locales/de";
import locale_it from "./locales/it";
import locale_zh from "./locales/zh";
import locale_pt from "./locales/pt";
import locale___lgl_name_70742D4252 from "./locales/pt-BR";
export type * from "./shared";

export const locales = ["en", "ru", "es", "fr", "de", "it", "zh", "pt", "pt-BR"] as const;
export const baseLocale = "en";

export const localeDirections = {
  en: "ltr",
  ru: "ltr",
  es: "ltr",
  fr: "ltr",
  de: "ltr",
  it: "ltr",
  zh: "ltr",
  pt: "ltr",
  "pt-BR": "ltr",
} as const;

export const localeModules = {
  en: locale_en,
  ru: locale_ru,
  es: locale_es,
  fr: locale_fr,
  de: locale_de,
  it: locale_it,
  zh: locale_zh,
  pt: locale_pt,
  "pt-BR": locale___lgl_name_70742D4252,
} as const;

export const localeLoaders = {
  en: () => Promise.resolve(locale_en),
  ru: () => Promise.resolve(locale_ru),
  es: () => Promise.resolve(locale_es),
  fr: () => Promise.resolve(locale_fr),
  de: () => Promise.resolve(locale_de),
  it: () => Promise.resolve(locale_it),
  zh: () => Promise.resolve(locale_zh),
  pt: () => Promise.resolve(locale_pt),
  "pt-BR": () => Promise.resolve(locale___lgl_name_70742D4252),
} as const;

type LinguiniLanguage = keyof typeof localeModules;
export type Locale = (typeof locales)[number];
export type TextDirection = "ltr" | "rtl";
export type Linguini = (typeof localeModules)[LinguiniLanguage];

type LinguiniLanguageInput = LinguiniLanguage;

const localeResolutionOverrides: Readonly<Record<string, Locale | null>> = {
  "chi": "zh",
  "cmn": "zh",
  "de-latn": "de",
  "deu": "de",
  "en-latn": "en",
  "eng": "en",
  "es-latn": "es",
  "fr-latn": "fr",
  "fra": "fr",
  "fre": "fr",
  "ger": "de",
  "hat": "fr",
  "hi-latn": "en",
  "ht": "fr",
  "i-default": "en",
  "it-latn": "it",
  "ita": "it",
  "por": "pt",
  "pt-latn": "pt",
  "ru-cyrl": "ru",
  "rus": "ru",
  "spa": "es",
  "zh-au": null,
  "zh-bn": null,
  "zh-cmn-hant": null,
  "zh-gan": null,
  "zh-gb": null,
  "zh-gf": null,
  "zh-hakka": null,
  "zh-hans": "zh",
  "zh-hk": null,
  "zh-id": null,
  "zh-min": null,
  "zh-min-nan": null,
  "zh-mo": null,
  "zh-pa": null,
  "zh-pf": null,
  "zh-ph": null,
  "zh-sr": null,
  "zh-th": null,
  "zh-tw": null,
  "zh-us": null,
  "zh-vn": null,
  "zh-wuu": null,
  "zh-xiang": null,
  "zh-yue": null,
  "zho": "zh",
};

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

function isLanguageScriptTag(locale: string): boolean {
  const parts = locale.split("-");
  return parts.length === 2
    && /^[A-Za-z]{2,8}$/.test(parts[0])
    && /^[A-Za-z]{4}$/.test(parts[1]);
}

export function createLinguini(language: LinguiniLanguageInput): Linguini {
  const locale = normalizeLocale(language) ?? baseLocale;
  return localeModules[locale];
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
    const key = tag.toLowerCase();
    if (Object.prototype.hasOwnProperty.call(localeResolutionOverrides, key)) {
      return localeResolutionOverrides[key] ?? undefined;
    }
    if (isLanguageScriptTag(tag)) return undefined;
  }
  return undefined;
}

export function getTextDirection(locale: Locale): TextDirection {
  return localeDirections[normalizeLocale(locale) ?? baseLocale];
}
