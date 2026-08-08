export const locales = [{{LOCALES}}] as const;
export const baseLocale = {{BASE_LOCALE}};

export const localeDirections = {
{{LOCALE_DIRECTIONS}}} as const;

export type Locale = (typeof locales)[number];
export type TextDirection = "ltr" | "rtl";

const localeResolutionOverrides: Readonly<Record<string, Locale | null>> = {
{{LOCALE_RESOLUTION_OVERRIDES}}};

function localeFallbackTags(locale: string): string[] {
  const tags: string[] = [];
  let tag = locale;
  while (tag) {
    tags.push(tag);
    const dash = tag.lastIndexOf("-");
    if (dash <= 0) break;
    tag = tag.slice(0, dash);
  }
  return tags;
}

function isLanguageScriptTag(locale: string): boolean {
  const parts = locale.split("-");
  return parts.length === 2
    && /^[A-Za-z]{2,8}$/.test(parts[0])
    && /^[A-Za-z]{4}$/.test(parts[1]);
}

export function isLocale(locale: unknown): locale is Locale {
  return normalizeLocale(locale) !== undefined;
}

export function normalizeLocale(locale: unknown): Locale | undefined {
  if (typeof locale !== "string") return undefined;
  for (const tag of localeFallbackTags(locale)) {
    const exact = locales.find((entry) => entry.toLowerCase() === tag.toLowerCase());
    if (exact) return exact;
    const key = tag.toLowerCase();
    if (Object.prototype.hasOwnProperty.call(localeResolutionOverrides, key)) {
      return localeResolutionOverrides[key] ?? undefined;
    }
    if (isLanguageScriptTag(tag)) return undefined;
  }
  return undefined;
}

export function getTextDirection(locale: Locale): TextDirection {
  return localeDirections[normalizeLocale(locale) ?? baseLocale];
}
