import type { TextDirection } from '@antepodeum/linguini-web';

export interface PagePayload {
  url: string;
}

export interface SerializedLinguiniContext<Locale extends string = string> {
  locale: Locale;
  payload?: PagePayload;
  baseLocale: Locale;
  locales: readonly Locale[];
  direction: TextDirection;
  lang: Locale;
  htmlAttrs: { lang: Locale; dir: TextDirection };
}
