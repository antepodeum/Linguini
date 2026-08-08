import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { createWebI18n } from "./web";
import * as runtime from "./index";
import {
  clearCurrentLocaleOverride,
  getCurrentLocale,
  initializeCurrentLocale,
  setCurrentLocale,
} from "./svelte-locale.svelte.js";

const AUTO_LINK_MAX_PENDING_ROOTS = 128;
const AUTO_LINK_NODE_BUDGET = 256;

export const linguini = createLinguiniRune(runtime, {{OPTIONS}});
export const l = linguini.l;
export const messages = linguini.messages;
export const setLocale = linguini.setLocale;
export const localizeHref = linguini.localizeHref;
export const localizeUrl = linguini.localizeUrl;
export const shouldLocalizeHref = linguini.shouldLocalizeHref;
export const shouldLocalizeLink = linguini.shouldLocalizeLink;
export const localizeHrefAttribute = linguini.localizeHrefAttribute;
export const delocalizeUrl = linguini.delocalizeUrl;
export const alternateLinks = linguini.alternateLinks;

const hot = (import.meta as ImportMeta & {
  hot?: { dispose(callback: () => void): void };
}).hot;
hot?.dispose(() => linguini.destroy());

function createLinguiniRune(runtime: typeof import("./index"), options = {}) {
  const web = createWebI18n(runtime, options);
  initializeCurrentLocale(readInitialLocale(web));
  const messages = runtime.createLinguiniProvider({
    getLocale: getCurrentLocale,
  });
  const autoLinks = browser && web.options.localizeLinks !== false
    ? startAutoLinkLocalization(web, getCurrentLocale)
    : undefined;

  async function setLocale(nextLocale: string, setOptions: Record<string, unknown> = {}) {
    const resolved = web.matchLocale(nextLocale) ?? web.baseLocale;
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
        clearCurrentLocaleOverride();
      }
      autoLinks?.refresh();
    }

    return resolved;
  }

  return {
    messages,
    l: messages,
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
    destroy: () => autoLinks?.destroy(),
  };
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
  const pendingRoots: Node[] = [];
  const queuedRoots = new Set<Node>();
  let activeTraversal: LinkTraversal | undefined;
  let animationFrame: number | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let destroyed = false;

  const schedule = () => {
    if (destroyed || animationFrame !== undefined || timer !== undefined) return;
    if (typeof requestAnimationFrame === "function") {
      animationFrame = requestAnimationFrame(() => {
        animationFrame = undefined;
        flush();
      });
    } else {
      timer = setTimeout(() => {
        timer = undefined;
        flush();
      }, 0);
    }
  };

  const enqueueRoot = (root: Node) => {
    if (destroyed || !isLinkTraversalRoot(root) || queuedRoots.has(root)) return;
    if (pendingRoots.length >= AUTO_LINK_MAX_PENDING_ROOTS) {
      enqueueDocument();
      return;
    }
    pendingRoots.push(root);
    queuedRoots.add(root);
    schedule();
  };

  const enqueueDocument = () => {
    pendingRoots.length = 0;
    queuedRoots.clear();
    activeTraversal = undefined;
    const root = document.documentElement;
    if (root) {
      pendingRoots.push(root);
      queuedRoots.add(root);
      schedule();
    }
  };

  const flush = () => {
    if (destroyed) return;
    let remaining = AUTO_LINK_NODE_BUDGET;
    while (remaining > 0) {
      if (!activeTraversal) {
        const root = pendingRoots.shift();
        if (!root) break;
        queuedRoots.delete(root);
        activeTraversal = createLinkTraversal(root);
        if (!activeTraversal) continue;
      }

      const element = nextTraversalElement(activeTraversal);
      if (!element) {
        activeTraversal = undefined;
        continue;
      }
      if (element.matches("a[href]")) {
        localizeAnchorElement(web, getLocale(), element);
      }
      remaining -= 1;
    }
    if (activeTraversal || pendingRoots.length > 0) schedule();
  };

  const observer = typeof MutationObserver === "undefined"
    ? undefined
    : new MutationObserver((mutations) => {
        let queued = 0;
        for (const mutation of mutations) {
          if (mutation.type === "attributes") {
            enqueueRoot(mutation.target);
            queued += 1;
            if (queued >= AUTO_LINK_MAX_PENDING_ROOTS) {
              enqueueDocument();
              return;
            }
            continue;
          }
          for (const node of mutation.addedNodes) {
            enqueueRoot(node);
            queued += 1;
            if (queued >= AUTO_LINK_MAX_PENDING_ROOTS) {
              enqueueDocument();
              return;
            }
          }
        }
      });
  if (document.documentElement) {
    observer?.observe(document.documentElement, {
      subtree: true,
      childList: true,
      attributes: true,
      attributeFilter: [
        "href",
        "download",
        "rel",
        "data-linguini-ignore",
        "data-linguini-no-localize",
      ],
    });
  }
  const onClick = (event: Event) => {
    const anchor = (event.target as Element | null)?.closest?.("a[href]");
    if (anchor) localizeAnchorElement(web, getLocale(), anchor);
  };
  document.addEventListener("click", onClick, true);
  const onReady = () => enqueueDocument();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", onReady, { once: true });
  } else {
    enqueueDocument();
  }

  return {
    refresh: enqueueDocument,
    destroy() {
      destroyed = true;
      observer?.disconnect();
      document.removeEventListener("click", onClick, true);
      document.removeEventListener("DOMContentLoaded", onReady);
      if (animationFrame !== undefined) cancelAnimationFrame(animationFrame);
      if (timer !== undefined) clearTimeout(timer);
      pendingRoots.length = 0;
      queuedRoots.clear();
      activeTraversal = undefined;
    },
  };
}

interface LinkTraversal {
  root: Element | undefined;
  rootPending: boolean;
  walker: TreeWalker;
}

function isLinkTraversalRoot(node: Node) {
  return node.nodeType === 1 || node.nodeType === 9 || node.nodeType === 11;
}

function createLinkTraversal(root: Node): LinkTraversal | undefined {
  const ownerDocument = root.nodeType === 9
    ? root as Document
    : root.ownerDocument;
  if (!ownerDocument) return undefined;
  return {
    root: root.nodeType === 1 ? root as Element : undefined,
    rootPending: root.nodeType === 1,
    walker: ownerDocument.createTreeWalker(root, 0x1),
  };
}

function nextTraversalElement(traversal: LinkTraversal): Element | undefined {
  if (traversal.rootPending) {
    traversal.rootPending = false;
    return traversal.root;
  }
  return (traversal.walker.nextNode() as Element | null) ?? undefined;
}

function localizeAnchorElement(web: ReturnType<typeof createWebI18n>, locale: string, anchor: Element) {
  const href = anchor.getAttribute("href");
  if (!href) return;
  const input = {
    currentUrl: window.location.href,
    origin: window.location.origin,
  };
  const attributes = {
    download: anchor.hasAttribute("download"),
    ignored: anchor.hasAttribute("data-linguini-ignore")
      || anchor.hasAttribute("data-linguini-no-localize"),
    rel: anchor.getAttribute("rel"),
  };
  if (!web.shouldLocalizeLink(href, attributes, input)) return;
  const localized = web.localizeHref(href, locale, input);
  if (localized !== href) anchor.setAttribute("href", localized);
}
