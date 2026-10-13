export type Locale = "en" | "fr";
export type TextDirection = "ltr" | "rtl";
export declare const locales: readonly Locale[];
export declare const baseLocale: Locale;
export declare function normalizeLocale(locale: unknown): Locale | undefined;
export declare function getTextDirection(locale: Locale): TextDirection;
