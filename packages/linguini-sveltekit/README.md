# @antepod/linguini-sveltekit

SvelteKit adapter for Linguini. Enable it with one TypeScript target option:

```toml
[targets.ts]
out = "src/lib/generated/linguini"
module = "esm"
declaration = true
framework = "sveltekit"
```

Then import the generated files directly:

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

Components can import only the generated `l` object:

```svelte
<script lang="ts">
  import { l } from "$lib/generated/linguini/svelte";
</script>

<h1>{l.home.title()}</h1>
<a href="/pricing">{l.nav.pricing()}</a>
```

Internal `<a href="...">` links are localized automatically on SSR and after
client-side hydration. Add `data-linguini-ignore` to a single link or set
`localize_links = false` in `[web]` to opt out.

The generated Svelte entrypoint also exports `setLocale`, `localizeHref`,
`localizeUrl`, `delocalizeUrl`, `alternateLinks`, and the `linguini` rune object
for programmatic URLs and locale switching.

## Global App Types

The generated `sveltekit.d.ts` augments `App.Locals` and `App.PageData` with
inline `import("...")` type references. SvelteKit global namespace declarations
cannot rely on imported local aliases, so keep your own `src/app.d.ts` free of
generated imports and let TypeScript include the generated declaration file.
