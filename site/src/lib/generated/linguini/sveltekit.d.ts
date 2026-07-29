import type { Handle, Reroute, ServerLoad } from "@sveltejs/kit";
import type { LinguiniRequestContext } from "./web";
import type { Locale, Linguini, TextDirection } from "./index";

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
      linguini: LinguiniRequestContext<Locale, Linguini>;
      locale: Locale;
      direction: TextDirection;
      l: Linguini;
    }
    interface PageData {
      linguini?: SerializedLinguiniContext<Locale>;
    }
  }
}
