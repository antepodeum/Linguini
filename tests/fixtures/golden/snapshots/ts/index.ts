import ru from "./locales/ru";

const localeModules = { ru } as const;

type LinguiniLanguage = keyof typeof localeModules;
export type Linguini = (typeof localeModules)[LinguiniLanguage];

type LinguiniLanguageInput = LinguiniLanguage | "ru";

let currentLanguage: () => LinguiniLanguageInput = () => "ru";

function resolveLinguiniLanguage(): LinguiniLanguageInput {
  return currentLanguage();
}

export function createLinguini(language: LinguiniLanguageInput): Linguini {
  return localeModules[language as LinguiniLanguage];
}

export function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini {
  if (typeof options.language === "function") {
    currentLanguage = options.language;
  } else {
    const language = options.language;
    currentLanguage = () => language;
  }
  return lgl;
}

export const lgl: Linguini = new Proxy({} as Linguini, {
  get(_target, property) {
    return createLinguini(resolveLinguiniLanguage())[property as keyof Linguini];
  },
});
