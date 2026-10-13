import type { Handle, Reroute, ServerLoad } from "@sveltejs/kit";
import type { LinguiniLocaleRequestContext, LinguiniRequestContext } from "./web";
import type { Locale, Linguini, TextDirection } from "./index";

export declare const linguiniHandle: Handle;
export declare const linguiniReroute: Reroute;
export declare const linguiniLoad: ServerLoad;
export declare const handle: Handle;
export declare const reroute: Reroute;
export declare const load: ServerLoad;

export interface SerializedLinguiniContext<Locale extends string = string> {
  locale: Locale;
  baseLocale: Locale;
  locales: readonly Locale[];
  direction: TextDirection;
  lang: Locale;
  htmlAttrs: { lang: Locale; dir: TextDirection };
}

declare global {
  namespace App {
    interface Locals {
      linguini: LinguiniLocaleRequestContext<Locale> | LinguiniRequestContext<Locale, Linguini>;
    }
    interface PageData {
      linguini?: SerializedLinguiniContext<Locale>;
    }
  }
}
