import { baseLocale, normalizeLocale, type Locale } from "./locale";

let activeLocale = $state<Locale>(baseLocale);

export function getCurrentLocale(): Locale {
  return activeLocale;
}

export function setCurrentLocale(locale: unknown): Locale {
  activeLocale = normalizeLocale(locale) ?? baseLocale;
  return activeLocale;
}
