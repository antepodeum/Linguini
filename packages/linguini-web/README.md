# @antepod/linguini-web

Framework-agnostic locale resolution and localized URL helpers for generated
Linguini TypeScript runtimes.

```ts
import { createWebI18n } from "@antepod/linguini-web";
import * as runtime from "./generated/linguini";

const i18n = createWebI18n(runtime, {
  strategy: ["url", "cookie", "header", "baseLocale"],
  cookieName: "SHOP_LOCALE",
  localStorageKey: "SHOP_LOCALE",
  prefixDefaultLocale: false,
  basePath: "/shop",
  trailingSlash: "never",
  exclude: ["/api/**"],
});

const context = await i18n.resolveRequest(request);
const title = context.l.home.title();
const ruPricing = context.localizeHref("/pricing", "ru");
const html = context.localizeMarkupLinks('<a href="/pricing">Pricing</a>');
const htmlAttrs = context.htmlAttrs;
```

Use `@antepod/linguini-sveltekit` for SvelteKit projects; it builds on this
package and provides generated hooks plus Svelte 5 rune-backed client state.
