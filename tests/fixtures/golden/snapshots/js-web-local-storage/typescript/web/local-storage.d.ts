export declare function resolveLocalStorageLocale<Locale extends string>(
  input: Record<string, unknown>,
  options: { localStorage: { key: string } },
  matchLocale: (value: unknown) => Locale | undefined,
): Locale | undefined;
