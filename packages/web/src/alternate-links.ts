import type { AlternateLink, LinguiniWebLocale } from './types.js';
import { parseRuntimeUrl } from './url.js';

/** Build alternate-language URLs only where the application requests them. */
export function alternateLinks<Locale extends string>(
    web: LinguiniWebLocale<Locale>,
    url: string | URL,
    input: Record<string, unknown> = {},
): AlternateLink[] {
    const parsed = parseRuntimeUrl(url, web.options, input);
    const links: AlternateLink[] = web.locales.map(locale => ({
        rel: 'alternate',
        hreflang: locale,
        href: web.localizeUrl(parsed, locale, input).toString(),
    }));
    links.push({ rel: 'alternate', hreflang: 'x-default', href: web.localizeUrl(parsed, web.baseLocale, input).toString() });
    return links;
}
