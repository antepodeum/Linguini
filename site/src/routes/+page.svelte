<script lang="ts">
  import {
    ArrowRight,
    Braces,
    CheckCircle2,
    Code2,
    Globe2,
    Languages,
    Layers3,
    Sparkles,
    TerminalSquare
  } from '@lucide/svelte';
  import Button from '$lib/components/button.svelte';
  import { createLinguini, type Locale } from '$lib/linguini';

  let locale: Locale = $state('en');
  const l = $derived(createLinguini(locale));

  const stages = [
    { icon: Braces, label: 'Schema', text: 'Message signatures, TypeKind, enums, formatter aliases.' },
    { icon: Languages, label: 'Locale', text: 'Plural, case, gender, and custom grammar live in language files.' },
    { icon: TerminalSquare, label: 'Codegen', text: 'Native TypeScript modules ship without runtime DSL parsing.' }
  ];

  const navIds = ['why', 'language', 'codegen', 'web'];

  const codeTabs = [
    {
      name: 'schema.lgs',
      code: 'type Money = Decimal @currency(code = "EUR")\ntype ShortDate = Date @date(style = "short")\n\ncheckout_total(amount: Money, created: ShortDate)'
    },
    {
      name: 'en.lgl',
      code: 'checkout_total = Total {amount} on {created}\n\n// Locale can override when needed:\n// {amount @number}'
    },
    {
      name: 'generated.ts',
      code: 'export function checkout_total(amount: Money, created: ShortDate) {\n  return "Total " + String(formatCurrency(amount, data, { code: "EUR" }))\n    + " on " + String(formatDate(created, data, { style: "short" }));\n}'
    }
  ];
</script>

<svelte:head>
  <title>Linguini — typed localization language</title>
  <meta
    name="description"
    content="Linguini is a compiled localization language with typed schemas, locale grammar, CLDR formatting, and TypeScript/Svelte support."
  />
</svelte:head>

<div class="grain"></div>

<main class="relative overflow-hidden">
  <nav class="mx-auto flex max-w-7xl items-center justify-between px-5 py-5 sm:px-8">
    <a href="/" class="flex items-center gap-3 font-semibold tracking-normal">
      <span class="flex h-9 w-9 items-center justify-center rounded-2xl bg-foreground text-background">
        <Globe2 size={18} />
      </span>
      Linguini
    </a>

    <div class="hidden items-center gap-6 text-sm text-muted-foreground md:flex">
      {#each l.nav as item, index (item)}
        <a href={`#${navIds[index]}`} class="transition hover:text-foreground">{item}</a>
      {/each}
    </div>

    <div class="flex items-center rounded-full border border-border bg-white/70 p-1 shadow-sm">
      <span class="px-3 text-xs text-muted-foreground">{l.localeToggle}</span>
      {#each ['en', 'ru'] as item (item)}
        <button
          class={[
            'h-8 rounded-full px-3 text-sm font-medium transition',
            locale === item ? 'bg-foreground text-background' : 'text-muted-foreground hover:text-foreground'
          ]}
          onclick={() => (locale = item as Locale)}
        >
          {item.toUpperCase()}
        </button>
      {/each}
    </div>
  </nav>

  <section class="mx-auto grid min-h-[calc(100vh-5rem)] max-w-7xl items-center gap-10 px-5 pb-16 pt-8 sm:px-8 lg:grid-cols-[1.02fr_0.98fr]">
    <div class="max-w-3xl">
      <div class="mb-6 inline-flex items-center gap-2 rounded-full border border-border bg-white/75 px-3 py-2 text-sm text-muted-foreground shadow-sm">
        <Sparkles size={16} class="text-accent" />
        {l.heroEyebrow}
      </div>
      <h1 class="font-serif text-7xl font-semibold leading-[0.9] tracking-normal text-foreground sm:text-8xl lg:text-9xl">
        {l.heroTitle}
      </h1>
      <p class="mt-7 max-w-2xl text-xl leading-8 text-muted-foreground sm:text-2xl sm:leading-9">
        {l.heroCopy}
      </p>

      <div class="mt-9 flex flex-col gap-3 sm:flex-row">
        <Button href="https://github.com/antepodeum/Linguini/blob/main/docs/getting-started.md">
          {l.primaryCta}
          <ArrowRight size={17} />
        </Button>
        <Button href="https://github.com/antepodeum/Linguini" variant="secondary">
          <Code2 size={17} />
          {l.secondaryCta}
        </Button>
      </div>

      <div class="mt-10 flex flex-wrap gap-3 text-sm">
        <span class="rounded-full border border-border bg-white/70 px-4 py-2">{l.schemaChip}</span>
        <span class="rounded-full border border-border bg-white/70 px-4 py-2">{l.localeChip}</span>
        <span class="rounded-full border border-border bg-white/70 px-4 py-2">{l.generatedChip}</span>
      </div>
    </div>

    <div class="relative">
      <div class="absolute -right-8 -top-8 h-40 w-40 rounded-full bg-accent/20 blur-3xl"></div>
      <div class="relative overflow-hidden rounded-[2rem] border border-foreground/10 bg-foreground p-4 text-background shadow-soft">
        <div class="mb-4 flex items-center gap-2 px-2 pt-1">
          <span class="h-3 w-3 rounded-full bg-accent"></span>
          <span class="h-3 w-3 rounded-full bg-muted"></span>
          <span class="h-3 w-3 rounded-full bg-primary"></span>
        </div>
        <div class="grid gap-3">
          {#each codeTabs as tab (tab.name)}
            <section class="rounded-3xl border border-white/10 bg-white/[0.06] p-4">
              <div class="mb-3 flex items-center justify-between text-xs uppercase text-background/55">
                <span>{tab.name}</span>
                <CheckCircle2 size={15} />
              </div>
              <pre class="overflow-x-auto whitespace-pre-wrap text-sm leading-6 text-background/88"><code>{tab.code}</code></pre>
            </section>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <section id="why" class="border-y border-border bg-white/45">
    <div class="mx-auto grid max-w-7xl gap-8 px-5 py-16 sm:px-8 lg:grid-cols-[0.85fr_1.15fr]">
      <div>
        <p class="text-sm font-semibold uppercase text-primary">Pipeline</p>
        <h2 class="mt-3 font-serif text-4xl font-semibold leading-tight sm:text-5xl">{l.proofTitle}</h2>
      </div>
      <div class="grid gap-4 sm:grid-cols-2">
        {#each l.features as feature (feature)}
          <div class="rounded-3xl border border-border bg-background/70 p-5 shadow-sm">
            <Layers3 class="mb-5 text-primary" size={22} />
            <p class="leading-7 text-muted-foreground">{feature}</p>
          </div>
        {/each}
      </div>
    </div>
  </section>

  <section id="language" class="mx-auto max-w-7xl px-5 py-16 sm:px-8">
    <div class="mb-8 flex flex-col justify-between gap-4 sm:flex-row sm:items-end">
      <div>
        <p id="codegen" class="text-sm font-semibold uppercase text-primary">{l.outputLabel}</p>
        <h2 class="mt-3 font-serif text-4xl font-semibold sm:text-5xl">{l.sampleTitle}</h2>
      </div>
      <Button href="https://github.com/antepodeum/Linguini/blob/main/docs/reference.md" variant="ghost">
        Reference
        <ArrowRight size={17} />
      </Button>
    </div>

    <div id="web" class="grid gap-4 lg:grid-cols-3">
      {#each stages as stage (stage.label)}
        {@const Icon = stage.icon}
        <article class="rounded-3xl border border-border bg-white/70 p-6 shadow-sm">
          <Icon class="mb-6 text-accent" size={28} />
          <h3 class="text-xl font-semibold">{stage.label}</h3>
          <p class="mt-3 leading-7 text-muted-foreground">{stage.text}</p>
        </article>
      {/each}
    </div>
  </section>
</main>
