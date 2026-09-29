/**
 * @template {string} Locale
 * @param {Record<string, unknown>} input
 * @param {{cookie: {name: string}}} options
 * @param {(value: unknown) => Locale | undefined} matchLocale
 * @returns {Locale | undefined}
 */
export function resolveCookieLocale(input, options, matchLocale) {
  for (const cookie of String((/** @type {string | undefined} */ (input.cookie)) ?? "").split(";")) {
    const [rawName, ...rawValue] = cookie.trim().split("=");
    if (rawName !== options.cookie.name) continue;
    try {
      return matchLocale(decodeURIComponent(rawValue.join("=")));
    } catch {
      return undefined;
    }
  }
  return undefined;
}
//# sourceMappingURL=cookie.js.map
