<script lang="ts">
  import {
    ArrowRight,
    Braces,
    Code2,
    Component,
    Cookie,
    FileCode2,
    Globe2,
    Hash,
    Languages,
    Route,
    Sparkles,
    Terminal,
    Zap
  } from '@lucide/svelte';
  import Button from '$lib/components/button.svelte';
  import { l, linguini, localizeHref, setLocale } from '$lib/generated/linguini/svelte';
  import { locales, type Locale } from '$lib/generated/linguini';
  import type { Fruit, Gender, Size } from '$lib/generated/linguini/locales/en';

  let count = $state(3);
  let fruit = $state<Fruit>('apple');
  let size = $state<Size>('small');
  let gender = $state<Gender>('neuter');
  let amount = $state(1299.5);
  let dateInput = $state('2026-05-19');

  const dateValue = $derived(new Date(`${dateInput}T12:00:00Z`) as unknown as string);
  const localizedRoot = $derived(localizeHref('/'));
  const localizedPlayground = $derived(localizeHref('/playground'));

  const nav = $derived([
    { id: 'why', label: l.main.nav_why },
    { id: 'codegen', label: l.main.nav_codegen },
    { id: 'language', label: l.main.nav_language },
    { id: 'web', label: l.main.nav_web }
  ]);

  const shippedTargets = $derived([
    {
      icon: FileCode2,
      title: l.main.codegen_ts_title,
      description: l.main.codegen_ts_desc,
      status: l.main.codegen_status_shipped
    },
    {
      icon: Component,
      title: l.main.codegen_svelte_title,
      description: l.main.codegen_svelte_desc,
      status: l.main.codegen_status_shipped
    }
  ]);

  const plannedTargets = $derived([
    { icon: Braces, label: l.main.codegen_rust },
    { icon: Zap, label: l.main.codegen_swift },
    { icon: Terminal, label: l.main.codegen_go },
    { icon: Hash, label: l.main.codegen_csharp },
    { icon: Code2, label: l.main.codegen_kotlin },
    { icon: Languages, label: l.main.codegen_python }
  ]);

  const playgroundLines = $derived([
    l.main.playground_sentence(fruit, size, gender, count, amount, dateValue),
    l.main.cart_summary(count, fruit),
    l.main.number_format(amount),
    l.main.currency_format(amount),
    l.main.date_format(dateValue),
    l.main.override_format(amount, dateValue),
    l.main.gender_line(gender),
    l.main.size_line(size)
  ]);

  function localeLabel(locale: Locale) {
    return locale === 'pt-BR' ? 'PT-BR' : locale.toUpperCase();
  }

  async function chooseLocale(locale: Locale) {
    await setLocale(locale, {
      cookie: true,
      navigate: true,
      replaceState: false,
      keepFocus: true,
      noScroll: true
    });
  }
</script>

<svelte:head>
  <title>{l.main.hero_title} — typed localization language</title>
  <meta name="description" content={l.main.hero_copy} />
</svelte:head>

<div class="grain"></div>

<main class="relative overflow-hidden">
  <nav class="sticky top-0 z-20 border-b border-border/70 bg-background/88 backdrop-blur-xl">
    <div class="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-4 px-5 py-4 sm:px-8">
    <a href={localizedRoot} class="flex items-center gap-3 font-semibold tracking-normal">
      <span class="flex h-9 w-9 items-center justify-center rounded-xl bg-primary text-primary-foreground shadow-[0_0_32px_hsl(15_96%_55%_/_0.35)]">
        <Globe2 size={18} />
      </span>
      <span class="text-lg tracking-[0.28em]">{l.main.hero_title.toUpperCase()}</span>
    </a>

    <div class="hidden items-center gap-6 text-sm text-muted-foreground md:flex">
      {#each nav as item (item.id)}
        <a href={`#${item.id}`} class="transition hover:text-foreground">{item.label}</a>
      {/each}
    </div>

    <div class="flex max-w-full flex-wrap items-center rounded-full border border-border bg-muted/70 p-1 shadow-[inset_0_1px_0_hsl(0_0%_100%_/_0.05)]">
      <span class="px-3 text-xs text-muted-foreground">{l.main.locale_label}</span>
      {#each locales as item (item)}
        <a
          href={localizeHref('/', item)}
          data-linguini-ignore
          class={[
            'flex h-8 items-center rounded-full px-2.5 text-xs font-medium transition sm:px-3 sm:text-sm',
            linguini.locale === item
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:text-foreground'
          ]}
          onclick={(event) => {
            event.preventDefault();
            chooseLocale(item);
          }}
        >
          {localeLabel(item)}
        </a>
      {/each}
    </div>
    </div>
  </nav>

  <section class="mx-auto grid min-h-[calc(100vh-4.5rem)] max-w-7xl items-center gap-10 px-5 pb-20 pt-14 sm:px-8 lg:grid-cols-[0.86fr_1.14fr]">
    <div class="max-w-3xl self-center">
      <div class="mb-7 inline-flex items-center gap-2 rounded-full border border-primary/35 bg-primary/10 px-3 py-2 text-sm text-primary">
        <Sparkles size={16} class="text-primary" />
        {l.main.hero_eyebrow}
      </div>
      <h1 class="font-serif text-7xl font-semibold leading-[0.86] tracking-normal text-foreground sm:text-8xl lg:text-[10.5rem]">
        {l.main.hero_title}
      </h1>
      <p class="mt-7 max-w-2xl text-xl leading-8 text-muted-foreground sm:text-2xl sm:leading-9">
        {l.main.hero_copy}
      </p>

      <div class="mt-9 flex flex-col gap-3 sm:flex-row">
        <Button href="https://github.com/antepodeum/Linguini/blob/main/docs/getting-started.md">
          {l.main.primary_cta}
          <ArrowRight size={17} />
        </Button>
        <Button href="https://github.com/antepodeum/Linguini" variant="secondary">
          <Code2 size={17} />
          {l.main.secondary_cta}
        </Button>
      </div>

      <div class="mt-10 grid max-w-xl grid-cols-3 overflow-hidden rounded-2xl border border-border bg-muted/35 text-center text-sm">
        <span class="px-3 py-3 text-muted-foreground">{l.main.hero_trait_typed}</span>
        <span class="border-x border-border px-3 py-3 text-muted-foreground">{l.main.hero_trait_compiled}</span>
        <span class="px-3 py-3 text-muted-foreground">{l.main.hero_trait_native}</span>
      </div>
    </div>

    <div class="hero-machine relative min-h-[560px] overflow-hidden">
      <div class="machine-track"></div>
      <div class="machine-capsule"></div>
      <div class="machine-capsule"></div>
      <div class="machine-capsule"></div>
      <div class="machine-core mt-16">
        <span class="machine-glyph">L</span>
      </div>
      <div class="absolute bottom-3 left-0 right-0 mx-auto w-[min(94%,620px)] overflow-hidden rounded-[1.45rem] border border-border bg-background/78 p-4 shadow-[0_26px_80px_hsl(205_30%_4%_/_0.58)] backdrop-blur-md">
        <div class="mb-3 flex items-center justify-between gap-3 border-b border-border/70 pb-3">
          <span class="font-mono text-xs text-muted-foreground">linguini/schema/main.lgs</span>
          <span class="rounded-full bg-primary/10 px-2.5 py-1 text-xs text-primary">{l.main.codegen_status_shipped}</span>
        </div>
        <div class="grid gap-2 font-mono text-sm leading-6 text-foreground/90">
          <p><span class="text-primary">type</span> Money = Decimal <span class="text-accent">@currency</span></p>
          <p><span class="text-primary">type</span> ShortDate = Date <span class="text-accent">@date</span>(style = "short")</p>
          <p><span class="text-primary">enum</span> Fruit &#123; apple, pear, orange &#125;</p>
          <p>playground_sentence(fruit: Fruit, count: Number, amount: Money)</p>
        </div>
      </div>
    </div>
  </section>

  <section id="why" class="border-y border-border/80 bg-muted/20">
    <div class="mx-auto max-w-7xl px-5 py-20 sm:px-8">
      <p class="text-sm font-semibold uppercase tracking-wide text-primary">{l.main.codegen_kicker}</p>
      <h2 class="mt-3 max-w-3xl font-serif text-4xl font-semibold leading-tight sm:text-5xl">{l.main.codegen_title}</h2>
      <p class="mt-6 max-w-3xl text-lg leading-8 text-muted-foreground">{l.main.codegen_intro}</p>
    </div>
  </section>

  <section id="codegen" class="mx-auto max-w-7xl px-5 py-16 sm:px-8">
    <div class="grid gap-4 lg:grid-cols-2">
      {#each shippedTargets as target (target.title)}
        {@const Icon = target.icon}
        <article class="group rounded-2xl border border-border bg-muted/25 p-6 transition hover:border-primary/30 hover:bg-muted/40">
          <div class="mb-5 flex items-start justify-between gap-4">
            <span class="flex h-12 w-12 items-center justify-center rounded-xl border border-primary/20 bg-primary/10 text-primary">
              <Icon size={24} />
            </span>
            <span class="rounded-full border border-primary/30 bg-primary/10 px-3 py-1 text-xs font-medium text-primary">
              {target.status}
            </span>
          </div>
          <h3 class="text-xl font-semibold">{target.title}</h3>
          <p class="mt-3 leading-7 text-muted-foreground">{target.description}</p>
        </article>
      {/each}
    </div>

    <div class="mt-10 rounded-2xl border border-dashed border-border bg-background/40 p-6 sm:p-8">
      <p class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">{l.main.codegen_planned_title}</p>
      <div class="mt-6 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {#each plannedTargets as target (target.label)}
          {@const Icon = target.icon}
          <div class="flex items-center gap-3 rounded-xl border border-border/70 bg-muted/20 px-4 py-3">
            <span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-border text-muted-foreground">
              <Icon size={18} />
            </span>
            <div class="min-w-0">
              <p class="font-medium">{target.label}</p>
              <p class="text-xs text-muted-foreground">{l.main.codegen_status_planned}</p>
            </div>
          </div>
        {/each}
      </div>
    </div>
  </section>

  <section id="language" class="border-t border-border/80">
    <div class="mx-auto max-w-7xl px-5 py-16 sm:px-8">
      <div class="mb-8 flex flex-col justify-between gap-4 sm:flex-row sm:items-end">
        <div>
          <p class="text-sm font-semibold uppercase tracking-wide text-primary">{l.main.nav_language}</p>
          <h2 class="mt-3 font-serif text-4xl font-semibold sm:text-5xl">{l.main.web_title}</h2>
        </div>
        <Button href="https://github.com/antepodeum/Linguini/blob/main/docs/reference.md" variant="ghost">
          {l.main.primary_cta}
          <ArrowRight size={17} />
        </Button>
      </div>
    </div>
  </section>

  <section id="web" class="border-t border-border bg-muted/30 px-5 py-16 sm:px-8">
    <div class="mx-auto grid max-w-7xl gap-8 lg:grid-cols-[0.8fr_1.2fr]">
      <div>
        <p class="text-sm font-semibold uppercase tracking-wide text-primary">{l.main.playground_kicker}</p>
        <h2 class="mt-3 font-serif text-4xl font-semibold sm:text-5xl">{l.main.playground_title}</h2>
        <div class="mt-8 grid gap-3 text-sm text-muted-foreground">
          <p class="flex items-center gap-2"><Route size={16} class="text-primary" /> {l.main.route_label}: {localizedPlayground}</p>
          <p class="flex items-center gap-2"><Cookie size={16} class="text-primary" /> {l.main.cookie_label}: LINGUINI_SITE_LOCALE={linguini.locale}</p>
        </div>
      </div>

      <div class="grid gap-4">
        <div class="grid gap-3 rounded-3xl border border-border bg-background/50 p-5 sm:grid-cols-2 lg:grid-cols-3">
          <label class="grid gap-2 text-sm">
            {l.main.count_label}
            <input class="rounded-2xl border border-border bg-muted/40 px-3 py-2 text-foreground" type="number" min="0" bind:value={count} />
          </label>
          <label class="grid gap-2 text-sm">
            {l.main.fruit_label}
            <select class="rounded-2xl border border-border bg-muted/40 px-3 py-2 text-foreground" bind:value={fruit}>
              <option value="apple">apple</option>
              <option value="pear">pear</option>
              <option value="orange">orange</option>
            </select>
          </label>
          <label class="grid gap-2 text-sm">
            {l.main.size_label}
            <select class="rounded-2xl border border-border bg-muted/40 px-3 py-2 text-foreground" bind:value={size}>
              <option value="small">small</option>
              <option value="big">big</option>
            </select>
          </label>
          <label class="grid gap-2 text-sm">
            {l.main.gender_label}
            <select class="rounded-2xl border border-border bg-muted/40 px-3 py-2 text-foreground" bind:value={gender}>
              <option value="male">male</option>
              <option value="female">female</option>
              <option value="neuter">neuter</option>
              <option value="other">other</option>
            </select>
          </label>
          <label class="grid gap-2 text-sm">
            {l.main.amount_label}
            <input class="rounded-2xl border border-border bg-muted/40 px-3 py-2 text-foreground" type="number" step="0.01" bind:value={amount} />
          </label>
          <label class="grid gap-2 text-sm">
            {l.main.date_label}
            <input class="rounded-2xl border border-border bg-muted/40 px-3 py-2 text-foreground" type="date" bind:value={dateInput} />
          </label>
        </div>

        <div class="rounded-3xl border border-border bg-background/50 p-5">
          <p class="mb-4 text-sm uppercase tracking-wide text-muted-foreground">{l.main.localized_path_label}</p>
          <p class="mb-6 font-mono text-sm text-primary">{localizedRoot}</p>
          <div class="grid gap-3">
            {#each playgroundLines as line (line)}
              <p class="rounded-2xl border border-border/70 bg-muted/30 px-4 py-3 text-foreground">{line}</p>
            {/each}
          </div>
        </div>
      </div>
    </div>
  </section>
</main>
