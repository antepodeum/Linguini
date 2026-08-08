{{IMPORTS}}
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
{{LOCALE_MODULES}}};

export declare const localeLoaders: {
{{LOCALE_LOADERS}}};

type LinguiniLanguage = keyof typeof localeModules;
export type Linguini = LinguiniMessages;

type LinguiniLanguageInput = LinguiniLanguage;

{{INDEX_RUNTIME_DECLARATIONS}}
