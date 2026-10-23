export interface LocaleSwitchRouteOptions {
  path?: string;
  returnQuery?: string;
  status?: 301 | 302 | 303 | 307 | 308;
}

export function persistLocaleCookie<Locale extends string>(web: import('./index.js').LinguiniWebLocale<Locale>, target: unknown, locale: Locale, input: Record<string, unknown> = {}) {
    web.setLocaleCookie(target, locale, input);
}

export function createLocaleSwitchHandler<Locale extends string>(web: import('./index.js').LinguiniWebLocale<Locale>, options: import('./server.js').LocaleSwitchRouteOptions = {}): (event: {
    url: URL;
    request: Request;
}) => Response | undefined {
    const path = options.path ?? '/_linguini/locale/{locale}';
    const marker = '{locale}';
    const position = path.indexOf(marker);
    const returnQuery = options.returnQuery ?? 'return';
    const status = options.status ?? 303;
    if (!path.startsWith('/') || path.startsWith('//') || position < 0
        || path.indexOf(marker, position + marker.length) !== -1 || /[?#]/.test(path)
        || path.split('/').includes('..') || !/^[A-Za-z0-9_-]+$/.test(returnQuery)
        || ![301, 302, 303, 307, 308].includes(status)) {
        throw new TypeError('Invalid locale switch route');
    }
    if (!web.options.locale.switch.writesPath && !web.options.locale.switch.writesCookie) {
        throw new TypeError('A locale switch endpoint requires path or cookie persistence');
    }
    const prefix = web.options.environment.base + path.slice(0, position);
    const suffix = path.slice(position + marker.length);
    return function handleLocaleSwitch({ url, request }) {
        if (!url.pathname.startsWith(prefix) || (suffix && !url.pathname.endsWith(suffix)))
            return undefined;
        const encoded = url.pathname.slice(prefix.length, suffix ? -suffix.length : undefined);
        if (!encoded || encoded.includes('/'))
            return undefined;
        let locale;
        try {
            locale = web.matchLocale(decodeURIComponent(encoded));
        }
        catch {
            return undefined;
        }
        if (!locale)
            return undefined;
        const target = sameOriginPath(url.searchParams.get(returnQuery), url.origin)
            ?? sameOriginPath(request.headers.get('referer'), url.origin)
            ?? (web.options.environment.base || '/');
        const input = { url, currentUrl: url, origin: url.origin, headers: request.headers };
        const location = web.options.locale.switch.writesPath ? web.localizeHref(target, locale, input) : target;
        const response = new Response(null, { status, headers: { location } });
        if (web.options.locale.switch.writesCookie)
            persistLocaleCookie(web, response, locale, { origin: url.origin });
        return response;
    };
}

function sameOriginPath(value: string | null, origin: string): string | undefined {
    if (!value)
        return undefined;
    let decoded = value;
    for (let depth = 0; depth < 3; depth += 1) {
        if (decoded.includes('\\') || decoded.startsWith('//'))
            return undefined;
        try {
            const next = decodeURIComponent(decoded);
            if (next === decoded)
                break;
            decoded = next;
        }
        catch {
            return undefined;
        }
    }
    try {
        const parsed = new URL(value, origin);
        return parsed.origin === origin ? `${parsed.pathname}${parsed.search}${parsed.hash}` : undefined;
    }
    catch {
        return undefined;
    }
}
