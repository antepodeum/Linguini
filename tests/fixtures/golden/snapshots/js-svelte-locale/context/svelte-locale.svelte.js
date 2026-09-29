import { baseLocale, normalizeLocale } from "./locale.js";

/** @typedef {import("./locale.js").Locale} Locale */
/** @typedef {(locale: Locale) => void | Promise<void>} LinguiniLocaleLoader */

/** @type {Set<LinguiniLocaleLoader>} */ const localeLoaders = new Set();

/** @type {Locale} */ let activeLocale = $state(baseLocale);

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

/** @param {unknown} locale @returns {Promise<Locale>} */
export async function prepareLocale(locale) {
  const resolved = normalizeLocale(locale) ?? baseLocale;
  const loaders = [...localeLoaders];
  await Promise.all(loaders.map((loader) => loader(resolved)));
  return resolved;
}

/** @returns {Locale} */
export function getCurrentLocale() {
  return activeLocale;
}

/** @param {unknown} locale @returns {Locale} */
export function setCurrentLocale(locale) {
  activeLocale = normalizeLocale(locale) ?? baseLocale;
  return activeLocale;
}
//# sourceMappingURL=svelte-locale.svelte.js.map
