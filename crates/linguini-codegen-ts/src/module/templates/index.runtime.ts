export type LinguiniProviderOptions = {
  getLocale?: () => LinguiniLanguageInput;
  resolveLanguage?: () => LinguiniLanguageInput;
};

function localeFallbackTags(locale: string): string[] {
  const tags: string[] = [];
  let tag = locale;
  while (tag) {
    tags.push(tag);
    const dash = tag.lastIndexOf("-");
    if (dash <= 0) break;
    tag = tag.slice(0, dash);
  }
  return tags;
}

function isLanguageScriptTag(locale: string): boolean {
  const parts = locale.split("-");
  return parts.length === 2
    && /^[A-Za-z]{2,8}$/.test(parts[0])
    && /^[A-Za-z]{4}$/.test(parts[1]);
}

export function createLinguini(language: LinguiniLanguageInput): Linguini {
  const locale = normalizeLocale(language) ?? baseLocale;
  return localeModules[locale];
}

export function createLinguiniProvider(options: LinguiniProviderOptions = {}): Linguini {
  const resolve = options.getLocale ?? options.resolveLanguage ?? (() => baseLocale);
  return new Proxy({} as Linguini, {
    get(_target, property) {
      return createLinguini(resolve())[property as keyof Linguini];
    },
  });
}

export function configureLinguini(options: {
  language: LinguiniLanguageInput | (() => LinguiniLanguageInput);
}): Linguini {
  if (typeof options.language === "function") {
    return createLinguiniProvider({ resolveLanguage: options.language });
  }
  return createLinguini(options.language);
}

export const lgl: Linguini = createLinguini(baseLocale);

export function isLocale(locale: unknown): locale is Locale {
  return normalizeLocale(locale) !== undefined;
}

export function normalizeLocale(locale: unknown): Locale | undefined {
  if (typeof locale !== "string") return undefined;
  for (const tag of localeFallbackTags(locale)) {
    const exact = locales.find((entry) => entry.toLowerCase() === tag.toLowerCase());
    if (exact) return exact;
    const key = tag.toLowerCase();
    if (Object.prototype.hasOwnProperty.call(localeResolutionOverrides, key)) {
      return localeResolutionOverrides[key] ?? undefined;
    }
    if (isLanguageScriptTag(tag)) return undefined;
  }
  return undefined;
}

export function getTextDirection(locale: Locale): TextDirection {
  return localeDirections[normalizeLocale(locale) ?? baseLocale];
}
