import * as locale from "./locale.js";
import { createWebLocaleI18n } from "./web.js";
import { initializeCurrentLocale } from "./svelte-locale.svelte.js";

const browser = typeof window !== "undefined" && typeof document !== "undefined";

/** @type {import("./web.js").LinguiniWebLocale<import("./locale.js").Locale>} */
export const web = createWebLocaleI18n(locale, { routing: { localePrefix: "except-default", canonical: "redirect" }, locale: { sources: ["path"], switch: { writesPath: true, writesCookie: true, writesLocalStorage: false } }, links: { mode: "transform" }, routes: { exclude: [] } }, {});

initializeCurrentLocale(readInitialLocale());

/** @returns {void} */
export function refreshLinguiniEffects() {
}

/** @returns {void} */
export function destroyLinguiniEffects() {
}


function readInitialLocale() {
  if (!browser) return web.baseLocale;
  return web.resolveLocaleSync({
    url: readBrowserCapability(() => window.location.href),
  });
}

/** @template T @param {() => T} read @returns {T | undefined} */
function readBrowserCapability(read) {
  try {
    return read();
  } catch {
    return undefined;
  }
}
//# sourceMappingURL=svelte-effects.svelte.js.map
