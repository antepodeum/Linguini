import type { TextDirection } from '@antepodeum/linguini-web';

export interface SerializedLinguiniContext<Locale extends string = string> {
  locale: Locale;
  baseLocale: Locale;
  locales: readonly Locale[];
  direction: TextDirection;
  lang: Locale;
  htmlAttrs: { lang: Locale; dir: TextDirection };
}
