import ru from "./locales/ru";

declare const localeModules: { readonly ru: typeof ru };

type LinguiniLanguage = keyof typeof localeModules;
export type Linguini = (typeof localeModules)[LinguiniLanguage];

type LinguiniLanguageInput = LinguiniLanguage | "ru";

export declare function createLinguini(language: LinguiniLanguageInput): Linguini;

export declare function createLinguiniProvider(options: {
  resolveLanguage: () => LinguiniLanguageInput;
}): Linguini;

export declare function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini;

export declare const lgl: Linguini;
