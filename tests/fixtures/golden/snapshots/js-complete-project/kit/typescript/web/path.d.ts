export declare function resolvePathLocale<Locale extends string>(
  input: Record<string, unknown>,
  options: { environment: { base: string } },
  locales: readonly Locale[],
): Locale | undefined;
