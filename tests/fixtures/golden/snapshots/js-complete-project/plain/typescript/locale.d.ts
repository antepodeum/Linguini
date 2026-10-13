export declare const locales: readonly ["en", "fr"];
export declare const baseLocale: "en";

export declare const localeDirections: {
  readonly en: "ltr";
  readonly fr: "ltr";
};

export type Locale = (typeof locales)[number];
export type TextDirection = "ltr" | "rtl";

export declare function isLocale(locale: unknown): locale is Locale;
export declare function normalizeLocale(locale: unknown): Locale | undefined;
export declare function getTextDirection(locale: Locale): TextDirection;
