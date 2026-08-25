export const locales = [{{LOCALES}}] as const;
export const baseLocale = {{BASE_LOCALE}};

export const localeDirections = {
{{LOCALE_DIRECTIONS}}} as const;

export type Locale = (typeof locales)[number];
export type TextDirection = "ltr" | "rtl";

const localeResolution: Readonly<Record<string, Locale>> = {
{{LOCALE_RESOLUTION}}};

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
