import type { LinguiniWebLocale } from "../web";

export function persistLocaleCookie<Locale extends string>(
  web: LinguiniWebLocale<Locale>,
  target: unknown,
  locale: Locale,
  input: Record<string, unknown> = {},
) {
  web.setLocaleCookie(target, locale, input);
}
