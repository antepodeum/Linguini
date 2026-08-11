<script lang="ts">
  import { onMount } from 'svelte';
  import { localizeHref, linguini, setLocale } from '$lib/generated/linguini/svelte-control';
  import { l } from '$lib/generated/linguini/svelte';

  const labLocales = ['en', 'ru', 'fr'] as const;
  type LabLocale = (typeof labLocales)[number];

  let resourceScripts = $state<string[]>([]);

  onMount(() => {
    resourceScripts = [
      ...new Set(
        performance
          .getEntriesByType('resource')
          .map((entry) => new URL(entry.name).pathname)
          .filter((pathname) => pathname.endsWith('.js'))
          .map((pathname) => pathname.slice(pathname.lastIndexOf('/') + 1))
          .filter(Boolean)
      )
    ].sort();
  });

  function chooseLocale(locale: LabLocale) {
    void setLocale(locale);
  }
</script>

<svelte:head>
  <title>Chunk lab</title>
  <meta
    name="description"
    content="A small production route for inspecting localized client chunks."
  />
</svelte:head>

<main class="mx-auto min-h-screen max-w-3xl px-5 py-12 sm:px-8">
  <div class="rounded-2xl border border-border/70 bg-muted/40 p-6 shadow-soft sm:p-10">
    <p class="font-mono text-xs uppercase tracking-[0.2em] text-primary">Chunk lab</p>
    <h1 class="mt-4 font-serif text-4xl tracking-tight text-foreground sm:text-5xl">
      {l.main.hero.title}
    </h1>
    <p class="mt-4 max-w-2xl text-lg leading-8 text-muted-foreground">{l.main.hero.tagline}</p>

    <section class="mt-10 border-t border-border/70 pt-6" aria-labelledby="locale-heading">
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h2 id="locale-heading" class="text-sm font-semibold text-foreground">
            {l.main.nav.locale_label}
          </h2>
          <p class="mt-1 text-sm text-muted-foreground">
            Active locale: <code class="font-mono text-primary">{linguini.locale}</code>
          </p>
        </div>

        <nav aria-label="Chunk lab locales" class="flex items-center gap-2">
          {#each labLocales as locale (locale)}
            <a
              href={localizeHref('/chunk-lab', locale)}
              data-linguini-no-localize
              aria-current={linguini.locale === locale ? 'page' : undefined}
              class:font-semibold={linguini.locale === locale}
              class="rounded-md border border-border/70 px-3 py-1.5 text-sm text-muted-foreground transition hover:border-primary hover:text-foreground"
              onclick={(event) => {
                event.preventDefault();
                chooseLocale(locale);
              }}
            >
              {locale.toUpperCase()}
            </a>
          {/each}
        </nav>
      </div>
    </section>

    <section class="mt-10 border-t border-border/70 pt-6" aria-labelledby="resources-heading">
      <div class="flex items-baseline justify-between gap-4">
        <h2 id="resources-heading" class="text-sm font-semibold text-foreground">
          Client Resource Timing JavaScript
        </h2>
        <span class="font-mono text-xs text-muted-foreground">{resourceScripts.length} files</span>
      </div>
      {#if resourceScripts.length > 0}
        <ul class="mt-4 grid gap-2 sm:grid-cols-2" aria-label="Client JavaScript resource filenames">
          {#each resourceScripts as filename (filename)}
            <li class="rounded-md bg-background/70 px-3 py-2 font-mono text-xs text-muted-foreground">
              {filename}
            </li>
          {/each}
        </ul>
      {:else}
        <p class="mt-4 text-sm text-muted-foreground">
          No client JavaScript resource timing entries yet.
        </p>
      {/if}
    </section>
  </div>
</main>
