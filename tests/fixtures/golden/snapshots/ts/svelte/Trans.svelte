<script lang="ts">
  import type { Snippet } from "svelte";

  export type RichTextPart = string | { name: string; text: string };
  export type RichTextComponents = Record<string, Snippet<[string]>>;

  let { value, components = {} }: {
    value: readonly RichTextPart[];
    components?: RichTextComponents;
  } = $props();
</script>

{#each value as part, index (index)}
  {#if typeof part === "string"}
    {part}
  {:else if components[part.name]}
    {@render components[part.name](part.text)}
  {:else}
    {part.text}
  {/if}
{/each}
