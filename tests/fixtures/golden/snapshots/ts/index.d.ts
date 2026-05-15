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

export type LinguiniProviderOptions = {
  getLocale?: () => LinguiniLanguageInput;
  resolveLanguage?: () => LinguiniLanguageInput;
};

export declare function createLinguini(language: LinguiniLanguageInput): Linguini;

export declare function getLocale(): Locale;

export declare function createLinguiniProvider(options?: LinguiniProviderOptions): Linguini;

export declare function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini;

export declare const lgl: Linguini;

export declare function setLocale(locale: LinguiniLanguageInput): Locale;
