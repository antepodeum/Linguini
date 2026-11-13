import type {PagePayload} from './types.js';

/** Live Kit responses put hydration preloads in HTTP headers rather than HTML. */
export function pagePreloadHeader(payload: PagePayload, existing: string | null, requestUrl: URL): string | undefined {
  if (!existing || !/<[^>]*\/immutable\/entry\/[^>]+>[^,]*\brel\s*=\s*(?:"modulepreload"|modulepreload)/i.test(existing)) return;
  const resolved = new URL(payload.url,requestUrl);
  const href = resolved.origin === requestUrl.origin
    ? resolved.pathname + resolved.search + resolved.hash
    : resolved.href;
  return `<${href}>; rel="modulepreload"; media="(scripting: enabled)"`;
}

/** Add the selected catalog beside Kit's own hydration preload, preserving streamed HTML. */
export function createPagePreload(payload: PagePayload) {
  const href = payload.url.replaceAll('&','&amp;').replaceAll('"','&quot;').replaceAll('<','&lt;').replaceAll('>','&gt;');
  const link = `<link rel="modulepreload" media="(scripting: enabled)" href="${href}">`;
  let pending = '';
  let inserted = false;
  return ({html,done}: {html:string;done:boolean}): string => {
    if (inserted) return html;
    const input = pending + html;
    pending = '';
    // CSR-disabled responses have no Kit hydration preloads and receive no locale preload.
    const links = /<link\b[^>]*>/gi;
    for (const match of input.matchAll(links)) {
      if (/\brel\s*=\s*(["'])modulepreload\1/i.test(match[0])
        && /\bhref\s*=\s*(["'])[^"']*\/immutable\/entry\/[^"']+\1/i.test(match[0])) {
        inserted = true;
        const end = match.index + match[0].length;
        return input.slice(0,end) + link + input.slice(end);
      }
    }
    if (!done) {
      const start = input.lastIndexOf('<');
      const tail = input.slice(start);
      if (start >= 0 && (/^<(?:l(?:i(?:n(?:k)?)?)?)?$/i.test(tail) || /^<link\b[^>]*$/i.test(tail))) {
        pending = tail;
        return input.slice(0,start);
      }
    }
    return input;
  };
}
