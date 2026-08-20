export function matchesRoute(
  pattern: string | RegExp | ((url: URL) => boolean),
  url: URL,
) {
  if (typeof pattern === "function") return Boolean(pattern(url));
  if (pattern instanceof RegExp) {
    if (!pattern.global && !pattern.sticky) return pattern.test(url.pathname);
    pattern.lastIndex = 0;
    try {
      return pattern.test(url.pathname);
    } finally {
      pattern.lastIndex = 0;
    }
  }
  if (pattern.endsWith("/**")) {
    const prefix = pattern.slice(0, -3).replace(/\/+$/, "");
    return !prefix
      || prefix === "/"
      || url.pathname === prefix
      || url.pathname.startsWith(`${prefix}/`);
  }
  return url.pathname === pattern;
}
