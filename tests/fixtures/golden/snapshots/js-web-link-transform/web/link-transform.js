import { web } from "../svelte-effects.svelte.js";
import { getCurrentLocale } from "../svelte-locale.svelte.js";

/**
 * @param {string} href
 * @param {{download?: boolean, ignored?: boolean, rel?: string | null}} [attributes={}]
 * @param {Record<string, unknown>} [input={}]
 * @returns {string}
 */
export function localizeTransformedHref(href, attributes = {}, input = {}) {
  return web.shouldLocalizeLink(href, attributes, input)
    ? web.localizeHref(href, getCurrentLocale(), input)
    : href;
}
//# sourceMappingURL=link-transform.js.map
