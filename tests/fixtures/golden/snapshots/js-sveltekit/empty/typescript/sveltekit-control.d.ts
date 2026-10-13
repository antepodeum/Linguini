import type { Handle, Reroute, ServerLoad } from "@sveltejs/kit";
import type { Locale, TextDirection } from "./locale";
import type { LinguiniLocaleRequestContext } from "./web";

export declare const linguiniHandle: Handle;
export declare const linguiniReroute: Reroute;
export declare const linguiniLoad: ServerLoad;
export declare const handle: Handle;
export declare const reroute: Reroute;
export declare const load: ServerLoad;

export type LinguiniServerLocaleContext<Locale extends string = string> = LinguiniLocaleRequestContext<Locale>;

export interface SerializedLinguiniContext<Locale extends string = string> {
  locale: Locale;
  baseLocale: Locale;
  locales: readonly Locale[];
  direction: TextDirection;
  lang: Locale;
  htmlAttrs: { lang: Locale; dir: TextDirection };
}

export type LinguiniSvelteKitLocaleContext = LinguiniServerLocaleContext<Locale>;
export type LinguiniSerializedContext = SerializedLinguiniContext<Locale>;
