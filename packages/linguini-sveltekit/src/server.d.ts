import type { Handle, Reroute, ServerLoad } from "@sveltejs/kit";
import type { LinguiniRuntime, LinguiniWebOptions, LinguiniRequestContext, TextDirection } from "@antepod/linguini-web";

export interface LinguiniSvelteKitOptions extends LinguiniWebOptions {
  web?: LinguiniWebOptions;
  redirectStatus?: 301 | 302 | 303 | 307 | 308;
  persistCookie?: boolean;
}

export interface SerializedLinguiniContext<Locale extends string = string> {
  locale: Locale;
  baseLocale: Locale;
  locales: readonly Locale[];
  direction: TextDirection;
  lang: Locale;
  htmlAttrs: { lang: Locale; dir: TextDirection };
}

export declare function createHandle<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options?: LinguiniSvelteKitOptions): Handle;
export declare function createReroute<Locale extends string, Linguini>(runtime: LinguiniRuntime<Locale, Linguini>, options?: LinguiniSvelteKitOptions): Reroute;
export declare function createLoad<Locale extends string = string>(): ServerLoad;
export declare function serializeContext<Locale extends string = string, Linguini = unknown>(context?: LinguiniRequestContext<Locale, Linguini>): SerializedLinguiniContext<Locale> | undefined;
