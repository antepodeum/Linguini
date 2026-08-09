import { page } from "$app/state";
import { baseLocale, normalizeLocale, type Locale } from "./locale";

export type LinguiniLocaleLoader = (locale: Locale) => void | Promise<void>;

const localeLoaders = new Set<LinguiniLocaleLoader>();

let clientLocale = $state<Locale>(baseLocale);
let hasClientOverride = $state(false);

/**
 * Register a locale message loader and return its deterministic disposer.
 *
 * The disposer is idempotent, which lets HMR consumers safely replace a loader
 * without retaining callbacks from a previous module instance.
 */
export function registerLocaleLoader(loader: LinguiniLocaleLoader): () => void {
  localeLoaders.add(loader);
  let disposed = false;
  return () => {
    if (disposed) return;
    disposed = true;
    localeLoaders.delete(loader);
  };
}

/**
 * Ensure all currently registered consumers have loaded a locale's messages.
 *
 * This intentionally snapshots the registry before starting work: a loader
 * may register or dispose another loader while its promise is pending, but
 * that mutation must not alter the current preparation operation.
 */
export async function prepareLocale(locale: unknown): Promise<Locale> {
  const resolved = normalizeLocale(locale) ?? baseLocale;
  const loaders = [...localeLoaders];
  await Promise.all(loaders.map((loader) => loader(resolved)));
  return resolved;
}

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
