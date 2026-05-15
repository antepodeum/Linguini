import { baseLocale, getTextDirection, localizeHref, locales, type Locale } from "./index";

export type LocaleLink = {
  locale: Locale;
  href: string;
  current: boolean;
  lang: string;
  dir: "ltr" | "rtl";
};

export type LocaleLinkOptions = {
  currentLocale?: Locale;
  stripBaseLocale?: boolean;
};

export function localeLinks(pathname: string, options: LocaleLinkOptions = {}): LocaleLink[] {
  const currentLocale = options.currentLocale ?? baseLocale;
  return locales.map((locale) => ({
    locale,
    href: localizeHref(pathname, locale, options),
    current: locale === currentLocale,
    lang: locale,
    dir: getTextDirection(locale),
  }));
}

export function staticLocaleEntries(paths: readonly string[], options: LocaleLinkOptions = {}): string[] {
  return paths.flatMap((path) => locales.map((locale) => localizeHref(path, locale, options)));
}

export function htmlLocaleAttributes(locale: Locale = baseLocale) {
  return { lang: locale, dir: getTextDirection(locale) } as const;
}
