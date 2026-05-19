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

export function createLinguini(language: LinguiniLanguageInput): Linguini {
  const locale = normalizeLocale(language) ?? baseLocale;
  return localeModules[locale as LinguiniLanguage];
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
  if (locales.includes(locale as Locale)) return locale as Locale;
  const language = locale.toLowerCase().split("-")[0];
  return locales.find((candidate) => candidate.toLowerCase() === language || candidate.toLowerCase().startsWith(`${language}-`));
}

export function getTextDirection(locale: Locale): TextDirection {
  return localeDirections[normalizeLocale(locale) ?? baseLocale];
}
