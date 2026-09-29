import { destroyLinguiniEffects, refreshLinguiniEffects, web } from "./svelte-effects.svelte.js";
import { getCurrentLocale, prepareLocale, setCurrentLocale } from "./svelte-locale.svelte.js";

/** @typedef {import("./locale.js").Locale} Locale */
/** @typedef {import("./web.js").LinkLocalizationAttributes} LinkLocalizationAttributes */

const browser = typeof window !== "undefined" && typeof document !== "undefined";

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
  /** @param {string} nextLocale @param {Record<string, unknown>} [setOptions] @returns {Promise<Locale>} */
  async function setLocale(nextLocale, setOptions = {}) {
    const resolved = await prepareLocale(nextLocale);
    /** @type {Record<string, unknown> & {cookie: boolean; navigate: boolean; replaceState: boolean; invalidateAll: boolean; keepFocus: boolean; noScroll: boolean}} */
    const options = {
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
        const href = web.localizeHref(window.location.href, resolved);
        if (options.replaceState) {
          window.location.replace(href);
        } else {
          window.location.assign(href);
        }
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
    /** @param {string} href @param {Locale} [locale] @param {Record<string, unknown>} [input] */
    localizeHref: (href, locale = getCurrentLocale(), input) => web.localizeHref(href, locale, input),
    /** @param {string | URL} url @param {Locale} [locale] @param {Record<string, unknown>} [input] */
    localizeUrl: (url, locale = getCurrentLocale(), input) => web.localizeUrl(url, locale, input),
    /** @param {string} href @param {Record<string, unknown>} [input] */
    shouldLocalizeHref: (href, input) => web.shouldLocalizeHref(href, input),
    /** @param {string} href @param {LinkLocalizationAttributes} [attributes] @param {Record<string, unknown>} [input] */
    shouldLocalizeLink: (href, attributes = {}, input) => web.shouldLocalizeLink(href, attributes, input),
    /** @param {string} href @param {Locale} [locale] @param {Record<string, unknown>} [input] */
    localizeHrefAttribute: (href, locale = getCurrentLocale(), input) => web.localizeHrefAttribute(href, locale, input),
    /** @param {string | URL} url @param {Record<string, unknown>} [input] */
    delocalizeUrl: (url, input) => web.delocalizeUrl(url, input),
    /** @param {string | URL} url @param {Record<string, unknown>} [input] */
    alternateLinks: (url, input) => web.alternateLinks(url, input),
    destroy: destroyLinguiniEffects,
  };
}

/** @param {typeof import("./svelte-effects.svelte.js").web} web @param {string} locale */
function writeLocalStorage(web, locale) {
  try {
    window.localStorage.setItem(web.options.localStorage.key, locale);
  } catch {
    // Ignore storage failures in private browsing and locked-down contexts.
  }
}

/** @param {typeof import("./svelte-effects.svelte.js").web} web @param {Locale} locale */
function writeLocaleCookie(web, locale) {
  try {
    document.cookie = web.serializeLocaleCookie(locale, { httpOnly: false });
  } catch {
    // Ignore cookie failures in sandboxed and locked-down contexts.
  }
}
//# sourceMappingURL=svelte-control.js.map
