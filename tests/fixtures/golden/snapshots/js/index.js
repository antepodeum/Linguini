import ru from "./locales/ru.js";

const localeModules = { ru };

export function createLinguini(language) {
  return localeModules[language];
}

export function configureLinguini(options) {
  return {
    get lgl() {
      const language = typeof options.language === "function" ? options.language() : options.language;
      return createLinguini(language);
    },
  };
}
