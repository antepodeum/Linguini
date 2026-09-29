import type { LinguiniWebLocale } from "../web";
{{SERVER_COOKIE_IMPORT}}

const routePath = "{{SWITCH_ROUTE_PATH}}";
const returnQuery = "{{SWITCH_ROUTE_RETURN_QUERY}}";
const redirectStatus = {{SWITCH_ROUTE_STATUS}};

export function handleLocaleSwitchRoute<Locale extends string>(
  web: LinguiniWebLocale<Locale>,
  event: { url: URL; request: Request },
): Response | undefined {
  const locale = matchSwitchLocale(web, event.url.pathname);
  if (!locale) return undefined;
  const input = {
    url: event.url,
    currentUrl: event.url,
    origin: event.url.origin,
    headers: event.request.headers,
  };
  const returnTarget = (safeReturnTarget(event) ?? web.options.environment.base) || "/";
  const location = web.options.locale.switch.writesPath
    ? web.localizeHref(returnTarget, locale, input)
    : returnTarget;
  const response = new Response(null, {
    status: redirectStatus,
    headers: { location },
  });
{{PERSIST_SWITCH_COOKIE}}
  return response;
}

function matchSwitchLocale<Locale extends string>(
  web: LinguiniWebLocale<Locale>,
  pathname: string,
) {
  const effectiveRoutePath = `${web.options.environment.base}${routePath}` || routePath;
  const marker = "{locale}";
  const markerIndex = effectiveRoutePath.indexOf(marker);
  const prefix = effectiveRoutePath.slice(0, markerIndex);
  const suffix = effectiveRoutePath.slice(markerIndex + marker.length);
  if (!pathname.startsWith(prefix) || !pathname.endsWith(suffix)) return undefined;
  const encoded = pathname.slice(
    prefix.length,
    suffix.length > 0 ? pathname.length - suffix.length : pathname.length,
  );
  if (!encoded || encoded.includes("/")) return undefined;
  try {
    return web.matchLocale(decodeURIComponent(encoded));
  } catch {
    return undefined;
  }
}

function safeReturnTarget(event: { url: URL; request: Request }) {
  const queryTarget = event.url.searchParams.get(returnQuery);
  const query = queryTarget ? sameOriginPath(queryTarget, event.url.origin) : undefined;
  if (query) return query;
  const referer = event.request.headers.get("referer");
  return referer ? sameOriginPath(referer, event.url.origin) : undefined;
}

function sameOriginPath(value: string, origin: string) {
  if (!value) return undefined;
  let decoded = value;
  for (let depth = 0; depth < 3; depth += 1) {
    if (decoded.includes("\\") || decoded.startsWith("//")) return undefined;
    try {
      const next = decodeURIComponent(decoded);
      if (next === decoded) break;
      decoded = next;
    } catch {
      return undefined;
    }
  }
  try {
    const parsed = new URL(value, origin);
    if (parsed.origin !== origin) return undefined;
    return `${parsed.pathname}${parsed.search}${parsed.hash}`;
  } catch {
    return undefined;
  }
}
