import * as runtime from "./index";
import {
  getCurrentLocale,
  prepareLocale,
  setCurrentLocale,
} from "./svelte-locale.svelte.js";

export const linguini = createLinguiniRune(runtime);
export const l = linguini.l;
export const messages = linguini.messages;
export const setLocale = linguini.setLocale;

function createLinguiniRune(runtime: typeof import("./index")) {
  const messages = runtime.createLinguiniProvider({
    getLocale: getCurrentLocale,
  });

  async function setLocale(nextLocale: string) {
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
