{{IMPORTS}}
export type * from "./shared";
import type { LinguiniMessages } from "./messages";
export type { LinguiniMessages } from "./messages";

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
export type Linguini = LinguiniMessages;

type LinguiniLanguageInput = LinguiniLanguage;

const localeResolutionOverrides: Readonly<Record<string, Locale | null>> = {
{{LOCALE_RESOLUTION_OVERRIDES}}};

{{INDEX_RUNTIME}}
