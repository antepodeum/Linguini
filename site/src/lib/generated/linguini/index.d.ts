import locale_en from "./locales/en";
import locale_ru from "./locales/ru";

export declare const locales: readonly ["en", "ru"];
export declare const baseLocale: "en";

export declare const localeDirections: {
  readonly en: "ltr";
  readonly ru: "ltr";
};

export declare const localeModules: {
  readonly en: typeof locale_en;
  readonly ru: typeof locale_ru;
};

export declare const localeLoaders: {
  readonly en: () => Promise<typeof locale_en>;
  readonly ru: () => Promise<typeof locale_ru>;
};

type LinguiniLanguage = keyof typeof localeModules;
export type Locale = (typeof locales)[number];
export type TextDirection = "ltr" | "rtl";
export type Linguini = (typeof localeModules)[LinguiniLanguage];

type LinguiniLanguageInput = LinguiniLanguage;

export type LinguiniProviderOptions = {
  getLocale?: () => LinguiniLanguageInput;
  resolveLanguage?: () => LinguiniLanguageInput;
};

export declare function createLinguini(language: LinguiniLanguageInput): Linguini;

export declare function createLinguiniProvider(options?: LinguiniProviderOptions): Linguini;

export declare function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini;

export declare const lgl: Linguini;

export declare function isLocale(locale: unknown): locale is Locale;
export declare function normalizeLocale(locale: unknown): Locale | undefined;
export declare function getTextDirection(locale: Locale): TextDirection;
