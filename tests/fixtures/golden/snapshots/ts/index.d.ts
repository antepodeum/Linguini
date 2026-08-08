import locale_ru from "./locales/ru";
import type { Locale } from "./locale";
export {
  locales,
  baseLocale,
  localeDirections,
  isLocale,
  normalizeLocale,
  getTextDirection,
} from "./locale";
export type { Locale, TextDirection } from "./locale";
export type * from "./shared";
import type { LinguiniMessages } from "./messages";
export type { LinguiniMessages } from "./messages";

export declare const localeModules: {
  readonly ru: typeof locale_ru;
};

export declare const localeLoaders: {
  readonly ru: () => Promise<typeof locale_ru>;
};

type LinguiniLanguage = keyof typeof localeModules;
export type Linguini = LinguiniMessages;

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
