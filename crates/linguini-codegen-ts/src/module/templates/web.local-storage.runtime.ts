export function resolveLocalStorageLocale<Locale extends string>(
  input: Record<string, unknown>,
  options: { localStorageKey: string },
  matchLocale: (value: unknown) => Locale | undefined,
): Locale | undefined {
  try {
    return matchLocale(
      (input.localStorage as Storage | undefined)?.getItem(options.localStorageKey) ?? undefined,
    );
  } catch {
    return undefined;
  }
}
