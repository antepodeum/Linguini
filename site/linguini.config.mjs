// linguini.config.mjs
// @ts-check
import { defineModules } from "../packages/project/src/modules.js";
import { web, svelteKit } from "../packages/project/src/ecmascript/modules.js";

export default defineModules([
  web({
    routing: {
      localePrefix: "always",
      canonical: "redirect"
    },
    locale: {
      sources: [
        "path",
        "cookie",
        "local-storage",
        "accept-language"
      ]
    },
    cookie: {
      name: "LINGUINI_SITE_LOCALE",
      path: "/",
      maxAge: "365d",
      sameSite: "lax",
      secure: false,
      httpOnly: false
    },
    localStorage: {
      key: "LINGUINI_SITE_LOCALE"
    },
    links: {
      mode: "runtime"
    },
    routes: {
      exclude: [
        "/_app/**",
        "/favicon.ico",
        "/robots.txt"
      ]
    }
  }),
  svelteKit()
]);
