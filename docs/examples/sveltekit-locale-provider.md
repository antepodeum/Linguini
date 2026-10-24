# SvelteKit locale provider example

The SvelteKit runtime adapter supplies request-local locale context. Configure a pathless
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

[targets.ts.bundler]
sources = ["src"]
```

Configure the runtime in an ordinary application module:

```ts
// src/lib/linguini-web.ts
import { base } from "$app/paths";
import {
  createWebLocaleI18n, cookieLocale, requestLanguage,
} from "@antepodeum/linguini-web";
import type { LocaleResolver, LinguiniWebOptions } from "@antepodeum/linguini-web";
import * as locale from "$lib/generated/linguini/locale.js";

const policy = {
  routing: { localePrefix: "never", canonical: "redirect" },
  locale: { switch: { writesPath: false, writesCookie: true, writesLocalStorage: false } },
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
  return createWeb([cookieLocale(), requestLanguage()]);
}
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

`localePrefix: "never"` cannot read a locale from the path. The cookie gives
both the server and browser a persistent source, while `accept-language`
negotiates the first request. Browser controls and the optional server switch route share the application runtime policy: write
the cookie and leave paths unprefixed.

Create the request hooks and root data loader:

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
export const delocalizeUrl = linguini.delocalizeUrl;
export const alternateLinks = linguini.alternateLinks;
```

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

The Svelte runtime consumes that request data and owns the reactive
locale context. Components import message access and controls, not individual
locale or message modules:

```svelte
<!-- src/routes/account/+page.svelte -->
<script lang="ts">
  import { l } from "$lib/generated/linguini/svelte";
  import { localizeHref, setLocale } from "$lib/linguini";

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

`setLocale` prepares a non-base locale before publishing it. SSR message access uses the server request locale. No application-level provider singleton,
manual locale barrel, or eager import of every locale is needed.
