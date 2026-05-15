import locale_ru from "./locales/ru";

export const locales = ["ru"] as const;
export const baseLocale = "ru";

export const localeModules = {
  ru: locale_ru,
} as const;

export const localeLoaders = {
  ru: () => Promise.resolve(locale_ru),
} as const;

type LinguiniLanguage = keyof typeof localeModules;
export type Locale = (typeof locales)[number];
export type Linguini = (typeof localeModules)[LinguiniLanguage];

type LinguiniLanguageInput = LinguiniLanguage;

export type LinguiniProviderOptions = {
  getLocale?: () => LinguiniLanguageInput;
  resolveLanguage?: () => LinguiniLanguageInput;
};
let activeLocale: Locale = baseLocale;

export function createLinguini(language: LinguiniLanguageInput): Linguini {
  return localeModules[language as LinguiniLanguage];
}

export function getLocale(): Locale {
  return activeLocale ?? baseLocale;
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

export function setLocale(locale: LinguiniLanguageInput): Locale {
  const resolved = normalizeLocale(locale) ?? baseLocale;
  activeLocale = resolved;
  return activeLocale;
}

function normalizeLocale(locale: unknown): Locale | undefined {
  if (typeof locale !== "string") return undefined;
  if (locales.includes(locale as Locale)) return locale as Locale;
  const language = locale.toLowerCase().split("-")[0];
  return locales.find((candidate) => candidate.toLowerCase() === language || candidate.toLowerCase().startsWith(`${language}-`));
}

