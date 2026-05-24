import * as runtime from "./index";

let activeLocale = runtime.baseLocale;

export const linguini = createLinguiniRune(runtime);
export const l = linguini.l;
export const messages = linguini.messages;
export const setLocale = linguini.setLocale;

function createLinguiniRune(runtime: typeof import("./index")) {
  const messages = runtime.createLinguiniProvider({
    getLocale: () => activeLocale,
  });

  async function setLocale(nextLocale: string) {
    activeLocale = runtime.normalizeLocale(nextLocale) ?? runtime.baseLocale;
    return activeLocale;
  }

  return {
    messages,
    l: messages,
    get locale() {
      return activeLocale;
    },
    get lang() {
      return activeLocale;
    },
    get direction() {
      return runtime.getTextDirection(activeLocale);
    },
    get textDirection() {
      return runtime.getTextDirection(activeLocale);
    },
    get htmlAttrs() {
      return { lang: activeLocale, dir: runtime.getTextDirection(activeLocale) };
    },
    setLocale,
  };
}
