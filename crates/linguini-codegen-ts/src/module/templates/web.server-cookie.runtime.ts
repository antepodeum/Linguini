import type { LinguiniWebLocale } from "../web";

export function persistLocaleCookie<Locale extends string>(
  web: LinguiniWebLocale<Locale>,
  target: unknown,
  locale: Locale,
) {
  web.setLocaleCookie(target, locale);
}
