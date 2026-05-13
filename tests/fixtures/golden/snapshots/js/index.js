import ru from "./locales/ru.js";

const localeModules = { ru };

let currentLanguage = () => "ru";

function resolveLinguiniLanguage() {
  return currentLanguage();
}

export function createLinguini(language) {
  return localeModules[language];
}

export function configureLinguini(options) {
  if (typeof options.language === "function") {
    currentLanguage = options.language;
  } else {
    const language = options.language;
    currentLanguage = () => language;
  }
  return lgl;
}

export const lgl = new Proxy({}, {
  get(_target, property) {
    return createLinguini(resolveLinguiniLanguage())[property];
  },
});
