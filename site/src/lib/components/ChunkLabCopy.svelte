<script lang="ts">
  import { l } from '$lib/generated/linguini/svelte';
  import { labTagline } from '$lib/chunk-lab-copy.js';
</script>

<h1 class="mt-4 font-serif text-4xl tracking-tight text-foreground sm:text-5xl">
  {l.main.hero.title}
</h1>
<p class="mt-4 max-w-2xl text-lg leading-8 text-muted-foreground">{labTagline()}</p>
