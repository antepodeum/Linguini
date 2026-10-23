import type { LocaleResolutionContext, LocaleResolver } from "../types.js";
type LanguagePreference = {
    range: string;
    quality: number;
    index: number;
    specificity: number;
};

export function resolveAcceptLanguageLocale<Locale extends string>(input: Record<string, unknown>, locales: readonly Locale[], baseLocale: Locale, matchLocale: (value: unknown) => Locale | undefined): Locale | undefined {
    const header = readHeader(input.headers, "accept-language");
    const value = header !== undefined
        ? resolveAcceptLanguage(locales, baseLocale, header)
        : resolveNavigatorLanguage(locales, baseLocale, input.navigator);
    return matchLocale(value);
}

function readHeader(headers: unknown, name: string): string | undefined {
    if (!headers)
        return undefined;
    const getter = ( headers as {
        get?: (header: string) => string | null | undefined;
    }).get;
    if (typeof getter !== "function")
        return undefined;
    try {
        return getter.call(headers, name) ?? undefined;
    }
    catch {
        return undefined;
    }
}

function parseAcceptLanguage(header: string | null | undefined): LanguagePreference[] {
    if (!header)
        return [];
    return String(header).split(",").flatMap((part, index) => {
        const [rawRange, ...parameters] = part.split(";");
        const range = rawRange.trim().toLowerCase();
        if (!/^(?:\*|[A-Za-z]{1,8}(?:-[A-Za-z0-9]{1,8})*)$/.test(range))
            return [];
        let quality = 1;
        for (const parameter of parameters) {
            const [rawName, ...rawValue] = parameter.split("=");
            if (rawName.trim().toLowerCase() !== "q")
                continue;
            const value = rawValue.join("=").trim();
            if (!/^(?:0(?:\.\d{0,3})?|1(?:\.0{0,3})?)$/.test(value))
                return [];
            quality = Number(value);
            break;
        }
        return [{ range, quality, index, specificity: range === "*" ? 0 : range.split("-").length }];
    });
}

function resolveAcceptLanguage<Locale extends string>(locales: readonly Locale[], baseLocale: Locale, header: string | null | undefined): Locale | undefined {
    const preferences = parseAcceptLanguage(header);
    const normalizedBase = baseLocale.toLowerCase();
     let bestLocale: Locale | undefined;
    let bestQuality = -1;
    let bestPreferenceIndex = Infinity;
    let bestIsBase = false;
    for (const locale of locales) {
        const normalizedLocale = locale.toLowerCase();
         let preference: LanguagePreference | undefined;
        for (const candidate of preferences) {
            // Input order breaks equal-specificity ties; a more specific q=0 still excludes.
            if ((!preference || candidate.specificity > preference.specificity) &&
                languageRangeMatches(candidate.range, normalizedLocale))
                preference = candidate;
        }
        if (!preference || preference.quality <= 0)
            continue;
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

function resolveNavigatorLanguage<Locale extends string>(locales: readonly Locale[], baseLocale: Locale, value: unknown): Locale | undefined {
    if (!value || (typeof value !== "object" && typeof value !== "function"))
        return undefined;
     const preferences: string[] = [];
    try {
        const languages = ( value as {
            languages?: unknown;
        }).languages;
        if (Array.isArray(languages))
            for (const language of languages)
                if (typeof language === "string")
                    preferences.push(language);
    }
    catch { }
    try {
        const language = ( value as {
            language?: unknown;
        }).language;
        if (typeof language === "string")
            preferences.push(language);
    }
    catch { }
    return preferences.length === 0 ? undefined : resolveAcceptLanguage(locales, baseLocale, preferences.join(","));
}

function languageRangeMatches(range: string, normalizedLocale: string): boolean {
    if (range === "*")
        return true;
    return range === normalizedLocale || normalizedLocale.startsWith(`${range}-`) || range.startsWith(`${normalizedLocale}-`);
}

export function requestLanguage(): LocaleResolver {

    function resolve<Locale extends string>(input: Record<string, unknown>, context: LocaleResolutionContext<Locale>): Locale | undefined {
        return resolveAcceptLanguageLocale(input, context.locales, context.baseLocale, context.matchLocale);
    }
    return Object.freeze({ id: "accept-language", server: true, resolve });
}
