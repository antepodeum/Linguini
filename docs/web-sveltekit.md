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

# SvelteKit auto-localizes internal <a href="..."> links by default.
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
ambient declarations.

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

Internal links are localized automatically by the SvelteKit helper. Write normal
SvelteKit links:

```svelte
<a href="/pricing">{l.nav.pricing()}</a>
<a href="/account/settings">{l.nav.settings()}</a>
```

For the `ru` locale these become `/ru/pricing` and `/ru/account/settings` during
SSR, and dynamically created client links are updated after hydration. External
links, `mailto:`/`tel:` links, hash-only links, `download` links, excluded routes,
and links marked with `data-linguini-ignore` or `data-linguini-no-localize` are
left unchanged.

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
  localizeMarkupLinks,
  localizeUrl,
  shouldLocalizeHref,
} from "$lib/generated/linguini/svelte";
```

## Server usage

`handle` stores a request-scoped context in `event.locals`:

```ts
export function load({ locals }) {
  return {
    title: locals.l.home.title(),
    locale: locals.locale,
    canonical: locals.linguini.localizeHref("/pricing"),
  };
}
```
