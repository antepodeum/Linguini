import type { AlternateLink, LinkLocalizationAttributes } from "./web";
import type { Locale, Linguini, TextDirection } from "./index";

export interface LinguiniSetLocaleOptions {
  navigate?: boolean;
  replaceState?: boolean;
  invalidateAll?: boolean;
  keepFocus?: boolean;
  noScroll?: boolean;
  cookie?: boolean;
  state?: App.PageState;
}

export interface LinguiniRune<Locale extends string, Linguini> {
  readonly messages: Linguini;
  readonly l: Linguini;
  readonly locale: Locale;
  readonly lang: Locale;
  readonly direction: TextDirection;
  readonly textDirection: TextDirection;
  readonly htmlAttrs: { lang: Locale; dir: TextDirection };
  setLocale(locale: Locale | string, options?: LinguiniSetLocaleOptions): Promise<Locale>;
  localizeHref(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  localizeUrl(url: string | URL, locale?: Locale, input?: Record<string, unknown>): URL;
  shouldLocalizeHref(href: string, input?: Record<string, unknown>): boolean;
  shouldLocalizeLink(href: string, attributes?: LinkLocalizationAttributes, input?: Record<string, unknown>): boolean;
  localizeHrefAttribute(href: string, locale?: Locale, input?: Record<string, unknown>): string;
  delocalizeUrl(url: string | URL, input?: Record<string, unknown>): URL;
  alternateLinks(url: string | URL, input?: Record<string, unknown>): AlternateLink[];
  destroy(): void;
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
