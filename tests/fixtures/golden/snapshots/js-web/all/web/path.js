/**
 * @template {string} Locale
 * @param {Record<string, unknown>} input
 * @param {{environment: {base: string}}} options
 * @param {readonly Locale[]} locales
 * @returns {Locale | undefined}
 */
export function resolvePathLocale(input, options, locales) {
  const value = /** @type {URL | string | undefined} */ (input.url);
  if (!value) return undefined;
  /** @type {URL} */ let parsed;
  try {
    parsed = new URL(String(value), "http://localhost");
  } catch {
    return undefined;
  }
  const pathname = parsed.pathname.startsWith("/") ? parsed.pathname : `/${parsed.pathname}`;
  const base = options.environment.base && options.environment.base !== "/"
    ? `/${options.environment.base.replace(/^\/+|\/+$/g, "")}`
    : "";
  const stripped = base && pathname === base
    ? "/"
    : base && pathname.startsWith(`${base}/`)
      ? pathname.slice(base.length)
      : pathname;
  const segment = stripped.split("/").filter(Boolean)[0];
  return typeof segment === "string"
    ? locales.find((locale) => locale.toLowerCase() === segment.toLowerCase())
    : undefined;
}
//# sourceMappingURL=path.js.map
