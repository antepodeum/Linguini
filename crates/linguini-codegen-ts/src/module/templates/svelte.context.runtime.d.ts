import type { Locale, Linguini, TextDirection } from "./index";

export interface LinguiniRune<Locale extends string, Linguini> {
  readonly messages: Linguini;
  readonly l: Linguini;
  readonly locale: Locale;
  readonly lang: Locale;
  readonly direction: TextDirection;
  readonly textDirection: TextDirection;
  readonly htmlAttrs: { lang: Locale; dir: TextDirection };
  setLocale(locale: Locale | string): Promise<Locale>;
}

export declare const linguini: LinguiniRune<Locale, Linguini>;
export declare const l: Linguini;
export declare const messages: Linguini;
export declare const setLocale: LinguiniRune<Locale, Linguini>["setLocale"];
