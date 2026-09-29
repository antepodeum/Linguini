import { browser } from "$app/environment";
import { base } from "$app/paths";
import * as locale from "./locale.js";
import { createWebLocaleI18n } from "./web.js";
import { getCurrentLocale, initializeCurrentLocale } from "./svelte-locale.svelte.js";

/** @type {import("./web.js").LinguiniWebLocale<import("./locale.js").Locale>} */
export const web = createWebLocaleI18n(locale, { routing: { localePrefix: "except-default", canonical: "redirect" }, locale: { sources: [], switch: { writesPath: true, writesCookie: true, writesLocalStorage: false } }, links: { mode: "manual" }, routes: { exclude: [] } }, { base });

initializeCurrentLocale(readInitialLocale());

/** @type {{ refresh(): void; destroy(): void } | undefined} */
const linkEffects = undefined;

/** @returns {void} */
export function refreshLinguiniEffects() {
  linkEffects?.refresh();
}

/** @returns {void} */
export function destroyLinguiniEffects() {
  linkEffects?.destroy();
}

const hot = (/** @type {ImportMeta & { hot?: { dispose(callback: () => void): void } }} */ (import.meta)).hot;
hot?.dispose(destroyLinguiniEffects);

function readInitialLocale() {
  if (!browser) return web.baseLocale;
  return web.resolveLocaleSync({

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
