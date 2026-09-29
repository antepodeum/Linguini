/** @template {string} Locale @param {import("../web.js").LinguiniWebLocale<Locale>} web @param {unknown} target @param {Locale} locale @param {Record<string, unknown>} [input] @returns {void} */
export function persistLocaleCookie(web, target, locale, input = {}) {
  web.setLocaleCookie(target, locale, input);
}
//# sourceMappingURL=server-cookie.js.map
