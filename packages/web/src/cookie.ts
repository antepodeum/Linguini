import type { LinguiniWebLocale } from './types.js';
import { resolveCookieSecure } from './url.js';

/** Serialize normalized application policy without depending on request or message adapters. */
export function serializeLocaleCookie<Locale extends string>(web: LinguiniWebLocale<Locale>, locale: NoInfer<Locale>, input: Record<string, unknown> = {}): string {
    const cookie = web.options.cookie;
    const parts = [`${cookie.name}=${encodeURIComponent(locale)}`];
    parts.push(`Max-Age=${input.maxAge ?? cookie.maxAge}`);
    parts.push(`Path=${cookie.path}`);
    if (cookie.domain)
        parts.push(`Domain=${cookie.domain}`);
    parts.push(`SameSite=${cookie.sameSite}`);
    if (resolveCookieSecure(cookie.secure, input, web.options.environment.origin))
        parts.push('Secure');
    if (input.httpOnly ?? cookie.httpOnly)
        parts.push('HttpOnly');
    return parts.join('; ');
}
