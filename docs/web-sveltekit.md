# Web and SvelteKit setup

Install the Vite plugin.

```sh
npm install @antepod/linguini-vite
```

## Linguini config

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
framework = "sveltekit"

[targets.ts.bundler]
sources = ["src"]
# locale_loading = "dynamic" # eager is the default
```

The bundler target enables exact per-message module imports for the configured
source roots. Components that import `l` from the generated Svelte facade are
rewritten to the message modules they statically read; the generated message
barrel is not needed in that browser graph. Parameterless leaves are values
(`l.home.title`), while parameterized leaves remain callable
(`l.home.greeting("Artemy")`).

Dynamic access is strict by default. A finite, explicit escape can be enabled
with canonical message paths:

```toml
[targets.ts.bundler.dynamic]
mode = "bundle"
allow = ["home.title", "home.greeting"]
```

`allow` is an exact list of canonical message leaves; wildcards and namespace
patterns are not accepted. `mode = "error"` is the default and rejects
computed or otherwise dynamic message access during the bundler build.

With `locale_loading = "dynamic"`, client virtual message modules import locale
chunks on demand and preload the selected locale before a locale switch. SSR
uses synchronous static locale imports; omit the field for eager loading.

Use `framework = "svelte"` for a Svelte-only app. Omit `framework` to generate
only the framework-agnostic TypeScript runtime.

## Web config

```toml
[web.routing]
locale_prefix = "always" # "always", "except-default", or "never"
canonical = "redirect"   # or "preserve"

# Sources are checked left-to-right; the first supported locale wins.
[web.locale]
sources = ["path", "cookie", "local-storage", "accept-language"]

[web.cookie]
name = "LINGUINI_LOCALE"
path = "/"
max_age = "365d"
same_site = "lax" # "lax", "strict", or "none"
secure = false     # "auto" is also supported
http_only = false

[web.local_storage]
key = "LINGUINI_LOCALE"

[web.links]
mode = "runtime" # "transform", "runtime", or "manual"

[web.routes]
exclude = ["/_app/**", "/favicon.ico"]
```

Linguini does not define a second trailing-slash policy. SvelteKit owns it per
route through the `trailingSlash` page option. Set an application-wide default
in the root layout and override it in child routes when needed:

```ts
// src/routes/+layout.ts
export const trailingSlash = "never"; // "never", "always", or "ignore"
```

Available locale sources:

| Source              | Reads from                                                  |
| ------------------- | ----------------------------------------------------------- |
| `path`              | the first localized path segment, for example `/ru/pricing` |
| `cookie`            | the configured locale cookie                                |
| `local-storage`     | the configured browser storage key                          |
| `accept-language`   | the server `Accept-Language` header                         |

## SvelteKit files

Add the Vite plugin so generated files update during development:

```ts
// vite.config.ts
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import linguini from "@antepod/linguini-vite";

export default defineConfig(({ command }) => ({
  plugins: [
    linguini({ buildOnStart: command === "serve" }),
    sveltekit(),
  ],
}));
```

Build the generated runtime before a production build. Keep the Vite plugin in
the production config so it can transform the generated message imports, but
disable its startup code generation when the build is driven explicitly:

```sh
linguini build
vite build
```

Use the generated hooks and root layout load:

```ts
// src/hooks.server.ts
export { handle } from "$lib/generated/linguini/sveltekit-control";
```

```ts
// src/hooks.ts
export { reroute } from "$lib/generated/linguini/sveltekit-control";
```

```ts
// src/routes/+layout.server.ts
export { load } from "$lib/generated/linguini/sveltekit-control";
```

These lightweight hooks import locale and web state without the eager message
barrel. Use the full generated `sveltekit` module instead when server code
needs `locals.linguini` message access or the typed server context.

The short `handle`, `reroute`, and `load` exports are for applications that do
not already define those hooks. For composition, import the collision-free
`linguiniHandle`, `linguiniReroute`, and `linguiniLoad` exports instead.
SvelteKit's `sequence` helper composes server handles; put Linguini first when a
later handle reads `event.locals.linguini`:

```ts
// src/hooks.server.ts
import { sequence } from "@sveltejs/kit/hooks";
import { linguiniHandle } from "$lib/generated/linguini/sveltekit";
import { appHandle } from "$lib/server/app-handle";

export const handle = sequence(linguiniHandle, appHandle);
```

SvelteKit accepts one `reroute` hook. Give the application hook explicit
priority and use Linguini when it does not reroute the URL:

```ts
// src/hooks.ts
import type { Reroute } from "@sveltejs/kit";
import { linguiniReroute } from "$lib/generated/linguini/sveltekit";
import { appReroute } from "$lib/app-reroute";

export const reroute: Reroute = async (event) =>
  (await appReroute(event)) ?? (await linguiniReroute(event));
```

Compose root server-load results explicitly. Spread the Linguini result last so
the reserved `data.linguini` field cannot be replaced accidentally:

```ts
// src/routes/+layout.server.ts
import type { LayoutServerLoad } from "./$types";
import { linguiniLoad } from "$lib/generated/linguini/sveltekit";
import { appLoad } from "$lib/server/app-load";

export const load: LayoutServerLoad = async (event) => ({
  ...(await appLoad(event)),
  ...(await linguiniLoad(event)),
});
```

Use the generated HTML placeholders:

```html
<!-- src/app.html -->
<html lang="%linguini.lang%" dir="%linguini.dir%">
  <body data-sveltekit-preload-data="hover">
    <div style="display: contents">%sveltekit.body%</div>
  </body>
</html>
```

The generated `sveltekit.d.ts` augments `App.Locals` and `App.PageData`, so
server loads can use the request-scoped locale context without handwritten app
ambient declarations. Linguini owns only `locals.linguini`; it does not reserve
generic application fields such as `locals.locale`, `locals.direction`, or
`locals.l`.

The generated declaration file is a module that imports local generated types
at the top level and augments SvelteKit's global `App` namespace inside
`declare global`. If your app already has `src/app.d.ts`, keep your own
declarations there and include the generated declaration through TypeScript's
normal `include` paths; do not copy generated imports into `namespace App`.

```ts
// src/app.d.ts
declare global {
  namespace App {
    interface Error {
      message: string;
    }
  }
}

export {};
```

## Svelte usage

Components usually import only `l`:

```svelte
<script lang="ts">
  import { l } from "$lib/generated/linguini/svelte";
</script>

<h1>{l.home.title}</h1>
```

The browser helper localizes internal links after hydration and watches links
created later. For localized SSR output, render the generated helper result
directly:

```svelte
<script lang="ts">
  import { l, localizeHref } from "$lib/generated/linguini/svelte";
</script>

<a href={localizeHref("/pricing")}>{l.nav.pricing}</a>
<a href={localizeHref("/account/settings")}>{l.nav.settings}</a>
```

For the `ru` locale these render as `/ru/pricing` and `/ru/account/settings`.
The generated SvelteKit response hook replaces only the Linguini HTML
placeholders in each response chunk; it does not parse or buffer arbitrary HTML.
External links, `mailto:`/`tel:` links, hash-only links, `download` links,
`rel="external"` links, excluded routes, and links marked with
`data-linguini-ignore` or `data-linguini-no-localize` are left unchanged by the
browser observer.

Use the generated helpers for locale switching or programmatic URLs:

```svelte
<script lang="ts">
  import { l } from "$lib/generated/linguini/svelte";
  import { setLocale } from "$lib/generated/linguini/svelte-control";
  import { locales } from "$lib/generated/linguini/locale";
</script>

<nav>
  {#each locales as locale}
    <button type="button" onclick={() => setLocale(locale)}>{locale}</button>
  {/each}
</nav>

<p>{l.home.subtitle}</p>
```

Other generated client helpers:

```ts
import {
  alternateLinks,
  delocalizeUrl,
  linguini,
  localizeHref,
  localizeHrefAttribute,
  localizeUrl,
  shouldLocalizeHref,
  shouldLocalizeLink,
} from "$lib/generated/linguini/svelte";
```

## Server usage

`handle` stores a request-scoped context in `event.locals`:

```ts
export function load({ locals }) {
  const { linguini } = locals;
  return {
    title: linguini.l.home.title,
    locale: linguini.locale,
    canonical: linguini.localizeHref("/pricing"),
  };
}
```
