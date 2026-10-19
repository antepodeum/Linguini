export declare const locales: readonly ["ru"];
export declare const baseLocale: "ru";

export declare const localeDirections: {
  readonly ru: "ltr";
};

export type Locale = (typeof locales)[number];
export type TextDirection = "ltr" | "rtl";

export declare function isLocale(locale: unknown): locale is Locale;
export declare function normalizeLocale(locale: unknown): Locale | undefined;
export declare function getTextDirection(locale: Locale): TextDirection;
