import { describe, expect, test } from "bun:test";
import { startRuntimeLinkLocalization as javascript } from "./web/runtime-links.js";
import { startRuntimeLinkLocalization as typescript } from "./typescript/web/runtime-links.ts";
import { withLinkBrowser } from "./browser-harness.ts";

const web = {
  shouldLocalizeLink: (href: string, attributes: { download?: boolean; ignored?: boolean }) =>
    href.startsWith("/") && !attributes.download && !attributes.ignored,
  localizeHref: (href: string, locale: string) => `/${locale}${href.replace(/^\/(?:en|de)/, "")}`,
};

describe("owned runtime-link target parity", () => {
  test("server initialization has no browser effects", () => {
    expect(javascript(web, () => "en")).toBeUndefined();
    expect(typescript(web, () => "en")).toBeUndefined();
  });

  for (const animation of [true, false]) {
    test(`${animation ? "animation" : "timer"} scheduling stays bounded and cleanup is owned`, () => {
      const run = (start: typeof javascript) => withLinkBrowser(animation, (browser) => {
        let locale = "en";
        const effects = start(web, () => locale)!;
        browser.flushOne();
        expect(browser.anchors.reduce((sum, anchor) => sum + anchor.visits, 0)).toBe(255);
        expect(browser.frames.size + browser.timers.size).toBe(1);
        browser.flushAll();
        expect(browser.anchors[0].getAttribute("href")).toBe("/en/item/0");
        expect(browser.anchors[1].getAttribute("href")).toBe("/item/1");
        expect(browser.anchors[2].getAttribute("href")).toBe("/item/2");
        locale = "de";
        browser.listeners.get("click")?.({ target: browser.anchors[0] });
        expect(browser.anchors[0].getAttribute("href")).toBe("/de/item/0");
        browser.mutate([{ type: "childList", addedNodes: browser.anchors.slice(0, 200) }]);
        browser.flushAll();
        expect(browser.anchors[599].getAttribute("href")).toBe("/de/item/599");
        effects.refresh();
        expect(browser.frames.size + browser.timers.size).toBe(1);
        effects.destroy();
        expect(browser.listeners.size).toBe(0);
        expect(browser.frames.size + browser.timers.size).toBe(0);
        expect(browser.disconnected()).toBe(1);
        effects.refresh();
        expect(browser.frames.size + browser.timers.size).toBe(0);
        return browser.anchors.map((anchor) => [anchor.getAttribute("href"), anchor.writes]);
      });
      expect(run(javascript)).toEqual(run(typescript));
    });
  }
});
