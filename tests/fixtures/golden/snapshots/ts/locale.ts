export const locales = ["ru"] as const;
export const baseLocale = "ru";

export const localeDirections = {
  ru: "ltr",
} as const;

export type Locale = (typeof locales)[number];
export type TextDirection = "ltr" | "rtl";

const localeResolution: Readonly<Record<string, Locale>> = {
  "ru": "ru",
  "ru-cyrl": "ru",
  "rus": "ru",
};

export function isLocale(locale: unknown): locale is Locale {
  return normalizeLocale(locale) !== undefined;
}

export function normalizeLocale(locale: unknown): Locale | undefined {
  if (typeof locale !== "string") return undefined;
  return localeResolution[locale.toLowerCase()];
}

export function getTextDirection(locale: Locale): TextDirection {
  return localeDirections[normalizeLocale(locale) ?? baseLocale];
}
