import type { CookieOptions, StorageOptions, LinguiniWebOptions, LocaleResolver, LocaleSource } from './types.js';

export function validateSources(sources: readonly LocaleResolver[]): LocaleSource[] {
    const ids = new Set<LocaleSource>();
    for (const source of sources) {
        if (!source || typeof source.resolve !== 'function' || typeof source.server !== 'boolean'
            || !['path', 'cookie', 'local-storage', 'accept-language'].includes(source.id)) {
            throw new TypeError('Locale sources must be runtime resolver capabilities');
        }
        if (ids.has(source.id))
            throw new TypeError(`Duplicate locale source: ${source.id}`);
        ids.add(source.id);
    }
    return [...ids];
}

export function validateOptions(options: LinguiniWebOptions, cookie: CookieOptions, storage: StorageOptions): void {
    if (options.routing?.localePrefix !== undefined
        && !['always', 'except-default', 'never'].includes(options.routing.localePrefix)) {
        throw new TypeError('Invalid locale prefix policy');
    }
    if (options.routing?.canonical !== undefined && !['redirect', 'preserve'].includes(options.routing.canonical)) {
        throw new TypeError('Invalid canonical URL policy');
    }
    if (cookie) {
        if (cookie.name !== undefined && !/^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/.test(cookie.name)) {
            throw new TypeError('Invalid locale cookie name');
        }
        if (cookie.path !== undefined && cookie.path !== 'auto'
            && (!cookie.path.startsWith('/') || /[\x00-\x1f\x7f;]/.test(cookie.path))) {
            throw new TypeError('Locale cookie path must be absolute');
        }
        if (cookie.domain !== undefined && (!/^[0-9A-Za-z.-]+$/.test(cookie.domain)
            || cookie.domain.startsWith('.') || cookie.domain.endsWith('.'))) {
            throw new TypeError('Invalid locale cookie domain');
        }
        if (cookie.maxAge !== undefined && (!Number.isSafeInteger(cookie.maxAge) || cookie.maxAge <= 0)) {
            throw new TypeError('Locale cookie maxAge must be a positive integer in seconds');
        }
        if (cookie.sameSite !== undefined && !['lax', 'strict', 'none'].includes(cookie.sameSite)) {
            throw new TypeError('Invalid locale cookie SameSite policy');
        }
        if (cookie.secure !== undefined && cookie.secure !== 'auto' && typeof cookie.secure !== 'boolean') {
            throw new TypeError('Locale cookie secure must be boolean or auto');
        }
        if (cookie.sameSite === 'none' && cookie.secure !== true) {
            throw new TypeError('SameSite=None requires a secure locale cookie');
        }
    }
    if (storage?.key !== undefined && (!storage.key || /[\x00-\x1f\x7f-\x9f]/.test(storage.key))) {
        throw new TypeError('Invalid locale storage key');
    }
    if (options.links?.mode !== undefined && !['manual', 'runtime', 'transform'].includes(options.links.mode)) {
        throw new TypeError('Invalid link localization mode');
    }
    for (const pattern of options.routes?.exclude ?? []) {
        if (typeof pattern === 'string' && (!pattern.startsWith('/') || pattern.startsWith('//') || /[?#]/.test(pattern))) {
            throw new TypeError('Route exclusions must be absolute paths');
        }
    }
}
