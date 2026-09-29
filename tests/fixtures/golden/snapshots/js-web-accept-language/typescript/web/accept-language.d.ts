export declare function resolveAcceptLanguageLocale<Locale extends string>(
  input: Record<string, unknown>,
  locales: readonly Locale[],
  baseLocale: Locale,
  matchLocale: (value: unknown) => Locale | undefined,
): Locale | undefined;
