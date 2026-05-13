# SvelteKit Locale Provider Example

Generated `index.ts` exposes a stable facade:

```ts
import { configureLinguini } from "$lib/generated/linguini";

let language = $state("en" as "en" | "ru");
export const linguini = configureLinguini({
  language: () => language,
});

export function setLanguage(next: "en" | "ru") {
  language = next;
}
```

Route data or cookies can own the source variable:

```ts
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = ({ data }) => {
  setLanguage(data.language);
  return {};
};
```

Components keep one call shape while the provider changes active locale:

```svelte
<script lang="ts">
  import { linguini } from "$lib/i18n";

  let fruit = "apple" as const;
  let size = "small" as const;
</script>

<p>{linguini.lgl.delivery(fruit, size, 1)}</p>
```
