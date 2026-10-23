import type { LinguiniWebEnvironment, LinguiniWebLocale, LinguiniWebOptions, LocaleSwitchPlan } from "./types.js";
import { normalizeBasePath } from "./url.js";
import { validateOptions, validateSources } from "./validation.js";

const DEFAULT_COOKIE_MAX_AGE = 60 * 60 * 24 * 365;

export function normalizeOptions(baseLocale: string, options: LinguiniWebOptions, environment: LinguiniWebEnvironment): LinguiniWebLocale<string>["options"] {
    const resolvers = options.locale?.sources ?? [];
    const sources = validateSources(resolvers);
    const cookie = {...options.cookie, ...resolvers.find(source => source.id === "cookie")?.cookie};
    const storage = {...options.localStorage, ...resolvers.find(source => source.id === "local-storage")?.localStorage};
    validateOptions(options, cookie, storage);
    const base = normalizeBasePath(environment.base ?? "");
    const origin = environment.origin;
    const cookiePath = cookie?.path === "auto" || cookie?.path === undefined
        ? base || "/"
        : cookie.path;
    const secure = cookie?.secure ?? "auto";
    return {
        routing: {
            localePrefix: options.routing?.localePrefix ?? "except-default",
            canonical: options.routing?.canonical ?? "redirect",
        },
        locale: {
            sources,
            switch: options.locale?.switch ?? localeSwitchFromSources(sources),
        },
        cookie: {
            name: cookie?.name ?? "LINGUINI_LOCALE",
            path: cookiePath,
            domain: cookie?.domain,
            maxAge: cookie?.maxAge ?? DEFAULT_COOKIE_MAX_AGE,
            sameSite: cookie?.sameSite ?? "lax",
            secure,
            httpOnly: Boolean(cookie?.httpOnly ?? false),
        },
        localStorage: { key: storage?.key ?? "LINGUINI_LOCALE" },
        links: { mode: options.links?.mode ?? "runtime" },
        routes: { exclude: options.routes?.exclude ?? [] },
        environment: { base, origin },
        baseLocale,
    };
}

function localeSwitchFromSources(sources: readonly string[]): LocaleSwitchPlan {
    return {
        writesPath: sources.includes("path"),
        writesCookie: sources.includes("cookie"),
        writesLocalStorage: sources.includes("local-storage"),
    };
}
