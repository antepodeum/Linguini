import ru from "./locales/ru";

const localeModules = { ru } as const;

type LinguiniLanguage = keyof typeof localeModules;
export type Linguini = (typeof localeModules)[LinguiniLanguage];

type LinguiniLanguageInput = LinguiniLanguage | "ru";

export function createLinguini(language: LinguiniLanguageInput): Linguini {
  return localeModules[language as LinguiniLanguage];
}

export function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): { readonly lgl: Linguini } {
  return {
    get lgl() {
      const language = typeof options.language === "function" ? options.language() : options.language;
      return createLinguini(language);
    },
  };
}
