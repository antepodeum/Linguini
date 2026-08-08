export type LinguiniProviderOptions = {
  getLocale?: () => LinguiniLanguageInput;
  resolveLanguage?: () => LinguiniLanguageInput;
};

export declare function createLinguini(language: LinguiniLanguageInput): Linguini;

export declare function createLinguiniProvider(options?: LinguiniProviderOptions): Linguini;

export declare function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini;

export declare const lgl: Linguini;
