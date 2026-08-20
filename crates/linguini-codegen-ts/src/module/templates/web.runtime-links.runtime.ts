import type { LinguiniWebLocale } from "../web";

const MAX_PENDING_ROOTS = 128;
const NODE_BUDGET = 256;

export function startRuntimeLinkLocalization<LocaleName extends string>(
  web: LinguiniWebLocale<LocaleName>,
  getLocale: () => LocaleName,
) {
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
  const ownerDocument = root.nodeType === 9 ? root as Document : root.ownerDocument;
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

function localizeAnchorElement<LocaleName extends string>(
  web: LinguiniWebLocale<LocaleName>,
  locale: LocaleName,
  anchor: Element,
) {
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
