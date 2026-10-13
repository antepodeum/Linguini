//! Request-local metadata context forwarded by the lightweight server adapter.
use super::runtime_code::RuntimeCode;
pub(super) fn append(code: &mut RuntimeCode) {
    code.shared("\n");
    code.typed("function createRequestContext(\n  web: import(\"./web\").LinguiniWebLocale<Locale>,\n  resolved: Locale,\n  contextInput: Record<string, unknown>,\n) {", "/** @param {import(\"./web.js\").LinguiniWebLocale<import(\"./locale.js\").Locale>} web @param {import(\"./locale.js\").Locale} resolved @param {Record<string, unknown>} contextInput @returns {import(\"./web.js\").LinguiniLocaleRequestContext<import(\"./locale.js\").Locale>} */\nfunction createRequestContext(web, resolved, contextInput) {");
    code.shared("\n  const resolvedLocale = web.matchLocale(resolved) ?? web.baseLocale;\n  return {\n    locale: resolvedLocale,\n    baseLocale: web.baseLocale,\n    locales: web.locales,\n    direction: web.getTextDirection(resolvedLocale),\n    textDirection: web.getTextDirection(resolvedLocale),\n    lang: resolvedLocale,\n    htmlAttrs: web.htmlAttrs(resolvedLocale),\n    ");
    code.typed(
        "localizeHref: (href: string, nextLocale = resolvedLocale, input = contextInput) =>",
        "localizeHref: (href, nextLocale = resolvedLocale, input = contextInput) =>",
    );
    code.shared(" web.localizeHref(href, nextLocale, input),\n    ");
    code.typed(
        "localizeUrl: (url: string | URL, nextLocale = resolvedLocale, input = contextInput) =>",
        "localizeUrl: (url, nextLocale = resolvedLocale, input = contextInput) =>",
    );
    code.shared(" web.localizeUrl(url, nextLocale, input),\n    ");
    code.typed(
        "shouldLocalizeHref: (href: string, input = contextInput) =>",
        "shouldLocalizeHref: (href, input = contextInput) =>",
    );
    code.shared(" web.shouldLocalizeHref(href, input),\n    ");
    code.typed(
        "shouldLocalizeLink: (href: string, attributes = {}, input = contextInput) =>",
        "shouldLocalizeLink: (href, attributes = {}, input = contextInput) =>",
    );
    code.shared(" web.shouldLocalizeLink(href, attributes, input),\n    ");
    code.typed("localizeHrefAttribute: (href: string, nextLocale = resolvedLocale, input = contextInput) =>", "localizeHrefAttribute: (href, nextLocale = resolvedLocale, input = contextInput) =>");
    code.shared(" web.localizeHrefAttribute(href, nextLocale, input),\n    ");
    code.typed(
        "delocalizeUrl: (url: string | URL, input = contextInput) =>",
        "delocalizeUrl: (url, input = contextInput) =>",
    );
    code.shared(" web.delocalizeUrl(url, input),\n    ");
    code.typed(
        "alternateLinks: (url: string | URL, input = contextInput) =>",
        "alternateLinks: (url, input = contextInput) =>",
    );
    code.shared(" web.alternateLinks(url, input),\n  };\n}\n");
}
