import type { LinguiniRune } from "@antepod/linguini-sveltekit/client";
import type { Locale, Linguini, TextDirection } from "./index";

export declare const linguini: LinguiniRune<Locale, Linguini>;
export declare const l: Linguini;
export declare const messages: Linguini;
export declare const setLocale: LinguiniRune<Locale, Linguini>["setLocale"];
export declare const localizeHref: LinguiniRune<Locale, Linguini>["localizeHref"];
export declare const localizeUrl: LinguiniRune<Locale, Linguini>["localizeUrl"];
export declare const delocalizeUrl: LinguiniRune<Locale, Linguini>["delocalizeUrl"];
export declare const alternateLinks: LinguiniRune<Locale, Linguini>["alternateLinks"];
