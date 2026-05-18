import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { page } from "$app/state";
import { createWebI18n } from "@antepod/linguini-web";

let activeAutoLinkCleanup;

export function createLinguiniRune(runtime, options = {}) {
  const web = createWebI18n(runtime, options.web ?? options);
  let clientLocale = $state(readInitialLocale(web));

  const messages = runtime.createLinguiniProvider({
    getLocale: () => getCurrentLocale(web, clientLocale),
  });

  if (browser && web.options.localizeLinks !== false) {
    activeAutoLinkCleanup?.();
    activeAutoLinkCleanup = startAutoLinkLocalization(web, () => getCurrentLocale(web, clientLocale));
  }

  async function setLocale(nextLocale, setOptions = {}) {
    const resolved = web.matchLocale(nextLocale) ?? web.baseLocale;
    clientLocale = resolved;

    if (browser) {
      writeLocalStorage(web, resolved);
      if (setOptions.cookie !== false) {
        document.cookie = web.serializeLocaleCookie(resolved, { httpOnly: false });
      }
      if (setOptions.navigate !== false) {
        const href = web.localizeHref(window.location.href, resolved);
        await goto(href, {
          replaceState: setOptions.replaceState ?? false,
          invalidateAll: setOptions.invalidateAll ?? true,
          keepFocus: setOptions.keepFocus,
          noScroll: setOptions.noScroll,
          state: setOptions.state,
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
    localizeHref: (href, locale = getCurrentLocale(web, clientLocale), input) => web.localizeHref(href, locale, input),
    localizeUrl: (url, locale = getCurrentLocale(web, clientLocale), input) => web.localizeUrl(url, locale, input),
    shouldLocalizeHref: (href, input) => web.shouldLocalizeHref(href, input),
    localizeHrefAttribute: (href, locale = getCurrentLocale(web, clientLocale), input) => web.localizeHrefAttribute(href, locale, input),
    localizeMarkupLinks: (html, locale = getCurrentLocale(web, clientLocale), input) => web.localizeMarkupLinks(html, locale, input),
    delocalizeUrl: (url, input) => web.delocalizeUrl(url, input),
    alternateLinks: (url, input) => web.alternateLinks(url, input),
  };
}

function getCurrentLocale(web, clientLocale) {
  const dataLocale = page.data?.linguini?.locale;
  return web.matchLocale(dataLocale) ?? web.matchLocale(clientLocale) ?? web.baseLocale;
}

function readInitialLocale(web) {
  if (!browser) return web.baseLocale;
  return web.resolveLocaleSync({
    url: window.location.href,
    cookie: document.cookie,
    localStorage,
    navigator,
  });
}

function writeLocalStorage(web, locale) {
  try {
    localStorage.setItem(web.options.localStorageKey, locale);
  } catch {
    // Ignore storage access failures in private browsing and locked-down contexts.
  }
}


function startAutoLinkLocalization(web, getLocale) {
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
          localizeAnchorElement(web, getLocale(), mutation.target);
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

  const onClick = (event) => {
    const anchor = event.target?.closest?.("a[href]");
    if (anchor) localizeAnchorElement(web, getLocale(), anchor);
  };
  document.addEventListener("click", onClick, true);

  return () => {
    observer?.disconnect();
    document.removeEventListener("click", onClick, true);
  };
}

function localizeDocumentLinks(web, locale) {
  if (typeof document === "undefined") return;
  localizeNodeLinks(web, locale, document);
}

function localizeNodeLinks(web, locale, node) {
  if (!node || node.nodeType !== 1 && node.nodeType !== 9 && node.nodeType !== 11) return;
  if (node.matches?.("a[href]")) localizeAnchorElement(web, locale, node);
  for (const anchor of node.querySelectorAll?.("a[href]") ?? []) {
    localizeAnchorElement(web, locale, anchor);
  }
}

function localizeAnchorElement(web, locale, anchor) {
  if (shouldSkipAnchor(anchor)) return;
  const href = anchor.getAttribute("href");
  if (!href) return;
  const localized = web.localizeHrefAttribute(href, locale, {
    currentUrl: window.location.href,
    origin: window.location.origin,
  });
  if (localized !== href) anchor.setAttribute("href", localized);
}

function shouldSkipAnchor(anchor) {
  if (anchor.hasAttribute("download")) return true;
  if (anchor.hasAttribute("data-linguini-ignore")) return true;
  if (anchor.hasAttribute("data-linguini-no-localize")) return true;
  return (anchor.getAttribute("rel") ?? "").toLowerCase().split(/\s+/).includes("external");
}
