import type { LinguiniWebLocale, LinkLocalizationAttributes } from "../web";

export declare function localizeTransformedHref<Locale extends string>(
  web: LinguiniWebLocale<Locale>,
  getLocale: () => Locale,
  href: string,
  attributes?: LinkLocalizationAttributes,
  input?: Record<string, unknown>,
): string;
