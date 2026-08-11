export declare function resolvePathLocale<Locale extends string>(
  input: Record<string, unknown>,
  options: { basePath: string },
  locales: readonly Locale[],
): Locale | undefined;
