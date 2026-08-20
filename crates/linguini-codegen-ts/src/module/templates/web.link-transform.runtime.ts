import type { LinkLocalizationAttributes } from "../web";
import { web } from "../svelte-effects.svelte.js";
import { getCurrentLocale } from "../svelte-locale.svelte.js";

export function localizeTransformedHref(
  href: string,
  attributes: LinkLocalizationAttributes = {},
  input: Record<string, unknown> = {},
) {
  return web.shouldLocalizeLink(href, attributes, input)
    ? web.localizeHref(href, getCurrentLocale(), input)
    : href;
}
