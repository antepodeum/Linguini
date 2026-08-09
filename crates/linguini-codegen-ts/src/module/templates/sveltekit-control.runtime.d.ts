import type { Handle, Reroute, ServerLoad } from "@sveltejs/kit";
import type { Locale, TextDirection } from "./locale";
import type { AlternateLink, LinkLocalizationAttributes } from "./web";

export declare const linguiniHandle: Handle;
export declare const linguiniReroute: Reroute;
export declare const linguiniLoad: ServerLoad;
export declare const handle: Handle;
export declare const reroute: Reroute;
export declare const load: ServerLoad;

export interface LinguiniServerLocaleContext<Locale extends string = string> {
  locale: Locale;
  baseLocale: Locale;
  locales: readonly Locale[];
  direction: TextDirection;
  textDirection: TextDirection;
  lang: Locale;
  htmlAttrs: { lang: Locale; dir: TextDirection };
  localizeHref(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  localizeUrl(url: string | URL, locale?: Locale, input?: Record<string, unknown>): URL;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  shouldLocalizeLink(href: string, attributes?: LinkLocalizationAttributes, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
}

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
