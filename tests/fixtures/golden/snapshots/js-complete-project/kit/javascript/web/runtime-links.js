const MAX_PENDING_ROOTS = 128;
const NODE_BUDGET = 256;

/**
 * @template {string} LocaleName
 * @param {import("../web.js").LinguiniWebLocale<LocaleName>} web
 * @param {() => LocaleName} getLocale
 * @returns {{refresh(): void, destroy(): void} | undefined}
 */
export function startRuntimeLinkLocalization(web, getLocale) {
  if (typeof document === "undefined") return undefined;
  /** @type {Node[]} */ const pendingRoots = [];
  /** @type {Set<Node>} */ const queuedRoots = new Set();
  /** @type {LinkTraversal | undefined} */ let activeTraversal;
  /** @type {number | undefined} */ let animationFrame;
  /** @type {ReturnType<typeof setTimeout> | undefined} */ let timer;
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

  /** @param {Node} root */
  const enqueueRoot = (root) => {
    if (destroyed || !isLinkTraversalRoot(root) || queuedRoots.has(root)) return;
    if (pendingRoots.length >= MAX_PENDING_ROOTS) {
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
    let remaining = NODE_BUDGET;
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
      if (element.matches("a[href]")) localizeAnchorElement(web, getLocale(), element);
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
          } else {
            for (const node of mutation.addedNodes) {
              enqueueRoot(node);
              queued += 1;
              if (queued >= MAX_PENDING_ROOTS) break;
            }
          }
          if (queued >= MAX_PENDING_ROOTS) {
            enqueueDocument();
            return;
          }
        }
      });
  if (document.documentElement) {
    observer?.observe(document.documentElement, {
      subtree: true,
      childList: true,
      attributes: true,
      attributeFilter: ["href", "download", "rel", "data-linguini-ignore", "data-linguini-no-localize"],
    });
  }
  /** @param {Event} event */
  const onClick = (event) => {
    const anchor = (/** @type {Element | null} */ (event.target))?.closest?.("a[href]");
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

/** @typedef {{root: Element | undefined, rootPending: boolean, walker: TreeWalker}} LinkTraversal */

/** @param {Node} node */
function isLinkTraversalRoot(node) {
  return node.nodeType === 1 || node.nodeType === 9 || node.nodeType === 11;
}

/**
 * @param {Node} root
 * @returns {LinkTraversal | undefined}
 */
function createLinkTraversal(root) {
  const ownerDocument = root.nodeType === 9 ? /** @type {Document} */ (root) : root.ownerDocument;
  if (!ownerDocument) return undefined;
  return {
    root: root.nodeType === 1 ? /** @type {Element} */ (root) : undefined,
    rootPending: root.nodeType === 1,
    walker: ownerDocument.createTreeWalker(root, 0x1),
  };
}

/**
 * @param {LinkTraversal} traversal
 * @returns {Element | undefined}
 */
function nextTraversalElement(traversal) {
  if (traversal.rootPending) {
    traversal.rootPending = false;
    return traversal.root;
  }
  return (/** @type {Element | null} */ (traversal.walker.nextNode())) ?? undefined;
}

/**
 * @template {string} LocaleName
 * @param {import("../web.js").LinguiniWebLocale<LocaleName>} web
 * @param {LocaleName} locale
 * @param {Element} anchor
 */
function localizeAnchorElement(web, locale, anchor) {
  const href = anchor.getAttribute("href");
  if (!href) return;
  const input = { currentUrl: window.location.href, origin: window.location.origin };
  const attributes = {
    download: anchor.hasAttribute("download"),
    ignored: anchor.hasAttribute("data-linguini-ignore") || anchor.hasAttribute("data-linguini-no-localize"),
    rel: anchor.getAttribute("rel"),
  };
  if (!web.shouldLocalizeLink(href, attributes, input)) return;
  const localized = web.localizeHref(href, locale, input);
  if (localized !== href) anchor.setAttribute("href", localized);
}
//# sourceMappingURL=runtime-links.js.map
