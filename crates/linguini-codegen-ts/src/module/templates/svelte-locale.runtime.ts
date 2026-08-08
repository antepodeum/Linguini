import { page } from "$app/state";
import { baseLocale, normalizeLocale, type Locale } from "./locale";

let clientLocale = $state<Locale>(baseLocale);
let hasClientOverride = $state(false);

export function getCurrentLocale(): Locale {
  const dataLocale = page.data?.linguini?.locale;
  const resolvedClientLocale = normalizeLocale(clientLocale) ?? baseLocale;
  return hasClientOverride
    ? resolvedClientLocale
    : normalizeLocale(dataLocale) ?? resolvedClientLocale;
}

export function initializeCurrentLocale(locale: unknown): Locale {
  clientLocale = normalizeLocale(locale) ?? baseLocale;
  hasClientOverride = false;
  return clientLocale;
}

export function setCurrentLocale(locale: unknown): Locale {
  clientLocale = normalizeLocale(locale) ?? baseLocale;
  hasClientOverride = true;
  return clientLocale;
}

export function clearCurrentLocaleOverride(): void {
  hasClientOverride = false;
}
