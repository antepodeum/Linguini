import locale_en from "./locales/en";
import locale_ru from "./locales/ru";
import locale_es from "./locales/es";
import locale_fr from "./locales/fr";
import locale_de from "./locales/de";
import locale_it from "./locales/it";
import locale_zh from "./locales/zh";

export declare const locales: readonly ["en", "ru", "es", "fr", "de", "it", "zh"];
export declare const baseLocale: "en";

export declare const localeDirections: {
  readonly en: "ltr";
  readonly ru: "ltr";
  readonly es: "ltr";
  readonly fr: "ltr";
  readonly de: "ltr";
  readonly it: "ltr";
  readonly zh: "ltr";
};

export declare const localeModules: {
  readonly en: typeof locale_en;
  readonly ru: typeof locale_ru;
  readonly es: typeof locale_es;
  readonly fr: typeof locale_fr;
  readonly de: typeof locale_de;
  readonly it: typeof locale_it;
  readonly zh: typeof locale_zh;
};

export declare const localeLoaders: {
  readonly en: () => Promise<typeof locale_en>;
  readonly ru: () => Promise<typeof locale_ru>;
  readonly es: () => Promise<typeof locale_es>;
  readonly fr: () => Promise<typeof locale_fr>;
  readonly de: () => Promise<typeof locale_de>;
  readonly it: () => Promise<typeof locale_it>;
  readonly zh: () => Promise<typeof locale_zh>;
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
