import * as runtime from "./index";
import { linguini as controls } from "./svelte-control.js";
import { registerLocaleLoader } from "./svelte-locale.svelte.js";

const unregisterRuntimeLoader = registerLocaleLoader(async (locale) => {
  await runtime.prepareLinguini(locale);
});
const hot = (import.meta as ImportMeta & {
  hot?: { dispose(callback: () => void): void };
}).hot;
hot?.dispose(unregisterRuntimeLoader);

export const linguini = createLinguiniRune(runtime, controls);
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

function createLinguiniRune(
  runtime: typeof import("./index"),
  controls: typeof import("./svelte-control.js").linguini,
) {
  const messages = runtime.createLinguiniProvider({
    getLocale: () => controls.locale,
  });

  return {
    messages,
    l: messages,
    get locale() {
      return controls.locale;
    },
    get lang() {
      return controls.lang;
    },
    get direction() {
      return controls.direction;
    },
    get textDirection() {
      return controls.textDirection;
    },
    get htmlAttrs() {
      return controls.htmlAttrs;
    },
    setLocale: controls.setLocale,
    localizeHref: controls.localizeHref,
    localizeUrl: controls.localizeUrl,
    shouldLocalizeHref: controls.shouldLocalizeHref,
    shouldLocalizeLink: controls.shouldLocalizeLink,
    localizeHrefAttribute: controls.localizeHrefAttribute,
    delocalizeUrl: controls.delocalizeUrl,
    alternateLinks: controls.alternateLinks,
    destroy: controls.destroy,
  };
}
