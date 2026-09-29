import type { Locale, Linguini } from "./index";
import type { LinguiniSvelteControl } from "./svelte-control.js";
export type { LinguiniSetLocaleOptions } from "./svelte-control.js";

export interface LinguiniRune<Locale extends string, Linguini> extends LinguiniSvelteControl<Locale> {
  readonly messages: Linguini;
  readonly l: Linguini;
}

export declare const linguini: LinguiniRune<Locale, Linguini>;
export declare const l: Linguini;
export declare const messages: Linguini;
export declare const setLocale: LinguiniRune<Locale, Linguini>["setLocale"];
export declare const localizeHref: LinguiniRune<Locale, Linguini>["localizeHref"];
export declare const localizeUrl: LinguiniRune<Locale, Linguini>["localizeUrl"];
export declare const shouldLocalizeHref: LinguiniRune<Locale, Linguini>["shouldLocalizeHref"];
export declare const shouldLocalizeLink: LinguiniRune<Locale, Linguini>["shouldLocalizeLink"];
export declare const localizeHrefAttribute: LinguiniRune<Locale, Linguini>["localizeHrefAttribute"];
export declare const delocalizeUrl: LinguiniRune<Locale, Linguini>["delocalizeUrl"];
export declare const alternateLinks: LinguiniRune<Locale, Linguini>["alternateLinks"];
