import { browser } from "$app/environment";
import { base } from "$app/paths";
import * as locale from "./locale.js";
import { createWebLocaleI18n } from "./web.js";
import { getCurrentLocale, initializeCurrentLocale } from "./svelte-locale.svelte.js";
import { startRuntimeLinkLocalization } from "./web/runtime-links.js";

/** @type {import("./web.js").LinguiniWebLocale<import("./locale.js").Locale>} */
export const web = createWebLocaleI18n(locale, { routing: { localePrefix: "except-default", canonical: "redirect" }, locale: { sources: ["path", "cookie", "local-storage", "accept-language"], switch: { writesPath: true, writesCookie: true, writesLocalStorage: false } }, links: { mode: "runtime" }, routes: { exclude: [] }, cookie: { name: "LINGUINI_LOCALE", path: "auto", maxAge: 31536000, sameSite: "lax", secure: "auto", httpOnly: false }, localStorage: { key: "LINGUINI_LOCALE" } }, { base });

initializeCurrentLocale(readInitialLocale());

const linkEffects = browser
  ? startRuntimeLinkLocalization(web, getCurrentLocale)
  : undefined;

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
    url: readBrowserCapability(() => window.location.href),
    cookie: readBrowserCapability(() => document.cookie),
    localStorage: readBrowserCapability(() => window.localStorage),
    navigator: readBrowserCapability(() => window.navigator),
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
