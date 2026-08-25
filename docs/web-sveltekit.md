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
with equivalent positional and named-object overloads
(`l.home.greeting("Artemy")` or `l.home.greeting({ name: "Artemy" })`).

Dynamic access is strict by default. A finite, explicit escape can be enabled
with canonical message paths:

```toml fragment=dynamic-policy
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

```toml fragment=web-policy
[web.routing]
locale_prefix = "except-default" # default; also "always" or "never"
canonical = "redirect"   # or "preserve"

# Sources are checked left-to-right; the first supported locale wins.
[web.locale]
sources = ["path", "cookie", "local-storage", "accept-language"]

[web.cookie]
name = "LINGUINI_LOCALE"
path = "auto"
max_age = "365d"
same_site = "lax" # "lax", "strict", or "none"
secure = false     # "auto" is also supported
http_only = false

[web.local_storage]
key = "LINGUINI_LOCALE"

[web.links]
mode = "transform" # "transform" (default), "runtime", or "manual"

[web.routes]
exclude = ["/_app/**", "/favicon.ico"]

# Optional no-JavaScript locale switch transport.
[web.switch_route]
path = "/_linguini/locale/{locale}"
return_query = "return"
status = 303
```

Supported locale matching is compiled from the pinned CLDR canonicalization and parent graph.
Aliases, likely-script parents, and configured locale spellings resolve through one generated
lookup; tags outside that pinned graph are unsupported and continue to the next source or the
implicit base-locale fallback.

Linguini derives URL environment facts from SvelteKit instead of duplicating
them in `linguini.toml` or generated runtime policy:

- Application base comes from SvelteKit's `$app/paths` `base` export. It follows
  `kit.paths.base` in both server and browser modules. `cookie.path = "auto"`
  uses that base, falling back to `/` for a root deployment.
- Request origin comes from `event.url.origin` on the server. Browser helpers
  use `window.location`; callers of the framework-agnostic runtime can supply
  the same request environment explicitly.
- Trailing-slash behavior belongs to the matched SvelteKit route. Linguini
  preserves the slash shape of the input URL; SvelteKit applies that route's
  `trailingSlash` option, including canonical redirects and prerendered output.

Set an application-wide trailing-slash default in the root layout and override
it in child routes when needed:

```ts
// src/routes/+layout.ts
export const trailingSlash = "never"; // "never", "always", or "ignore"
```

Changing `BASE_PATH`/`kit.paths.base` therefore requires no post-generation
rewrite of Linguini files. Route-specific trailing-slash overrides also require
no Linguini configuration.

Available locale sources:

| Source              | Reads from                                                  |
| ------------------- | ----------------------------------------------------------- |
| `path`              | the first localized path segment, for example `/ru/pricing` |
| `cookie`            | the configured locale cookie                                |
| `local-storage`     | the configured browser storage key                          |
| `accept-language`   | the server `Accept-Language` header                         |

Sources are resolved exactly in configured order. Without an explicit list,
the default is `path`, `cookie`, then `accept-language`; with
`locale_prefix = "never"`, it is `cookie`, then `accept-language`. If no source
selects a supported locale, the project default locale wins. `always` prefixes
every locale, `except-default` omits only the default locale, and `never` omits
all locale path segments and therefore cannot be combined with the `path`
source.

The cookie capability exists only when `cookie` is selected. Its defaults are
the project-derived `LINGUINI_<PROJECT>_LOCALE` name, automatic SvelteKit base
path, no domain, `365d`, `SameSite=Lax`, secure `auto`, and `HttpOnly=false`.
Cookie names, paths, domains, and positive durations are validated;
`SameSite=None` requires `secure = true`, and browser switching requires
`http_only = false`. The local-storage capability likewise exists only when
selected and defaults to the project-derived `linguini:<project>:locale` key;
its key must be non-empty and contain no control characters.

SvelteKit SSR cannot read browser local storage. A
`sources = ["local-storage"]` SvelteKit configuration is rejected with a
diagnostic; add `cookie` or `path` so the server can resolve the locale, or use
the client-only `svelte` framework. A switch route likewise requires `path` or
`cookie`, because an HTTP response cannot write local storage.

Generated output contains only selected capabilities: source resolvers, one
link implementation, an exclusion matcher when exclusions exist, server cookie
persistence when cookies are selected, and the switch route only when its table
is present.

Route exclusions are exact path matches unless a pattern ends in `/**`; that
form matches the named path and its slash-delimited descendants, but not a
longer sibling prefix. Patterns must start with one `/`, cannot contain a query
or fragment, and are shared by locale resolution, canonical redirects, link
localization, and the switch route.

Common policies are concise:

```toml fragment=policy-alternatives
# Path-only: locale is in every URL and switching navigates.
[web.routing]
locale_prefix = "always"
[web.locale]
sources = ["path"]

# Cookie-only/pathless: switching persists without changing the URL.
# [web.routing]
# locale_prefix = "never"
# [web.locale]
# sources = ["cookie", "accept-language"]

# Hybrid: URL wins, cookie remembers, header negotiates first visit.
# [web.locale]
# sources = ["path", "cookie", "accept-language"]

# Client-only Svelte storage (not valid as the sole SvelteKit SSR source).
# [web.locale]
# sources = ["local-storage"]
```

## Migration from flat web options

Flat `[web]` keys are rejected. Migrate each configured field explicitly:

| Removed flat config | Current config |
| --- | --- |
| `strategy = ["url", "cookie", "localStorage", "header"]` | `[web.locale] sources = ["path", "cookie", "local-storage", "accept-language"]` |
| `cookie_name` | `[web.cookie] name` |
| `cookie_path` | `[web.cookie] path`; prefer `"auto"` for the SvelteKit base |
| `cookie_domain` | `[web.cookie] domain` |
| `cookie_max_age = 86400` | `[web.cookie] max_age = "1d"` |
| `cookie_same_site` | `[web.cookie] same_site` |
| `cookie_secure` | `[web.cookie] secure`; `"auto"` derives it from the request protocol |
| `cookie_http_only` | `[web.cookie] http_only` |
| `local_storage_key` | `[web.local_storage] key` |
| `global_variable_name` | removed; choose a validated locale source |
| `prefix_default_locale = true` | `[web.routing] locale_prefix = "always"` |
| `prefix_default_locale = false` | `[web.routing] locale_prefix = "except-default"` |
| `redirect = true` / `false` | `[web.routing] canonical = "redirect"` / `"preserve"` |
| `exclude` | `[web.routes] exclude` |
| `localize_links = true` | `[web.links] mode = "runtime"` (or `"transform"` for build-time anchors) |
| `localize_links = false` | `[web.links] mode = "manual"` |
| `base_path` | removed; SvelteKit supplies `$app/paths.base` |
| `origin` | removed; request/browser context supplies it |
| `trailing_slash` | removed; the matched SvelteKit route owns it |
| `targets.ts.module` | removed; generated output is always ESM |

The former `url`, `localStorage`, and `header` strategy spellings are now
`path`, `local-storage`, and `accept-language`. `baseLocale` is an implicit final
fallback and is not listed. The unused `preferredLanguage`, `navigator`,
`globalVariable`, `custom-*`, and `global_variable_name` paths have no direct
replacement; choose one of the four validated sources instead.

`base_path`, `origin`, and `trailing_slash` also have no Linguini config
replacement. SvelteKit supplies base and origin at runtime and owns the matched
route's trailing-slash policy, as described above. `targets.ts.module` is
removed because generated output is always ESM.

Framework-agnostic callers of `createWebI18n` use the same nested shape:

| Removed runtime option | Current runtime option |
| --- | --- |
| `sources` | `locale.sources` |
| `localeSwitch` | `locale.switch` (normally generated from config) |
| `localePrefix` / `prefixDefaultLocale` | `routing.localePrefix` |
| `redirect` | `routing.canonical` with `"redirect"` or `"preserve"` |
| `cookieName`, `cookiePath`, `cookieDomain` | `cookie.name`, `cookie.path`, `cookie.domain` |
| `cookieMaxAge`, `cookieSameSite` | `cookie.maxAge`, `cookie.sameSite` |
| `cookieSecure`, `cookieHttpOnly` | `cookie.secure`, `cookie.httpOnly` |
| `localStorageKey` | `localStorage.key` |
| `globalVariableName` | removed; choose a validated locale source |
| `localizeLinks` | `links.mode` |
| `exclude` | `routes.exclude` |
| `basePath`, `origin` | third `environment` argument: `base`, `origin` |
| `trailingSlash` | removed; preserve the input path and let the router canonicalize it |

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

The generated `linguini-app.d.ts` augments `App.Locals` and `App.PageData`, so
server loads can use the request-scoped locale context without handwritten app
ambient declarations. The locale-only control hook and full hook are represented
truthfully as a union; narrow on `"l" in linguini` before reading messages.
Linguini owns only `locals.linguini`; it does not reserve
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

With `mode = "transform"`, the Vite plugin rewrites safe static Svelte anchor
attributes to the generated `localizeHref` helper. `mode = "runtime"` instead
ships the bounded browser observer for anchors created after hydration.
`mode = "manual"` emits neither mechanism. For dynamic or programmatic URLs,
render the helper result directly:

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
selected link implementation.

When `[web.switch_route]` is enabled, ordinary static links can switch locale
without JavaScript:

```svelte
<a href="/_linguini/locale/en?return=/account">English</a>
<a href="/_linguini/locale/ru?return=/account">Русский</a>
```

The generated handler validates the locale and same-origin return target, falls
back to a same-origin `Referer` and then the SvelteKit base path, uses
`localizeHref` only when the shared transition plan writes the path, writes the
configured cookie only when selected, and returns the configured redirect
status. Absolute, protocol-relative, backslash, and encoded external return
targets are rejected.

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
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = ({ locals }) => {
  const { linguini } = locals;
  if (!("l" in linguini)) {
    throw new Error("Use linguiniHandle from the generated sveltekit module");
  }
  return {
    title: linguini.l.home.title,
    locale: linguini.locale,
    canonical: linguini.localizeHref("/pricing"),
  };
};
```
