import {{LOCALE_IDENTIFIER}} from "./locales/{{LOCALE_PATH}}";
export type * from "./shared";
import type { LinguiniMessages } from "./messages";
export type { LinguiniMessages } from "./messages";

declare const localeModules: { readonly {{LOCALE_IDENTIFIER}}: typeof {{LOCALE_IDENTIFIER}} };

type LinguiniLanguage = keyof typeof localeModules;
export type Linguini = LinguiniMessages;

type LinguiniLanguageInput = LinguiniLanguage | {{LOCALE_LITERAL}};

export declare function createLinguini(language: LinguiniLanguageInput): Linguini;

export declare function createLinguiniProvider(options: {
  resolveLanguage: () => LinguiniLanguageInput;
}): Linguini;

export declare function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini;

export declare const lgl: Linguini;
