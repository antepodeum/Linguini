type LanguagePreference = {
  range: string;
  quality: number;
  index: number;
  specificity: number;
};

export function resolveAcceptLanguageLocale<Locale extends string>(
  input: Record<string, unknown>,
  locales: readonly Locale[],
  baseLocale: Locale,
  matchLocale: (value: unknown) => Locale | undefined,
): Locale | undefined {
  const header = readHeader(input.headers, "accept-language");
  const value = header !== undefined
    ? resolveAcceptLanguage(locales, baseLocale, header)
    : resolveNavigatorLanguage(locales, baseLocale, input.navigator);
  return matchLocale(value);
}

function readHeader(headers: unknown, name: string) {
  if (!headers) return undefined;
  const getter = (headers as { get?: (header: string) => string | null | undefined }).get;
  if (typeof getter !== "function") return undefined;
  try {
    return getter.call(headers, name) ?? undefined;
  } catch {
    return undefined;
  }
}

function parseAcceptLanguage(header: string | null | undefined): LanguagePreference[] {
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

function resolveAcceptLanguage<Locale extends string>(
  locales: readonly Locale[], baseLocale: Locale, header: string | null | undefined,
): Locale | undefined {
  const preferences = parseAcceptLanguage(header);
  let best: { locale: Locale; quality: number; preferenceIndex: number; base: boolean; localeIndex: number } | undefined;
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

function resolveNavigatorLanguage<Locale extends string>(locales: readonly Locale[], baseLocale: Locale, value: unknown) {
  if (!value || (typeof value !== "object" && typeof value !== "function")) return undefined;
  const preferences: string[] = [];
  try {
    const languages = (value as { languages?: unknown }).languages;
    if (Array.isArray(languages)) for (const language of languages) if (typeof language === "string") preferences.push(language);
  } catch {}
  try {
    const language = (value as { language?: unknown }).language;
    if (typeof language === "string") preferences.push(language);
  } catch {}
  return preferences.length === 0 ? undefined : resolveAcceptLanguage(locales, baseLocale, preferences.join(","));
}

function languageRangeMatches(range: string, locale: string) {
  if (range === "*") return true;
  const normalizedRange = range.toLowerCase();
  const normalizedLocale = locale.toLowerCase();
  return normalizedRange === normalizedLocale || normalizedLocale.startsWith(`${normalizedRange}-`) || normalizedRange.startsWith(`${normalizedLocale}-`);
}
