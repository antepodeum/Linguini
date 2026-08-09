import type { Locale, TextDirection } from "./locale";
import type { AlternateLink, LinkLocalizationAttributes } from "./web";

export interface LinguiniSetLocaleOptions {
  navigate?: boolean;
  replaceState?: boolean;
  invalidateAll?: boolean;
  keepFocus?: boolean;
  noScroll?: boolean;
  cookie?: boolean;
  state?: {{PAGE_STATE}};
}

export interface LinguiniSvelteControl<Locale extends string> {
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

export declare const linguini: LinguiniSvelteControl<Locale>;
export declare const setLocale: LinguiniSvelteControl<Locale>["setLocale"];
export declare const localizeHref: LinguiniSvelteControl<Locale>["localizeHref"];
export declare const localizeUrl: LinguiniSvelteControl<Locale>["localizeUrl"];
export declare const shouldLocalizeHref: LinguiniSvelteControl<Locale>["shouldLocalizeHref"];
export declare const shouldLocalizeLink: LinguiniSvelteControl<Locale>["shouldLocalizeLink"];
export declare const localizeHrefAttribute: LinguiniSvelteControl<Locale>["localizeHrefAttribute"];
export declare const delocalizeUrl: LinguiniSvelteControl<Locale>["delocalizeUrl"];
export declare const alternateLinks: LinguiniSvelteControl<Locale>["alternateLinks"];
export declare const destroy: LinguiniSvelteControl<Locale>["destroy"];
