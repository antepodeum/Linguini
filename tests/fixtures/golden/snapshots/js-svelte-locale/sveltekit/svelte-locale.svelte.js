import { page } from "$app/state";
import { baseLocale, normalizeLocale } from "./locale.js";

/** @typedef {import("./locale.js").Locale} Locale */
/** @typedef {(locale: Locale) => void | Promise<void>} LinguiniLocaleLoader */

/** @type {Set<LinguiniLocaleLoader>} */ const localeLoaders = new Set();

/** @type {Locale} */ let clientLocale = $state(baseLocale);
let hasClientOverride = $state(false);

/**
 * Register a locale message loader and return its deterministic disposer.
 *
 * The disposer is idempotent, which lets HMR consumers safely replace a loader
 * without retaining callbacks from a previous module instance.
 */
/** @param {LinguiniLocaleLoader} loader @returns {() => void} */
export function registerLocaleLoader(loader) {
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
/** @param {unknown} locale @returns {Promise<Locale>} */
export async function prepareLocale(locale) {
  const resolved = normalizeLocale(locale) ?? baseLocale;
  const loaders = [...localeLoaders];
  await Promise.all(loaders.map((loader) => loader(resolved)));
  return resolved;
}

/** @returns {Locale} */
export function getCurrentLocale() {
  const dataLocale = page.data?.linguini?.locale;
  const resolvedClientLocale = normalizeLocale(clientLocale) ?? baseLocale;
  return hasClientOverride
    ? resolvedClientLocale
    : normalizeLocale(dataLocale) ?? resolvedClientLocale;
}

/** @param {unknown} locale @returns {Locale} */
export function initializeCurrentLocale(locale) {
  clientLocale = normalizeLocale(locale) ?? baseLocale;
  hasClientOverride = false;
  return clientLocale;
}

/** @param {unknown} locale @returns {Locale} */
export function setCurrentLocale(locale) {
  clientLocale = normalizeLocale(locale) ?? baseLocale;
  hasClientOverride = true;
  return clientLocale;
}

/** @returns {void} */
export function clearCurrentLocaleOverride() {
  hasClientOverride = false;
}
//# sourceMappingURL=svelte-locale.svelte.js.map
