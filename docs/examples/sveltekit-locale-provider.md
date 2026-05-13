# SvelteKit Locale Provider Example

Generated `index.ts` exposes direct locale facades:

```ts
import { createLinguini, createLinguiniProvider, lgl } from "$lib/generated/linguini";

lgl.delivery("apple", "small", 1);
createLinguini("ru").delivery("apple", "small", 1);
createLinguiniProvider({ resolveLanguage: () => "ru" }).delivery("apple", "small", 1);
```

SSR hooks can resolve language from a cookie first, then known request headers:

```ts
// src/app.d.ts
declare global {
  namespace App {
    interface Locals {
      language: "en" | "ru";
    }
  }
}

export {};
```

```ts
// src/hooks.server.ts
import type { Handle } from "@sveltejs/kit";

const supportedLanguages = ["en", "ru"] as const;
type Language = (typeof supportedLanguages)[number];

function isLanguage(value: string | undefined): value is Language {
  return supportedLanguages.includes(value as Language);
}

function languageFromAcceptLanguage(header: string | null): Language | undefined {
  return header
    ?.split(",")
    .map((part) => part.trim().split(";")[0])
    .find(isLanguage);
}

export const handle: Handle = async ({ event, resolve }) => {
  const cookieLanguage = event.cookies.get("locale");
  const headerLanguage =
    event.request.headers.get("x-linguini-locale") ??
    event.request.headers.get("x-locale") ??
    languageFromAcceptLanguage(event.request.headers.get("accept-language"));

  event.locals.language = isLanguage(cookieLanguage) ? cookieLanguage : headerLanguage ?? "en";
  return resolve(event);
};
```

Route data carries the resolved language:

```ts
// src/routes/+layout.server.ts
import type { LayoutServerLoad } from "./$types";

export const load: LayoutServerLoad = ({ locals }) => {
  return { language: locals.language };
};
```

Server modules can use the same resolved value directly:

```ts
// src/routes/api/preview/+server.ts
import { json } from "@sveltejs/kit";
import { createLinguini } from "$lib/generated/linguini";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = ({ locals }) => {
  const lgl = createLinguini(locals.language);
  return json({ title: lgl.delivery("apple", "small", 1) });
};
```

For SSR, keep locale state in Svelte context, not a server module singleton:

```ts
// src/lib/i18n.ts
import { getContext, setContext } from "svelte";
import type { Linguini } from "$lib/generated/linguini";

const key = Symbol("linguini");

export function setLgl(lgl: Linguini) {
  setContext(key, lgl);
}

export function getLgl(): Linguini {
  return getContext<Linguini>(key);
}
```

```svelte
<!-- src/routes/+layout.svelte -->
<script lang="ts">
  import { createLinguini, type Linguini } from "$lib/generated/linguini";
  import { setLgl } from "$lib/i18n";

  let { data, children } = $props();
  let active = $derived(createLinguini(data.language));
  const lgl = new Proxy({} as Linguini, {
    get(_target, property) {
      return active[property as keyof Linguini];
    },
  });

  setLgl(lgl);
</script>

{@render children()}
```

Components keep the short call shape:

```svelte
<script lang="ts">
  import { getLgl } from "$lib/i18n";

  const lgl = getLgl();

  let fruit = "apple" as const;
  let size = "small" as const;
</script>

<p>{lgl.delivery(fruit, size, 1)}</p>
```

For client-only apps or SPA mode, importing generated `lgl` directly is fine. For SSR, prefer per-request context above.
