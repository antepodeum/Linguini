<script lang="ts">
  import { l } from '$lib/generated/linguini/svelte';
  import { locales } from '$lib/generated/linguini/locale.js';
  import { localizeHref } from '$lib/linguini.js';

  const headingKeys = ['title', 'tagline'] as const;
  const formatterKeys = ['number_format', 'currency_format'] as const;
  const formatters = formatterKeys.map(key => l.main.playground[key]);
  const amount = 1299.5;
  const headings = $derived(headingKeys.map(key => l.main.hero[key]));
  const formatted = $derived(formatters.map(format => format(amount)));
  const named = $derived(l.main.playground[formatterKeys[0]]({value:amount}));
</script>

<svelte:head><title>Message lab</title></svelte:head>

<main class="mx-auto min-h-screen max-w-3xl px-5 py-12 sm:px-8">
  <h1 class="font-serif text-4xl">Message lab</h1>
  <nav aria-label="Message lab locales" class="mt-6 flex flex-wrap gap-4">
    {#each locales as locale (locale)}
      <a href={localizeHref('/message-lab',locale)} data-linguini-no-localize>{locale}</a>
    {/each}
  </nav>
  <section class="mt-6" aria-label="Computed message values">
    {#each headings as heading, index (headingKeys[index])}<p>{heading}</p>{/each}
  </section>
  <section class="mt-6" aria-label="Computed message functions">
    {#each formatted as value, index (formatterKeys[index])}<p>{value}</p>{/each}
    <p>{named}</p>
  </section>
  <p class="mt-6" data-fallback>{l.message_lab.fallback}</p>
</main>
