import ru from "./locales/ru.js";

const localeModules = { ru };

export function createLinguini(language) {
  return localeModules[language];
}

export function createLinguiniProvider(options) {
  return new Proxy({}, {
    get(_target, property) {
      return createLinguini(options.resolveLanguage())[property];
    },
  });
}

export function configureLinguini(options) {
  if (typeof options.language === "function") {
    return createLinguiniProvider({ resolveLanguage: options.language });
  }
  return createLinguini(options.language);
}

export const lgl = createLinguini("ru");
