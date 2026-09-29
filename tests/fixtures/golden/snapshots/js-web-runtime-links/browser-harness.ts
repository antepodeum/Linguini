export class LinkElement {
  readonly nodeType = 1;
  ownerDocument: unknown;
  visits = 0;
  writes = 0;
  children: LinkElement[] = [];
  readonly attributes = new Map<string, string>();

  constructor(href?: string) {
    if (href !== undefined) this.attributes.set("href", href);
  }

  matches() {
    this.visits += 1;
    return this.attributes.has("href");
  }

  closest() { return this.attributes.has("href") ? this : null; }
  getAttribute(name: string) { return this.attributes.get(name) ?? null; }
  hasAttribute(name: string) { return this.attributes.has(name); }
  setAttribute(name: string, value: string) { this.writes += 1; this.attributes.set(name, value); }
}

export function withLinkBrowser<T>(animation: boolean, run: (browser: ReturnType<typeof createBrowser>) => T): T {
  const browser = createBrowser(animation);
  const originals = new Map<string, PropertyDescriptor | undefined>();
  for (const [key, value] of Object.entries(browser.globals)) {
    originals.set(key, Object.getOwnPropertyDescriptor(globalThis, key));
    Object.defineProperty(globalThis, key, { configurable: true, writable: true, value });
  }
  try { return run(browser); }
  finally {
    for (const [key, descriptor] of originals) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else Reflect.deleteProperty(globalThis, key);
    }
  }
}

function createBrowser(animation: boolean) {
  let nonce = 0;
  let disconnected = 0;
  let observerCallback: ((mutations: unknown[]) => void) | undefined;
  const frames = new Map<number, () => void>();
  const timers = new Map<number, () => void>();
  const listeners = new Map<string, (event: unknown) => void>();
  const root = new LinkElement();
  const anchors = Array.from({ length: 600 }, (_, index) => new LinkElement(`/item/${index}`));
  root.children = anchors;
  anchors[1].attributes.set("download", "");
  anchors[2].attributes.set("data-linguini-ignore", "");
  const document = {
    nodeType: 9,
    readyState: "complete",
    documentElement: root,
    createTreeWalker(node: LinkElement) {
      let index = 0;
      const descendants = node.children;
      return { nextNode: () => descendants[index++] ?? null };
    },
    addEventListener(name: string, callback: (event: unknown) => void) { listeners.set(name, callback); },
    removeEventListener(name: string) { listeners.delete(name); },
  };
  for (const element of [root, ...anchors]) element.ownerDocument = document;
  const globals = {
    document,
    window: { location: { href: "https://example.test/", origin: "https://example.test" } },
    requestAnimationFrame: animation ? (callback: () => void) => { frames.set(++nonce, callback); return nonce; } : undefined,
    cancelAnimationFrame: (id: number) => frames.delete(id),
    setTimeout: (callback: () => void) => { timers.set(++nonce, callback); return nonce; },
    clearTimeout: (id: number) => timers.delete(id),
    MutationObserver: class {
      constructor(callback: (mutations: unknown[]) => void) { observerCallback = callback; }
      observe() {}
      disconnect() { disconnected += 1; }
    },
  };
  function flushOne() {
    const queue = animation ? frames : timers;
    const callbacks = [...queue.values()];
    queue.clear();
    for (const callback of callbacks) callback();
  }
  function flushAll() {
    let ticks = 0;
    while (frames.size || timers.size) {
      if (++ticks > 20) throw new Error("link scheduler did not settle");
      flushOne();
    }
  }
  return {
    globals, root, anchors, listeners, frames, timers, flushOne, flushAll,
    mutate: (mutations: unknown[]) => observerCallback?.(mutations),
    disconnected: () => disconnected,
  };
}
