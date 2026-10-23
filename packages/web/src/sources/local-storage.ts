import type { LocaleResolutionContext, LocaleResolver, StorageOptions } from "../types.js";
export function resolveLocalStorageLocale<Locale extends string>(input: Record<string, unknown>, options: {
    localStorage: {
        key: string;
    };
}, matchLocale: (value: unknown) => Locale | undefined): Locale | undefined {
    try {
        return matchLocale(( input.localStorage as Storage | undefined)?.getItem(options.localStorage.key) ?? undefined);
    }
    catch {
        return undefined;
    }
}

export function localStorageLocale(options: StorageOptions = {}): LocaleResolver {

    function resolve<Locale extends string>(input: Record<string, unknown>, context: LocaleResolutionContext<Locale>): Locale | undefined {
        return resolveLocalStorageLocale(input, context.options, context.matchLocale);
    }
    return Object.freeze({ id: "local-storage", server: false, resolve, localStorage: Object.freeze({ ...options }) });
}
