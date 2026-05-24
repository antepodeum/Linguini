{{IMPORTS}}
export type * from "./shared";

export declare const locales: readonly [{{LOCALES}}];
export declare const baseLocale: {{BASE_LOCALE}};

export declare const localeDirections: {
{{LOCALE_DIRECTIONS}}};

export declare const localeModules: {
{{LOCALE_MODULES}}};

export declare const localeLoaders: {
{{LOCALE_LOADERS}}};

type LinguiniLanguage = keyof typeof localeModules;
export type Locale = (typeof locales)[number];
export type TextDirection = "ltr" | "rtl";
export type Linguini = (typeof localeModules)[LinguiniLanguage];

type LinguiniLanguageInput = LinguiniLanguage;

{{INDEX_RUNTIME_DECLARATIONS}}
