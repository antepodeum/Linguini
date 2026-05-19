import { createLinguiniRune } from "@antepod/linguini-sveltekit/client";
import * as runtime from "./index";

export const linguini = createLinguiniRune(runtime, { strategy: ["url", "cookie", "localStorage", "preferredLanguage", "baseLocale"] as const, cookieName: "LINGUINI_SITE_LOCALE", cookiePath: "/", cookieMaxAge: 31536000, cookieSameSite: "lax", cookieSecure: false, cookieHttpOnly: false, localStorageKey: "LINGUINI_SITE_LOCALE", prefixDefaultLocale: true, basePath: "", trailingSlash: "always", redirect: true, exclude: ["/_app/**", "/favicon.ico", "/robots.txt"] as const, localizeLinks: true } as const);
export const l = linguini.l;
export const messages = linguini.messages;
export const setLocale = linguini.setLocale;
export const localizeHref = linguini.localizeHref;
export const localizeUrl = linguini.localizeUrl;
export const delocalizeUrl = linguini.delocalizeUrl;
export const alternateLinks = linguini.alternateLinks;
