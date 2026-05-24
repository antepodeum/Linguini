import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { page } from "$app/state";
import { createWebI18n } from "./web";
import * as runtime from "./index";

let activeAutoLinkCleanup: (() => void) | undefined;

export const linguini = createLinguiniRune(runtime, {{OPTIONS}});
export const l = linguini.l;
export const messages = linguini.messages;
export const setLocale = linguini.setLocale;
export const localizeHref = linguini.localizeHref;
export const localizeUrl = linguini.localizeUrl;
export const delocalizeUrl = linguini.delocalizeUrl;
export const alternateLinks = linguini.alternateLinks;

function createLinguiniRune(runtime: typeof import("./index"), options = {}) {
  const web = createWebI18n(runtime, options);
  let clientLocale = readInitialLocale(web);
  const messages = runtime.createLinguiniProvider({
    getLocale: () => getCurrentLocale(web, clientLocale),
  });

  if (browser && web.options.localizeLinks !== false) {
    activeAutoLinkCleanup?.();
    activeAutoLinkCleanup = startAutoLinkLocalization(web, () => getCurrentLocale(web, clientLocale));
  }

  async function setLocale(nextLocale: string, setOptions: Record<string, unknown> = {}) {
    const resolved = web.matchLocale(nextLocale) ?? web.baseLocale;
    const options = {
      cookie: true,
      navigate: true,
      replaceState: false,
      invalidateAll: true,
      keepFocus: true,
      noScroll: true,
      ...setOptions,
    };
    clientLocale = resolved;

    if (browser) {
      writeLocalStorage(web, resolved);
      if (options.cookie) {
        document.cookie = web.serializeLocaleCookie(resolved, { httpOnly: false });
      }
      if (options.navigate) {
        const href = web.localizeHref(window.location.href, resolved);
        await goto(href, {
          replaceState: Boolean(options.replaceState),
          invalidateAll: Boolean(options.invalidateAll),
          keepFocus: options.keepFocus as boolean | undefined,
          noScroll: options.noScroll as boolean | undefined,
          state: options.state as App.PageState | undefined,
        });
      }
      localizeDocumentLinks(web, resolved);
    }

    return resolved;
  }

  return {
    messages,
    l: messages,
    get locale() {
      return getCurrentLocale(web, clientLocale);
    },
    get lang() {
      return getCurrentLocale(web, clientLocale);
    },
    get direction() {
      return web.getTextDirection(getCurrentLocale(web, clientLocale));
    },
    get textDirection() {
      return web.getTextDirection(getCurrentLocale(web, clientLocale));
    },
    get htmlAttrs() {
      return web.htmlAttrs(getCurrentLocale(web, clientLocale));
    },
    setLocale,
    localizeHref: (href: string, locale = getCurrentLocale(web, clientLocale), input?: Record<string, unknown>) => web.localizeHref(href, locale, input),
    localizeUrl: (url: string | URL, locale = getCurrentLocale(web, clientLocale), input?: Record<string, unknown>) => web.localizeUrl(url, locale, input),
    shouldLocalizeHref: (href: string, input?: Record<string, unknown>) => web.shouldLocalizeHref(href, input),
    localizeHrefAttribute: (href: string, locale = getCurrentLocale(web, clientLocale), input?: Record<string, unknown>) => web.localizeHrefAttribute(href, locale, input),
    localizeMarkupLinks: (html: string, locale = getCurrentLocale(web, clientLocale), input?: Record<string, unknown>) => web.localizeMarkupLinks(html, locale, input),
    delocalizeUrl: (url: string | URL, input?: Record<string, unknown>) => web.delocalizeUrl(url, input),
    alternateLinks: (url: string | URL, input?: Record<string, unknown>) => web.alternateLinks(url, input),
  };
}

function getCurrentLocale(web: ReturnType<typeof createWebI18n>, clientLocale: string): any {
  const dataLocale = page.data?.linguini?.locale;
  return web.matchLocale(dataLocale) ?? web.matchLocale(clientLocale) ?? web.baseLocale;
}

function readInitialLocale(web: ReturnType<typeof createWebI18n>): any {
  if (!browser) return web.baseLocale;
  return web.resolveLocaleSync({
    url: window.location.href,
    cookie: document.cookie,
    localStorage,
    navigator,
  });
}

function writeLocalStorage(web: ReturnType<typeof createWebI18n>, locale: string) {
  try {
    localStorage.setItem(web.options.localStorageKey, locale);
  } catch {
    // Ignore storage failures in private browsing and locked-down contexts.
  }
}

function startAutoLinkLocalization(web: ReturnType<typeof createWebI18n>, getLocale: () => string) {
  if (typeof document === "undefined") return undefined;
  const localize = () => localizeDocumentLinks(web, getLocale());
  queueMicrotask(localize);
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", localize, { once: true });
  }
  const observer = typeof MutationObserver === "undefined"
    ? undefined
    : new MutationObserver((mutations) => {
        for (const mutation of mutations) {
          if (mutation.type === "attributes") {
            localizeAnchorElement(web, getLocale(), mutation.target as Element);
            continue;
          }
          for (const node of mutation.addedNodes) {
            localizeNodeLinks(web, getLocale(), node);
          }
        }
      });
  observer?.observe(document.documentElement, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ["href"],
  });
  const onClick = (event: Event) => {
    const anchor = (event.target as Element | null)?.closest?.("a[href]");
    if (anchor) localizeAnchorElement(web, getLocale(), anchor);
  };
  document.addEventListener("click", onClick, true);
  return () => {
    observer?.disconnect();
    document.removeEventListener("click", onClick, true);
  };
}

function localizeDocumentLinks(web: ReturnType<typeof createWebI18n>, locale: string) {
  if (typeof document === "undefined") return;
  localizeNodeLinks(web, locale, document);
}

function localizeNodeLinks(web: ReturnType<typeof createWebI18n>, locale: string, node: Node) {
  if (node.nodeType !== 1 && node.nodeType !== 9 && node.nodeType !== 11) return;
  const element = node as Element;
  if (element.matches?.("a[href]")) localizeAnchorElement(web, locale, element);
  for (const anchor of element.querySelectorAll?.("a[href]") ?? []) {
    localizeAnchorElement(web, locale, anchor);
  }
}

function localizeAnchorElement(web: ReturnType<typeof createWebI18n>, locale: string, anchor: Element) {
  if (shouldSkipAnchor(anchor)) return;
  const href = anchor.getAttribute("href");
  if (!href) return;
  const localized = web.localizeHrefAttribute(href, locale, {
    currentUrl: window.location.href,
    origin: window.location.origin,
  });
  if (localized !== href) anchor.setAttribute("href", localized);
}

function shouldSkipAnchor(anchor: Element) {
  if (anchor.hasAttribute("download")) return true;
  if (anchor.hasAttribute("data-linguini-ignore")) return true;
  if (anchor.hasAttribute("data-linguini-no-localize")) return true;
  return (anchor.getAttribute("rel") ?? "").toLowerCase().split(/\s+/).includes("external");
}
