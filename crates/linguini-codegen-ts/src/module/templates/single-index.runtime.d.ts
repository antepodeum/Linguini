import {{LOCALE_IDENTIFIER}} from "./locales/{{LOCALE_PATH}}";

declare const localeModules: { readonly {{LOCALE_IDENTIFIER}}: typeof {{LOCALE_IDENTIFIER}} };

type LinguiniLanguage = keyof typeof localeModules;
export type Linguini = (typeof localeModules)[LinguiniLanguage];

type LinguiniLanguageInput = LinguiniLanguage | {{LOCALE_LITERAL}};

export declare function createLinguini(language: LinguiniLanguageInput): Linguini;

export declare function createLinguiniProvider(options: {
  resolveLanguage: () => LinguiniLanguageInput;
}): Linguini;

export declare function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini;

export declare const lgl: Linguini;
