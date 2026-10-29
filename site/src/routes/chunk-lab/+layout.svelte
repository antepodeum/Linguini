<script lang="ts">
  import { l } from '$lib/generated/linguini/svelte';
  let { children } = $props();
</script>

<svelte:head><meta name="linguini-lab-label" content={l.main.nav.locale_label} /></svelte:head>
{@render children()}
