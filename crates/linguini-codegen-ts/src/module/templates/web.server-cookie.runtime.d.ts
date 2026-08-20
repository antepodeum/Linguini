import type { LinguiniWebLocale } from "../web";

export declare function persistLocaleCookie<Locale extends string>(
  web: LinguiniWebLocale<Locale>,
  target: unknown,
  locale: Locale,
): void;
