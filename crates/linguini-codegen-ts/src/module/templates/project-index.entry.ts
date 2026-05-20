{{IMPORTS}}

export const locales = [{{LOCALES}}] as const;
export const baseLocale = {{BASE_LOCALE}};

export const localeDirections = {
{{LOCALE_DIRECTIONS}}} as const;

export const localeModules = {
{{LOCALE_MODULES}}} as const;

export const localeLoaders = {
{{LOCALE_LOADERS}}} as const;

type LinguiniLanguage = keyof typeof localeModules;
export type Locale = (typeof locales)[number];
export type TextDirection = "ltr" | "rtl";
export type Linguini = (typeof localeModules)[LinguiniLanguage];

type LinguiniLanguageInput = LinguiniLanguage;

{{INDEX_RUNTIME}}
