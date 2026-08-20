export declare function resolveCookieLocale<Locale extends string>(
  input: Record<string, unknown>,
  options: { cookie: { name: string } },
  matchLocale: (value: unknown) => Locale | undefined,
): Locale | undefined;
