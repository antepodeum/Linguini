export function resolveCookieLocale<Locale extends string>(
  input: Record<string, unknown>,
  options: { cookieName: string },
  matchLocale: (value: unknown) => Locale | undefined,
): Locale | undefined {
  for (const cookie of String(input.cookie as string | undefined ?? "").split(";")) {
    const [rawName, ...rawValue] = cookie.trim().split("=");
    if (rawName !== options.cookieName) continue;
    try {
      return matchLocale(decodeURIComponent(rawValue.join("=")));
    } catch {
      return undefined;
    }
  }
  return undefined;
}
