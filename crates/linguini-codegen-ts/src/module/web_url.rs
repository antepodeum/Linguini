//! Shared url statements for the web facade.

use crate::ecmascript::RuntimeCode;

pub(super) fn append(code: &mut RuntimeCode) {
    code.shared("\n");
    code.typed("function matchLocaleValue<Locale extends string>(locales: readonly Locale[], value: unknown): Locale | undefined {", "/** @template {string} Locale @param {readonly Locale[]} locales @param {unknown} value @returns {Locale | undefined} */\nfunction matchLocaleValue(locales, value) {");
    code.shared("\n  const candidates = Array.isArray(value) ? value : [value];\n  for (const candidate of candidates) {\n    if (typeof candidate !== \"string\") continue;\n    const exact = locales.find(\n      (locale) => locale.toLowerCase() === candidate.toLowerCase(),\n    );\n    if (exact) return exact;\n  }\n  return undefined;\n}\n\n");
    code.typed("function matchExactLocale<Locale extends string>(locales: readonly Locale[], value: unknown): Locale | undefined {", "/** @template {string} Locale @param {readonly Locale[]} locales @param {unknown} value @returns {Locale | undefined} */\nfunction matchExactLocale(locales, value) {");
    code.shared("\n  if (typeof value !== \"string\") return undefined;\n  return locales.find((locale) => locale.toLowerCase() === value.toLowerCase());\n}\n\n\n");
    code.typed("function stripLeadingLocale<Locale extends string>(pathname: string, locales: readonly Locale[]) {", "/** @template {string} Locale @param {string} pathname @param {readonly Locale[]} locales @returns {string} */\nfunction stripLeadingLocale(pathname, locales) {");
    code.shared("\n  const parts = pathname.split(\"/\").filter(Boolean);\n  return matchExactLocale(locales, parts[0]) ? ensureSlash(parts.slice(1).join(\"/\")) : ensureSlash(parts.join(\"/\"));\n}\n\n");
    code.typed("function stripBasePath(pathname: string, basePath: string) {", "/** @param {string} pathname @param {string} basePath @returns {string} */\nfunction stripBasePath(pathname, basePath) {");
    code.shared("\n  const normalizedPath = ensureSlash(pathname);\n  const normalizedBase = basePath && basePath !== \"/\" ? ensureSlash(basePath).replace(/\\/$/, \"\") : \"\";\n  if (!normalizedBase) return normalizedPath;\n  if (normalizedPath === normalizedBase) return \"/\";\n  return normalizedPath.startsWith(`${normalizedBase}/`) ? normalizedPath.slice(normalizedBase.length) || \"/\" : normalizedPath;\n}\n\n");
    code.typed("function joinPath(...parts: Array<string | undefined>) {", "/** @param {...(string | undefined)} parts @returns {string} */\nfunction joinPath(...parts) {");
    code.shared("\n  const joined = parts\n    .filter((part) => part !== undefined && part !== \"\")\n    .map((part) => String(part).replace(/^\\/+|\\/+$/g, \"\"))\n    .filter(Boolean)\n    .join(\"/\");\n  return `/${joined}`.replace(/\\/+/g, \"/\");\n}\n\n");
    code.typed(
        "function ensureSlash(pathname: string) {",
        "/** @param {string} pathname @returns {string} */\nfunction ensureSlash(pathname) {",
    );
    code.shared("\n  const value = String(pathname || \"/\");\n  return value.startsWith(\"/\") ? value : `/${value}`;\n}\n\n");
    code.typed("function shouldPrefixLocale(options: ReturnType<typeof normalizeOptions>, locale: string) {", "/** @param {ReturnType<typeof normalizeOptions>} options @param {string} locale @returns {boolean} */\nfunction shouldPrefixLocale(options, locale) {");
    code.shared("\n  switch (options.routing.localePrefix) {\n    case \"always\":\n      return true;\n    case \"never\":\n      return false;\n    case \"except-default\":\n    default:\n      return locale !== options.baseLocale;\n  }\n}\n\n\n");
    code.typed("function resolveBaseUrl(options: ReturnType<typeof normalizeOptions>, input: Record<string, unknown> = {}) {", "/** @param {ReturnType<typeof normalizeOptions>} options @param {Record<string, unknown>} [input] @returns {URL} */\nfunction resolveBaseUrl(options, input = {}) {");
    code.shared("\n  const origin = parseUrl(input.origin ?? options.environment.origin ?? browserLocationHref() ?? FALLBACK_URL, FALLBACK_URL);\n  const current = input.currentUrl ?? input.url;\n  return current === undefined ? origin : parseUrl(current, origin);\n}\n\n");
    code.typed(
        "function normalizeBasePath(value: string) {",
        "/** @param {string} value @returns {string} */\nfunction normalizeBasePath(value) {",
    );
    code.shared("\n  if (!value || value === \"/\") return \"\";\n  return ensureSlash(value).replace(/\\/+$/, \"\");\n}\n\n");
    code.typed("function resolveEnvironmentProtocol(origin: string | URL | undefined) {", "/** @param {string | URL | undefined} origin @returns {string} */\nfunction resolveEnvironmentProtocol(origin) {");
    code.shared("\n  try {\n    return parseUrl(origin ?? browserLocationHref() ?? FALLBACK_URL, FALLBACK_URL).protocol;\n  } catch {\n    return \"http:\";\n  }\n}\n\n");
    code.typed("function resolveCookieSecure(policy: boolean | \"auto\", input: Record<string, unknown>) {", "/** @param {boolean | \"auto\"} policy @param {Record<string, unknown>} input @returns {boolean} */\nfunction resolveCookieSecure(policy, input) {");
    code.shared("\n  if (typeof input.secure === \"boolean\") return input.secure;\n  if (policy !== \"auto\") return policy;\n  return resolveEnvironmentProtocol(");
    code.typed(
        "input.origin as string | URL | undefined",
        "/** @type {string | URL | undefined} */ (input.origin)",
    );
    code.shared(") === \"https:\";\n}\n\n");
    code.typed("function parseRuntimeUrl(url: string | URL, options: ReturnType<typeof normalizeOptions>, input: Record<string, unknown> = {}) {", "/** @param {string | URL} url @param {ReturnType<typeof normalizeOptions>} options @param {Record<string, unknown>} [input] @returns {URL} */\nfunction parseRuntimeUrl(url, options, input = {}) {");
    code.shared("\n  return parseUrl(url, resolveBaseUrl(options, input));\n}\n\n");
    code.typed("function parseUrl(value: unknown, base: string | URL) {", "/** @param {unknown} value @param {string | URL} base @returns {URL} */\nfunction parseUrl(value, base) {");
    code.shared("\n  try {\n    return new URL(String(value), String(base));\n  } catch {\n    throw new TypeError(INVALID_URL_MESSAGE);\n  }\n}\n\n");
    code.typed("function isInternalHttpUrl(url: URL, origin: string) {", "/** @param {URL} url @param {string} origin @returns {boolean} */\nfunction isInternalHttpUrl(url, origin) {");
    code.shared("\n  return [\"http:\", \"https:\"].includes(url.protocol) && url.origin === origin;\n}\n\nfunction browserLocationHref() {\n  const href = ");
    code.typed(
        "(globalThis as { location?: { href?: unknown } }).location?.href",
        "(/** @type {{ location?: { href?: unknown } }} */ (globalThis)).location?.href",
    );
    code.shared(";\n  return typeof href === \"string\" ? href : undefined;\n}\n");
}
