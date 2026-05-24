import {{LOCALE_IDENTIFIER}} from "./locales/{{LOCALE_PATH}}";
export type * from "./shared";

const localeModules = { {{LOCALE_IDENTIFIER}} } as const;

type LinguiniLanguage = keyof typeof localeModules;
export type Linguini = (typeof localeModules)[LinguiniLanguage];

type LinguiniLanguageInput = LinguiniLanguage | {{LOCALE_LITERAL}};

export function createLinguini(language: LinguiniLanguageInput): Linguini {
  return localeModules[language as LinguiniLanguage];
}

export function createLinguiniProvider(options: {
  resolveLanguage: () => LinguiniLanguageInput;
}): Linguini {
  return new Proxy({} as Linguini, {
    get(_target, property) {
      return createLinguini(options.resolveLanguage())[property as keyof Linguini];
    },
  });
}

export function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini {
  if (typeof options.language === "function") {
    return createLinguiniProvider({ resolveLanguage: options.language });
  }
  return createLinguini(options.language);
}

export const lgl: Linguini = createLinguini({{LOCALE_LITERAL}});
