import { createHandle, createLoad, createReroute } from "@antepod/linguini-sveltekit/server";
import * as runtime from "./index";

const options = { strategy: ["url", "cookie", "localStorage", "preferredLanguage", "baseLocale"] as const, cookieName: "LINGUINI_SITE_LOCALE", cookiePath: "/", cookieMaxAge: 31536000, cookieSameSite: "lax", cookieSecure: false, cookieHttpOnly: false, localStorageKey: "LINGUINI_SITE_LOCALE", prefixDefaultLocale: true, basePath: "", trailingSlash: "always", redirect: true, exclude: ["/_app/**", "/favicon.ico", "/robots.txt"] as const, localizeLinks: true } as const;

export const handle = createHandle(runtime, options);
export const reroute = createReroute(runtime, options);
export const load = createLoad();
