# Web and SvelteKit

Web routing and SvelteKit hooks are reusable TypeScript runtime packages. Configure them in
application modules. Native codegen compiles messages, CLDR operations, types/maps, and small
project bindings; routing, request-language, and persistence options remain application code.

```sh
npm install @antepodeum/linguini-web @antepodeum/linguini-sveltekit
npm install --save-dev @antepodeum/linguini-vite
```

## Translation build

```toml
[project]
name = "app"
default_locale = "en"
locales = ["en", "ru", "ar"]

[paths]
schema = "schema"
locale = "locales"

[targets.ts]
out = "src/lib/generated/linguini"
declaration = true
gitignore = true

[targets.ts.bundler]
sources = ["src"]
locale_loading = "dynamic"
```

Use the SvelteKit build adapter, which composes the shared Vite/Svelte integration:

```ts
// vite.config.ts
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import linguini from "@antepodeum/linguini-sveltekit/vite";

export default defineConfig({
  plugins: [linguini({ buildOnStart: true }), sveltekit()],
});
```

The adapter prepares the native Svelte import/type bindings and keeps exact message selection,
source maps, watch, and HMR in the shared Vite core. Direct native builds use
`linguini build --svelte --svelte-host-module @antepodeum/linguini-sveltekit/client`.
Standalone Svelte installs `@antepodeum/linguini-svelte` and uses
`@antepodeum/linguini-vite/svelte`; framework-independent consumers use the main Vite entry.

With dynamic locale loading, client messages load before the active locale changes. SSR evaluates
messages synchronously using the request locale. Sparse translation fallback is folded per message
into the selected locale payload. Composed page delivery and browser-request verification are
tracked separately in the remediation checklist.

Finite dynamic message access requires an exact allow list. Unbounded access is diagnosed
instead of widening a page to the whole catalog:

```toml fragment=dynamic-policy
[targets.ts.bundler.dynamic]
mode = "bundle"
allow = ["home.title", "home.greeting"]
```

The default `mode = "error"` rejects computed accesses. Wildcards and namespace patterns are
not accepted in the allow list.

## Application runtime configuration

Share URL and persistence settings in an ordinary application module. Resolver functions are
referenced only by the server factory, so unused request-language and cookie-reading code can
be removed from the client graph.

```ts
// src/lib/linguini-web.ts
import { base } from "$app/paths";
import {
  createWebLocaleI18n, pathLocale, cookieLocale, requestLanguage,
} from "@antepodeum/linguini-web";
import type { LocaleResolver, LinguiniWebOptions } from "@antepodeum/linguini-web";
import * as locale from "$lib/generated/linguini/locale.js";

const policy = {
  routing: { localePrefix: "except-default", canonical: "redirect" },
  locale: { switch: { writesPath: true, writesCookie: true, writesLocalStorage: false } },
  cookie: { name: "APP_LOCALE", path: "auto", maxAge: 365 * 24 * 60 * 60 },
  links: { mode: "manual" },
  routes: { exclude: ["/_app/**", "/favicon.ico"] },
} satisfies LinguiniWebOptions;

function createWeb(sources: readonly LocaleResolver[] = []) {
  return createWebLocaleI18n(locale, {
    ...policy, locale: { ...policy.locale, sources },
  }, { base });
}
export const web = createWeb();
export function createRequestWeb() {
  return createWeb([pathLocale(), cookieLocale(), requestLanguage()]);
}
```

Resolvers run in configured order. An empty list selects the base locale. `pathLocale()` reads
its first localized path segment, `cookieLocale()` reads the configured cookie, and
`requestLanguage()` negotiates the request header. `localStorageLocale()` is available for
browser applications; SvelteKit rejects storage-only server resolution. Initial browser rendering
uses the serialized SSR locale instead of negotiating it again.

Locale matching uses the generated metadata's pinned CLDR canonicalization and parent lookup.
Unsupported values continue to the next resolver and finally to the base locale. Runtime adapters
parse actual URLs, headers, and cookies; they do not interpret CLDR formatting patterns.

Routing accepts `always`, `except-default`, or `never` locale prefixes and `redirect` or `preserve`
canonical handling. Cookie options include `name`, `path`, `domain`, positive `maxAge` in seconds,
`sameSite`, `secure`, and `httpOnly`. `auto` cookie paths follow SvelteKit's base. `SameSite=None`
requires a secure cookie. HttpOnly browser switching needs a server persistence callback or the
switch endpoint. Read sources and write transports are separate runtime choices.

```ts
// src/hooks.server.ts
import { createHandle } from "@antepodeum/linguini-sveltekit/server";
import { createRequestWeb } from "$lib/linguini-web";

export const handle = createHandle(createRequestWeb(), {
  switchRoute: { path: "/_linguini/locale/{locale}", returnQuery: "return", status: 303 },
});
```

```ts
// src/hooks.ts
import { createReroute } from "@antepodeum/linguini-sveltekit";
import { web } from "$lib/linguini-web";

export const reroute = createReroute(web);
```

```ts
// src/routes/+layout.server.ts
import { createLoad } from "@antepodeum/linguini-sveltekit/server";
import type { Locale } from "$lib/generated/linguini/locale.js";

export const load = createLoad<Locale>();
```

The hook stores request-local metadata in `locals.linguini`, handles canonical redirects and
optional switch endpoints, and replaces Linguini HTML placeholders while streaming. Split
placeholders retain only their unfinished suffix. Hooks do not import message catalogs.
The root load serializes locale metadata for matching SSR and hydration.

```ts
// src/app.d.ts
import type { LinguiniLocaleRequestContext } from "@antepodeum/linguini-web";
import type { SerializedLinguiniContext } from "@antepodeum/linguini-sveltekit";
import type { Locale } from "$lib/generated/linguini/locale.js";

declare global {
  namespace App {
    interface Error {}
    interface Locals { linguini?: LinguiniLocaleRequestContext<Locale> }
    interface PageData { linguini?: SerializedLinguiniContext<Locale> }
  }
}
export {};
```

```html
<!doctype html>
<html lang="%linguini.lang%" dir="%linguini.dir%">
  <head>%sveltekit.head%</head>
  <body data-sveltekit-preload-data="hover"><div style="display: contents">%sveltekit.body%</div></body>
</html>
```

SvelteKit owns the base path, request origin, and matched route's trailing-slash policy.
Linguini preserves the input slash shape; no generated policy rewrite is needed.

```ts
// src/routes/+layout.ts
export const trailingSlash = "never";
```

## Controls and message calls

```ts
// src/lib/linguini.ts
import { createControls } from "@antepodeum/linguini-sveltekit/client";
import * as state from "$lib/generated/linguini/svelte-locale.svelte.js";
import { web } from "./linguini-web";

export const linguini = createControls(web, state);
export const setLocale = linguini.setLocale;
export const localizeHref = linguini.localizeHref;
export const localizeUrl = linguini.localizeUrl;
export const shouldLocalizeHref = linguini.shouldLocalizeHref;
export const shouldLocalizeLink = linguini.shouldLocalizeLink;
export const localizeHrefAttribute = linguini.localizeHrefAttribute;
export const localizeLinkHref = linguini.localizeLinkHref;
export const delocalizeUrl = linguini.delocalizeUrl;
export const alternateLinks = linguini.alternateLinks;
```

```svelte
<script lang="ts">
  import { l } from "$lib/generated/linguini/svelte";
</script>

<h1>{l.home.title}</h1>
```

```svelte
<script lang="ts">
  import { l } from "$lib/generated/linguini/svelte";
  import { localizeHref } from "$lib/linguini";
</script>

<a href={localizeHref("/pricing")}>{l.nav.pricing}</a>
<a href={localizeHref("/account/settings")}>{l.nav.settings}</a>
```

```svelte
<script lang="ts">
  import { base } from "$app/paths";
</script>

<a href={`${base}/_linguini/locale/en?return=/account`}>English</a>
<a href={`${base}/_linguini/locale/ru?return=/account`}>Русский</a>
```

The optional endpoint validates the locale and same-origin return path, then applies the selected
path/cookie transport. It rejects external, protocol-relative, backslash, and encoded external
return targets. Without a return path it uses a safe Referer or the application base.

```svelte
<script lang="ts">
  import { l } from "$lib/generated/linguini/svelte";
  import { setLocale } from "$lib/linguini";
  import { locales } from "$lib/generated/linguini/locale";
</script>

<nav>
  {#each locales as locale}
    <button type="button" onclick={() => setLocale(locale)}>{locale}</button>
  {/each}
</nav>
<p>{l.home.subtitle}</p>
```

Controls preload registered active consumers before publishing a locale. Denied browser storage
or cookie writes do not break rendering. Navigation and history options pass through the SvelteKit
adapter. Loader disposal remains owned by the Vite/Svelte integration.

Use `linkModule: "src/lib/link-transform.ts"` in the Vite options to transform static anchors
through an application module exporting `localizeTransformedHref`. For browser-created anchors,
explicitly initialize `startRuntimeLinkLocalization` from `@antepodeum/linguini-web/runtime-links`
and dispose it when its owner is destroyed. These choices do not change native message generation.

```ts
// src/lib/documented-client-helpers.ts
import {
  alternateLinks, delocalizeUrl, linguini, localizeHref, localizeHrefAttribute,
  localizeUrl, localizeLinkHref, shouldLocalizeHref, shouldLocalizeLink,
} from "$lib/linguini";
```

For a link with attributes, `localizeLinkHref(href, locale, attributes)` checks download, ignore,
external-rel, URL, and exclusion policy using one URL/environment snapshot. Runtime anchor
localization uses this combined operation. Exclusion predicates run in order against an isolated
URL, so their mutations do not rewrite the eventual target. Request metadata snapshots direction
once per context; its direction fields agree, while another request evaluates its own direction.

The root HTML `lang` marker also supplies the server-selected locale while SvelteKit loads
hydration modules, before `page.data` is available. Keep the marker synchronized with the server
hook; client startup uses this explicit value rather than resolving cookies or language preferences.

## Server messages and hook composition

Server load functions can explicitly prepare and select generated messages. Request locale is
passed as a value; it is never stored in shared mutable server state.

```ts
import type { PageServerLoad } from "./$types";
import { prepareLinguini, createLinguini } from "$lib/generated/linguini/index.js";

export const load: PageServerLoad = async ({ locals }) => {
  const context = locals.linguini;
  if (!context) throw new Error("The locale hook is missing");
  await prepareLinguini(context.locale);
  const messages = createLinguini(context.locale);
  return { title: messages.home.title, locale: context.locale, canonical: context.localizeHref("/pricing") };
};
```

Compose hooks through SvelteKit's standard interfaces:

```ts
// src/hooks.server.ts
import { sequence } from "@sveltejs/kit/hooks";
import { createHandle } from "@antepodeum/linguini-sveltekit/server";
import { createRequestWeb } from "$lib/linguini-web";
import { appHandle } from "$lib/server/app-handle";

const linguiniHandle = createHandle(createRequestWeb(), {
  switchRoute: { path: "/_linguini/locale/{locale}" },
});
export const handle = sequence(linguiniHandle, appHandle);
```

```ts
// src/hooks.ts
import type { Reroute } from "@sveltejs/kit";
import { createReroute } from "@antepodeum/linguini-sveltekit";
import { web } from "$lib/linguini-web";
import { appReroute } from "$lib/app-reroute";

const linguiniReroute = createReroute(web);
export const reroute: Reroute = async event =>
  (await appReroute(event)) ?? (await linguiniReroute(event));
```

```ts
// src/routes/+layout.server.ts
import type { LayoutServerLoad } from "./$types";
import type { Locale } from "$lib/generated/linguini/locale.js";
import { createLoad } from "@antepodeum/linguini-sveltekit/server";
import { appLoad } from "$lib/server/app-load";

const linguiniLoad = createLoad<Locale>();
export const load: LayoutServerLoad = async event => ({
  ...(await appLoad(event)),
  ...linguiniLoad(event),
});
```
