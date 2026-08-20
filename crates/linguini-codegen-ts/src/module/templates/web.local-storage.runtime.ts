export function resolveLocalStorageLocale<Locale extends string>(
  input: Record<string, unknown>,
  options: { localStorage: { key: string } },
  matchLocale: (value: unknown) => Locale | undefined,
): Locale | undefined {
  try {
    return matchLocale(
      (input.localStorage as Storage | undefined)?.getItem(options.localStorage.key) ?? undefined,
    );
  } catch {
    return undefined;
  }
}
