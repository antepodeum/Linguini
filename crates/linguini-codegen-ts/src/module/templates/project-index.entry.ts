{{IMPORTS}}
import { baseLocale, normalizeLocale, type Locale } from "./locale";
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

export const localeModules: Partial<Record<Locale, LinguiniMessages>> = {
{{LOCALE_MODULES}}} as const;

export const localeLoaders = {
{{LOCALE_LOADERS}}} as const;

type LinguiniLanguage = Locale;
export type Linguini = LinguiniMessages;

type LinguiniLanguageInput = LinguiniLanguage;

{{INDEX_RUNTIME}}
