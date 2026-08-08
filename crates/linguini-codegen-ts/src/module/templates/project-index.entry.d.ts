{{IMPORTS}}
export type * from "./shared";
import type { LinguiniMessages } from "./messages";
export type { LinguiniMessages } from "./messages";

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
export type Linguini = LinguiniMessages;

type LinguiniLanguageInput = LinguiniLanguage;

{{INDEX_RUNTIME_DECLARATIONS}}
