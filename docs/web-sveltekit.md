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
module = "esm"
declaration = true
gitignore = true
framework = "sveltekit"
```

Use `framework = "svelte"` for a Svelte-only app. Omit `framework` to generate
only the framework-agnostic TypeScript runtime.

## Web config

```toml
[web]
# Locale sources are checked left-to-right. The first supported locale wins.
strategy = ["url", "cookie", "localStorage", "header", "baseLocale"]

# URL routing and localized URL generation.
base_path = ""
prefix_default_locale = false
trailing_slash = "ignore" # "ignore", "always", "never", or "directory"
redirect = true
origin = "https://example.com"
exclude = ["/api/**", "/_app/**", "/favicon.ico"]

# The browser helper auto-localizes internal <a href="..."> links by default.
# Use data-linguini-ignore on a single link or localize_links = false globally
# to keep hrefs unchanged.
localize_links = true

# Cookie persistence.
cookie_name = "LINGUINI_LOCALE"
cookie_path = "/"
cookie_max_age = 31536000
cookie_same_site = "lax" # "lax", "strict", or "none"
cookie_secure = false
cookie_http_only = false
# cookie_domain = "example.com"

# Browser storage persistence.
local_storage_key = "LINGUINI_LOCALE"

# Existing-app escape hatch for strategy = ["globalVariable", ...].
# global_variable_name = "__LINGUINI_LOCALE__"
```

Available strategies:

| Strategy            | Reads from                                                  |
| ------------------- | ----------------------------------------------------------- |
| `url`               | the first localized path segment, for example `/ru/pricing` |
| `cookie`            | the configured locale cookie                                |
| `localStorage`      | the configured browser storage key                          |
| `header`            | the server `Accept-Language` header                         |
| `navigator`         | browser `navigator.languages` / `navigator.language`        |
| `preferredLanguage` | `header` on the server, `navigator` in the browser          |
| `globalVariable`    | the configured global variable name                         |
| `baseLocale`        | `project.default_locale`                                    |

## SvelteKit files

Add the Vite plugin so generated files update during development:

```ts
// vite.config.ts
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import linguini from "@antepod/linguini-vite";

export default defineConfig({
  plugins: [sveltekit(), linguini()],
});
```

Build the generated runtime:

```sh
linguini build
```

Use the generated hooks and root layout load:

```ts
// src/hooks.server.ts
export { handle } from "$lib/generated/linguini/sveltekit";
```

```ts
// src/hooks.ts
export { reroute } from "$lib/generated/linguini/sveltekit";
```

```ts
// src/routes/+layout.server.ts
export { load } from "$lib/generated/linguini/sveltekit";
```

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

<h1>{l.home.title()}</h1>
```

The browser helper localizes internal links after hydration and watches links
created later. For localized SSR output, render the generated helper result
directly:

```svelte
<script lang="ts">
  import { l, localizeHref } from "$lib/generated/linguini/svelte";
</script>

<a href={localizeHref("/pricing")}>{l.nav.pricing()}</a>
<a href={localizeHref("/account/settings")}>{l.nav.settings()}</a>
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
  import { l, setLocale } from "$lib/generated/linguini/svelte";
  import { locales } from "$lib/generated/linguini";
</script>

<nav>
  {#each locales as locale}
    <button type="button" onclick={() => setLocale(locale)}>{locale}</button>
  {/each}
</nav>

<p>{l.home.subtitle()}</p>
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
