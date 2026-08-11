export function resolvePathLocale<Locale extends string>(
  input: Record<string, unknown>,
  options: { basePath: string },
  locales: readonly Locale[],
): Locale | undefined {
  const value = input.url as URL | string | undefined;
  if (!value) return undefined;
  let parsed: URL;
  try {
    parsed = new URL(String(value), "http://localhost");
  } catch {
    return undefined;
  }
  const pathname = parsed.pathname.startsWith("/") ? parsed.pathname : `/${parsed.pathname}`;
  const base = options.basePath && options.basePath !== "/"
    ? `/${options.basePath.replace(/^\/+|\/+$/g, "")}`
    : "";
  const stripped = base && pathname === base
    ? "/"
    : base && pathname.startsWith(`${base}/`)
      ? pathname.slice(base.length)
      : pathname;
  const segment = stripped.split("/").filter(Boolean)[0];
  return typeof segment === "string"
    ? locales.find((locale) => locale.toLowerCase() === segment.toLowerCase())
    : undefined;
}
