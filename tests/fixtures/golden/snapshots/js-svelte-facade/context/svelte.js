import * as runtime from "./index.js";
import { getCurrentLocale, prepareLocale, setCurrentLocale } from "./svelte-locale.svelte.js";

export const linguini = createLinguiniRune(runtime);
export const l = linguini.l;
export const messages = linguini.messages;
export const setLocale = linguini.setLocale;

/** @param {typeof import("./index.js")} runtime */
function createLinguiniRune(runtime) {
  const messages = runtime.createLinguiniProvider({
    getLocale: getCurrentLocale,
  });

  /** @param {string} nextLocale @returns {Promise<import("./index.js").Locale>} */
  async function setLocale(nextLocale) {
    return setCurrentLocale(await prepareLocale(nextLocale));
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
      return runtime.getTextDirection(getCurrentLocale());
    },
    get textDirection() {
      return runtime.getTextDirection(getCurrentLocale());
    },
    get htmlAttrs() {
      const locale = getCurrentLocale();
      return { lang: locale, dir: runtime.getTextDirection(locale) };
    },
    setLocale,
  };
}
//# sourceMappingURL=svelte.js.map
