/** @typedef {{range: string, quality: number, index: number, specificity: number}} LanguagePreference */

/**
 * @template {string} Locale
 * @param {Record<string, unknown>} input
 * @param {readonly Locale[]} locales
 * @param {Locale} baseLocale
 * @param {(value: unknown) => Locale | undefined} matchLocale
 * @returns {Locale | undefined}
 */
export function resolveAcceptLanguageLocale(input, locales, baseLocale, matchLocale) {
  const header = readHeader(input.headers, "accept-language");
  const value = header !== undefined
    ? resolveAcceptLanguage(locales, baseLocale, header)
    : resolveNavigatorLanguage(locales, baseLocale, input.navigator);
  return matchLocale(value);
}

/**
 * @param {unknown} headers
 * @param {string} name
 * @returns {string | undefined}
 */
function readHeader(headers, name) {
  if (!headers) return undefined;
  const getter = (/** @type {{get?: (header: string) => string | null | undefined}} */ (headers)).get;
  if (typeof getter !== "function") return undefined;
  try {
    return getter.call(headers, name) ?? undefined;
  } catch {
    return undefined;
  }
}

/**
 * @param {string | null | undefined} header
 * @returns {LanguagePreference[]}
 */
function parseAcceptLanguage(header) {
  if (!header) return [];
  return String(header).split(",").flatMap((part, index) => {
    const [rawRange, ...parameters] = part.split(";");
    const range = rawRange.trim();
    if (!/^(?:\*|[A-Za-z]{1,8}(?:-[A-Za-z0-9]{1,8})*)$/.test(range)) return [];
    let quality = 1;
    for (const parameter of parameters) {
      const [rawName, ...rawValue] = parameter.split("=");
      if (rawName.trim().toLowerCase() !== "q") continue;
      const value = rawValue.join("=").trim();
      if (!/^(?:0(?:\.\d{0,3})?|1(?:\.0{0,3})?)$/.test(value)) return [];
      quality = Number(value);
      break;
    }
    return [{ range, quality, index, specificity: range === "*" ? 0 : range.split("-").length }];
  });
}

/**
 * @template {string} Locale
 * @param {readonly Locale[]} locales
 * @param {Locale} baseLocale
 * @param {string | null | undefined} header
 * @returns {Locale | undefined}
 */
function resolveAcceptLanguage(locales, baseLocale, header) {
  const preferences = parseAcceptLanguage(header);
  /** @type {{locale: Locale, quality: number, preferenceIndex: number, base: boolean, localeIndex: number} | undefined} */ let best;
  for (const [localeIndex, locale] of locales.entries()) {
    const preference = preferences
      .filter((candidate) => languageRangeMatches(candidate.range, locale))
      .sort((left, right) => right.specificity - left.specificity || left.index - right.index)[0];
    if (!preference || preference.quality <= 0) continue;
    const candidate = { locale, quality: preference.quality, preferenceIndex: preference.index, base: locale.toLowerCase() === baseLocale.toLowerCase(), localeIndex };
    if (!best || candidate.quality > best.quality ||
      (candidate.quality === best.quality && candidate.preferenceIndex < best.preferenceIndex) ||
      (candidate.quality === best.quality && candidate.preferenceIndex === best.preferenceIndex && candidate.base && !best.base) ||
      (candidate.quality === best.quality && candidate.preferenceIndex === best.preferenceIndex && candidate.base === best.base && candidate.localeIndex < best.localeIndex)) best = candidate;
  }
  return best?.locale;
}

/**
 * @template {string} Locale
 * @param {readonly Locale[]} locales
 * @param {Locale} baseLocale
 * @param {unknown} value
 * @returns {Locale | undefined}
 */
function resolveNavigatorLanguage(locales, baseLocale, value) {
  if (!value || (typeof value !== "object" && typeof value !== "function")) return undefined;
  /** @type {string[]} */ const preferences = [];
  try {
    const languages = (/** @type {{languages?: unknown}} */ (value)).languages;
    if (Array.isArray(languages)) for (const language of languages) if (typeof language === "string") preferences.push(language);
  } catch {}
  try {
    const language = (/** @type {{language?: unknown}} */ (value)).language;
    if (typeof language === "string") preferences.push(language);
  } catch {}
  return preferences.length === 0 ? undefined : resolveAcceptLanguage(locales, baseLocale, preferences.join(","));
}

/**
 * @param {string} range
 * @param {string} locale
 * @returns {boolean}
 */
function languageRangeMatches(range, locale) {
  if (range === "*") return true;
  const normalizedRange = range.toLowerCase();
  const normalizedLocale = locale.toLowerCase();
  return normalizedRange === normalizedLocale || normalizedLocale.startsWith(`${normalizedRange}-`) || normalizedRange.startsWith(`${normalizedLocale}-`);
}
//# sourceMappingURL=accept-language.js.map
