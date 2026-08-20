import type { LinguiniWebLocale, LinkLocalizationAttributes } from "../web";

export function localizeTransformedHref<Locale extends string>(
  web: LinguiniWebLocale<Locale>,
  getLocale: () => Locale,
  href: string,
  attributes: LinkLocalizationAttributes = {},
  input: Record<string, unknown> = {},
) {
  return web.shouldLocalizeLink(href, attributes, input)
    ? web.localizeHref(href, getLocale(), input)
    : href;
}
