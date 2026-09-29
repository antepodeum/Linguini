import { browser } from "$app/environment";
import { base } from "$app/paths";
import * as locale from "./locale";
import { createWebLocaleI18n } from "./web";
import { getCurrentLocale, initializeCurrentLocale } from "./svelte-locale.svelte.js";

export const web = createWebLocaleI18n(locale, { routing: { localePrefix: "except-default", canonical: "redirect" }, locale: { sources: ["local-storage"] as const, switch: { writesPath: true, writesCookie: true, writesLocalStorage: false } as const }, links: { mode: "manual" }, routes: { exclude: [] as const }, localStorage: { key: "LINGUINI_LOCALE" } } as const, { base });

initializeCurrentLocale(readInitialLocale());

const linkEffects: { refresh(): void; destroy(): void } | undefined = undefined;

export function refreshLinguiniEffects(): void {
  linkEffects?.refresh();
}

export function destroyLinguiniEffects(): void {
  linkEffects?.destroy();
}

const hot = (import.meta as ImportMeta & {
  hot?: { dispose(callback: () => void): void };
}).hot;
hot?.dispose(destroyLinguiniEffects);

function readInitialLocale() {
  if (!browser) return web.baseLocale;
  return web.resolveLocaleSync({
    localStorage: readBrowserCapability(() => window.localStorage),
  });
}

function readBrowserCapability<T>(read: () => T): T | undefined {
  try {
    return read();
  } catch {
    return undefined;
  }
}
//# sourceMappingURL=svelte-effects.svelte.ts.map
