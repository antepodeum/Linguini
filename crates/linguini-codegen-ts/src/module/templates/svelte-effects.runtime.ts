{{BROWSER_RUNTIME}}
import * as locale from "./locale";
import { createWebLocaleI18n } from "./web";
import {
  getCurrentLocale,
  initializeCurrentLocale,
} from "./svelte-locale.svelte.js";
{{LINK_RUNTIME_IMPORT}}

export const web = createWebLocaleI18n(locale, {{OPTIONS}});

initializeCurrentLocale(readInitialLocale());

{{LINK_RUNTIME_START}}

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
