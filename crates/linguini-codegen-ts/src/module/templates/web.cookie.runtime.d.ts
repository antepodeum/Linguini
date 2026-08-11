export declare function resolveCookieLocale<Locale extends string>(
  input: Record<string, unknown>,
  options: { cookieName: string },
  matchLocale: (value: unknown) => Locale | undefined,
): Locale | undefined;
