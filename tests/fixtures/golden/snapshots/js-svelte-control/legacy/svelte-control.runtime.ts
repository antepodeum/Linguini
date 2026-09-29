{{NAVIGATION_RUNTIME}}
import {
  destroyLinguiniEffects,
  refreshLinguiniEffects,
  web,
} from "./svelte-effects.svelte.js";
{{LOCALE_RUNTIME}}

export const linguini = createLinguiniControl();
export const setLocale = linguini.setLocale;
export const localizeHref = linguini.localizeHref;
export const localizeUrl = linguini.localizeUrl;
export const shouldLocalizeHref = linguini.shouldLocalizeHref;
export const shouldLocalizeLink = linguini.shouldLocalizeLink;
export const localizeHrefAttribute = linguini.localizeHrefAttribute;
export const delocalizeUrl = linguini.delocalizeUrl;
export const alternateLinks = linguini.alternateLinks;
export const destroy = linguini.destroy;

function createLinguiniControl() {
  async function setLocale(nextLocale: string, setOptions: Record<string, unknown> = {}) {
    const resolved = await prepareLocale(nextLocale);
    const options: Record<string, unknown> & {
      cookie: boolean;
      navigate: boolean;
      replaceState: boolean;
      invalidateAll: boolean;
      keepFocus: boolean;
      noScroll: boolean;
    } = {
      cookie: true,
      navigate: true,
      replaceState: false,
      invalidateAll: true,
      keepFocus: true,
      noScroll: true,
      ...setOptions,
    };
    if (browser) {
      setCurrentLocale(resolved);
      if (web.options.locale.switch.writesLocalStorage) {
        writeLocalStorage(web, resolved);
      }
      if (options.cookie && web.options.locale.switch.writesCookie) {
        writeLocaleCookie(web, resolved);
      }
      if (options.navigate && web.options.locale.switch.writesPath) {
{{NAVIGATION}}
      }
      refreshLinguiniEffects();
    }

    return resolved;
  }

  return {
    get locale() {
      return getCurrentLocale();
    },
    get lang() {
      return getCurrentLocale();
    },
    get direction() {
      return web.getTextDirection(getCurrentLocale());
    },
    get textDirection() {
      return web.getTextDirection(getCurrentLocale());
    },
    get htmlAttrs() {
      return web.htmlAttrs(getCurrentLocale());
    },
    setLocale,
    localizeHref: (href: string, locale = getCurrentLocale(), input?: Record<string, unknown>) => web.localizeHref(href, locale, input),
    localizeUrl: (url: string | URL, locale = getCurrentLocale(), input?: Record<string, unknown>) => web.localizeUrl(url, locale, input),
    shouldLocalizeHref: (href: string, input?: Record<string, unknown>) => web.shouldLocalizeHref(href, input),
    shouldLocalizeLink: (href: string, attributes = {}, input?: Record<string, unknown>) => web.shouldLocalizeLink(href, attributes, input),
    localizeHrefAttribute: (href: string, locale = getCurrentLocale(), input?: Record<string, unknown>) => web.localizeHrefAttribute(href, locale, input),
    delocalizeUrl: (url: string | URL, input?: Record<string, unknown>) => web.delocalizeUrl(url, input),
    alternateLinks: (url: string | URL, input?: Record<string, unknown>) => web.alternateLinks(url, input),
    destroy: destroyLinguiniEffects,
  };
}

function writeLocalStorage(web: typeof import("./svelte-effects.svelte.js").web, locale: string) {
  try {
    window.localStorage.setItem(web.options.localStorage.key, locale);
  } catch {
    // Ignore storage failures in private browsing and locked-down contexts.
  }
}

function writeLocaleCookie(
  web: typeof import("./svelte-effects.svelte.js").web,
  locale: Parameters<typeof web.serializeLocaleCookie>[0],
) {
  try {
    document.cookie = web.serializeLocaleCookie(locale, { httpOnly: false });
  } catch {
    // Ignore cookie failures in sandboxed and locked-down contexts.
  }
}
