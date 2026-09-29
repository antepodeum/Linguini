import * as locale from "./locale";
import { createWebLocaleI18n } from "./web";
import { getCurrentLocale, initializeCurrentLocale } from "./svelte-locale.svelte.js";
import { startRuntimeLinkLocalization } from "./web/runtime-links.js";

const browser = typeof window !== "undefined" && typeof document !== "undefined";

export const web = createWebLocaleI18n(locale, { routing: { localePrefix: "except-default", canonical: "redirect" }, locale: { sources: ["path", "cookie", "local-storage", "accept-language"] as const, switch: { writesPath: true, writesCookie: true, writesLocalStorage: false } as const }, links: { mode: "runtime" }, routes: { exclude: [] as const }, cookie: { name: "LINGUINI_LOCALE", path: "auto", maxAge: 31536000, sameSite: "lax", secure: "auto", httpOnly: false }, localStorage: { key: "LINGUINI_LOCALE" } } as const, {});

initializeCurrentLocale(readInitialLocale());

const linkEffects = browser
  ? startRuntimeLinkLocalization(web, getCurrentLocale)
  : undefined;

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
    url: readBrowserCapability(() => window.location.href),
    cookie: readBrowserCapability(() => document.cookie),
    localStorage: readBrowserCapability(() => window.localStorage),
    navigator: readBrowserCapability(() => window.navigator),
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
