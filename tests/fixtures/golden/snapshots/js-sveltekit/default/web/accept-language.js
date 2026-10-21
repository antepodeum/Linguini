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
    const range = rawRange.trim().toLowerCase();
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
  const normalizedBase = baseLocale.toLowerCase();
  /** @type {Locale | undefined} */ let bestLocale;
  let bestQuality = -1;
  let bestPreferenceIndex = Infinity;
  let bestIsBase = false;
  for (const locale of locales) {
    const normalizedLocale = locale.toLowerCase();
    /** @type {LanguagePreference | undefined} */ let preference;
    for (const candidate of preferences) {
      // Input order breaks equal-specificity ties; a more specific q=0 still excludes.
      if ((!preference || candidate.specificity > preference.specificity) &&
        languageRangeMatches(candidate.range, normalizedLocale)) preference = candidate;
    }
    if (!preference || preference.quality <= 0) continue;
    const isBase = normalizedLocale === normalizedBase;
    if (preference.quality > bestQuality ||
      (preference.quality === bestQuality && preference.index < bestPreferenceIndex) ||
      (preference.quality === bestQuality && preference.index === bestPreferenceIndex && isBase && !bestIsBase)) {
      bestLocale = locale;
      bestQuality = preference.quality;
      bestPreferenceIndex = preference.index;
      bestIsBase = isBase;
    }
  }
  return bestLocale;
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
 * @param {string} normalizedLocale
 * @returns {boolean}
 */
function languageRangeMatches(range, normalizedLocale) {
  if (range === "*") return true;
  return range === normalizedLocale || normalizedLocale.startsWith(`${range}-`) || range.startsWith(`${normalizedLocale}-`);
}
//# sourceMappingURL=accept-language.js.map
