import type { LinguiniWebLocale, LinguiniLocaleRequestContext } from './types.js';

export function createLocaleRequestContext<Locale extends string>(
  web: LinguiniWebLocale<Locale>,
  locale: Locale,
  input: Record<string, unknown> = {},
): LinguiniLocaleRequestContext<Locale> {
  const baseLocale = web.baseLocale;
  const locales = web.locales;
  const htmlAttrs = web.htmlAttrs(locale);
  return {
    locale,
    baseLocale,
    locales,
    direction: htmlAttrs.dir,
    textDirection: htmlAttrs.dir,
    lang: locale,
    htmlAttrs,
    localizeHref: (href, next = locale, context = input) => web.localizeHref(href, next, context),
    localizeUrl: (url, next = locale, context = input) => web.localizeUrl(url, next, context),
    delocalizeUrl: (url, context = input) => web.delocalizeUrl(url, context),
  };
}
