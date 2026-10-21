import { base } from "../host.js";
import * as locale from "./locale.js";
import { createWebLocaleI18n } from "./web.js";
import { initializeCurrentLocale } from "./svelte-locale.svelte.js";

/** @type {import("./web.js").LinguiniWebLocale<import("./locale.js").Locale>} */
export const web = createWebLocaleI18n(locale, { routing: { localePrefix: "except-default", canonical: "redirect" }, locale: { sources: [], switch: { writesPath: true, writesCookie: true, writesLocalStorage: false } }, links: { mode: "manual" }, routes: { exclude: [] } }, { base });

initializeCurrentLocale(web.baseLocale);

/** @returns {void} */
export function refreshLinguiniEffects() {
}

/** @returns {void} */
export function destroyLinguiniEffects() {
}
//# sourceMappingURL=svelte-effects.svelte.js.map
