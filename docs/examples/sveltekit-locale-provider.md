# SvelteKit locale provider example

The generated SvelteKit adapter is the locale provider. Configure a pathless
site with cookie persistence and the optional no-JavaScript switch transport:

```toml
[project]
name = "app"
default_locale = "en"
locales = ["en", "ru"]

[paths]
schema = "linguini/schema"
locale = "linguini/locale"

[targets.ts]
out = "src/lib/generated/linguini"
declaration = true
framework = "sveltekit"

[targets.ts.bundler]
sources = ["src"]

[web.routing]
locale_prefix = "never"

[web.locale]
sources = ["cookie", "accept-language"]

[web.cookie]
path = "auto"

[web.links]
mode = "transform"

[web.switch_route]
path = "/_linguini/locale/{locale}"
return_query = "return"
status = 303
```

The component below uses this schema and its two locale implementations:

```lgs
// linguini/schema/account.lgs

title
greeting(name: String)
settings
```

```lgl
// linguini/locale/account/en.lgl

title = Account
greeting = Hello, {name}!
settings = Settings
```

```lgl
// linguini/locale/account/ru.lgl

title = Аккаунт
greeting = Привет, {name}!
settings = Настройки
```

`locale_prefix = "never"` cannot read a locale from the path. The cookie gives
both the server and browser a persistent source, while `accept-language`
negotiates the first request. The generated browser switch and switch route use
the same compiled transition plan: write the cookie, leave the path unprefixed,
and navigate to the resulting URL.

Export the generated request hooks and root data loader:

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

The generated Svelte facade consumes that request data and owns the reactive
locale context. Components import message access and controls, not individual
locale or message modules:

```svelte
<!-- src/routes/account/+page.svelte -->
<script lang="ts">
  import {
    l,
    localizeHref,
    setLocale,
  } from "$lib/generated/linguini/svelte";

  let name = "Artemy";
</script>

<svelte:head>
  <title>{l.account.title}</title>
</svelte:head>

<h1>{l.account.greeting({ name })}</h1>
<a href={localizeHref("/account/settings")}>{l.account.settings}</a>

<button type="button" onclick={() => setLocale("en")}>English</button>
<button type="button" onclick={() => setLocale("ru")}>Русский</button>

<noscript>
  <a href="/_linguini/locale/en?return=/account">English</a>
  <a href="/_linguini/locale/ru?return=/account">Русский</a>
</noscript>
```

Here `title` and `settings` are parameterless schema messages, so they are
values. `greeting(name: String)` is callable through its named-object overload.
`localizeHref` is still the single programmatic-link helper; under this pathless
policy it preserves `/account/settings` while applying base/origin rules.

`setLocale` prepares a non-base locale before publishing it. The SvelteKit
request path does the same during SSR. No application-level provider singleton,
manual locale barrel, or eager import of every locale is needed.
