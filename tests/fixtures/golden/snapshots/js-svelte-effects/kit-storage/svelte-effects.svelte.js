import { browser } from "../host.js";
import { base } from "../host.js";
import * as locale from "./locale.js";
import { createWebLocaleI18n } from "./web.js";
import { initializeCurrentLocale } from "./svelte-locale.svelte.js";

/** @type {import("./web.js").LinguiniWebLocale<import("./locale.js").Locale>} */
export const web = createWebLocaleI18n(locale, { routing: { localePrefix: "except-default", canonical: "redirect" }, locale: { sources: ["local-storage"], switch: { writesPath: true, writesCookie: true, writesLocalStorage: false } }, links: { mode: "manual" }, routes: { exclude: [] }, localStorage: { key: "LINGUINI_LOCALE" } }, { base });

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
    localStorage: readBrowserCapability(() => window.localStorage),
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
