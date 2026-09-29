export interface LinguiniWebLocale<Locale extends string> {
  shouldLocalizeLink(href: string, attributes?: { download?: boolean; ignored?: boolean; rel?: string | null }, input?: Record<string, unknown>): boolean;
  localizeHref(href: string, locale: Locale, input?: Record<string, unknown>): string;
}
