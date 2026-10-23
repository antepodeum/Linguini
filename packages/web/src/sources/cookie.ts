import type { CookieOptions, LocaleResolutionContext, LocaleResolver } from "../types.js";
export function resolveCookieLocale<Locale extends string>(input: Record<string, unknown>, options: {
    cookie: {
        name: string;
    };
}, matchLocale: (value: unknown) => Locale | undefined): Locale | undefined {
    for (const cookie of String(( input.cookie as string | undefined) ?? "").split(";")) {
        const [rawName, ...rawValue] = cookie.trim().split("=");
        if (rawName !== options.cookie.name)
            continue;
        try {
            return matchLocale(decodeURIComponent(rawValue.join("=")));
        }
        catch {
            return undefined;
        }
    }
    return undefined;
}

export function cookieLocale(options: CookieOptions = {}): LocaleResolver {

    function resolve<Locale extends string>(input: Record<string, unknown>, context: LocaleResolutionContext<Locale>): Locale | undefined {
        return resolveCookieLocale(input, context.options, context.matchLocale);
    }
    return Object.freeze({ id: "cookie", server: true, resolve, cookie: Object.freeze({ ...options }) });
}
