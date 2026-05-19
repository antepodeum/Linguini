<script lang="ts">
  import {
    ArrowRight,
    Braces,
    CheckCircle2,
    Code2,
    Cookie,
    Globe2,
    Languages,
    Layers3,
    Route,
    Sparkles,
    TerminalSquare
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
    { id: 'language', label: l.main.nav_language },
    { id: 'codegen', label: l.main.nav_codegen },
    { id: 'web', label: l.main.nav_web }
  ]);

  const stages = $derived([
    {
      icon: Braces,
      label: 'Schema',
      text: l.main.feature_schema
    },
    {
      icon: Languages,
      label: 'Locale',
      text: l.main.feature_locale
    },
    {
      icon: TerminalSquare,
      label: 'Codegen',
      text: l.main.feature_cldr
    }
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

  const codeTabs = [
    {
      name: 'linguini/schema/main.lgs',
      code: 'type Money = Decimal @currency(code = "USD")\ntype ShortDate = Date @date(style = "short")\n\nenum Fruit { apple, pear, orange }\nenum Size { small, big }\nenum Gender { male, female, neuter, other }\n\nplayground_sentence(fruit: Fruit, size: Size, gender: Gender, count: Number, amount: Money, date: ShortDate)'
    },
    {
      name: 'linguini/locale/main/ru.lgl',
      code: 'playground_sentence = {Delivered(count, gender)} {count} {SizeWord(size)} {fruit.nom(count)}. Итого {amount @currency(code = "RUB")}; дата {date}.'
    },
    {
      name: 'generated/sveltekit.ts',
      code: 'export const handle = createHandle(runtime, options);\nexport const reroute = createReroute(runtime, options);\nexport const load = createLoad();'
    }
  ];

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
  <nav class="mx-auto flex max-w-7xl items-center justify-between px-5 py-5 sm:px-8">
    <a href={localizedRoot} class="flex items-center gap-3 font-semibold tracking-normal">
      <span class="flex h-9 w-9 items-center justify-center rounded-2xl bg-foreground text-background">
        <Globe2 size={18} />
      </span>
      {l.main.hero_title}
    </a>

    <div class="hidden items-center gap-6 text-sm text-muted-foreground md:flex">
      {#each nav as item (item.id)}
        <a href={`#${item.id}`} class="transition hover:text-foreground">{item.label}</a>
      {/each}
    </div>

    <div class="flex items-center rounded-full border border-border bg-white/70 p-1 shadow-sm">
      <span class="px-3 text-xs text-muted-foreground">{l.main.locale_label}</span>
      {#each locales as item (item)}
        <a
          href={localizeHref('/', item)}
          data-linguini-ignore
          class={[
            'flex h-8 items-center rounded-full px-3 text-sm font-medium transition',
            linguini.locale === item
              ? 'bg-foreground text-background'
              : 'text-muted-foreground hover:text-foreground'
          ]}
          onclick={(event) => {
            event.preventDefault();
            chooseLocale(item);
          }}
        >
          {item.toUpperCase()}
        </a>
      {/each}
    </div>
  </nav>

  <section class="mx-auto grid min-h-[calc(100vh-5rem)] max-w-7xl items-center gap-10 px-5 pb-16 pt-8 sm:px-8 lg:grid-cols-[1.02fr_0.98fr]">
    <div class="max-w-3xl">
      <div class="mb-6 inline-flex items-center gap-2 rounded-full border border-border bg-white/75 px-3 py-2 text-sm text-muted-foreground shadow-sm">
        <Sparkles size={16} class="text-accent" />
        {l.main.hero_eyebrow}
      </div>
      <h1 class="font-serif text-7xl font-semibold leading-[0.9] tracking-normal text-foreground sm:text-8xl lg:text-9xl">
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

      <div class="mt-10 flex flex-wrap gap-3 text-sm">
        <span class="rounded-full border border-border bg-white/70 px-4 py-2">{l.main.schema_chip}</span>
        <span class="rounded-full border border-border bg-white/70 px-4 py-2">{l.main.locale_chip}</span>
        <span class="rounded-full border border-border bg-white/70 px-4 py-2">{l.main.generated_chip}</span>
      </div>
    </div>

    <div class="relative">
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
        <p class="text-sm font-semibold uppercase text-primary">{l.main.proof_kicker}</p>
        <h2 class="mt-3 font-serif text-4xl font-semibold leading-tight sm:text-5xl">{l.main.proof_title}</h2>
      </div>
      <div class="grid gap-4 sm:grid-cols-2">
        {#each [l.main.feature_schema, l.main.feature_locale, l.main.feature_cldr, l.main.feature_web] as feature (feature)}
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
        <p id="codegen" class="text-sm font-semibold uppercase text-primary">{l.main.sample_kicker}</p>
        <h2 class="mt-3 font-serif text-4xl font-semibold sm:text-5xl">{l.main.sample_title}</h2>
      </div>
      <Button href="https://github.com/antepodeum/Linguini/blob/main/docs/reference.md" variant="ghost">
        {l.main.reference_cta}
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

  <section class="border-t border-border bg-foreground px-5 py-16 text-background sm:px-8">
    <div class="mx-auto grid max-w-7xl gap-8 lg:grid-cols-[0.8fr_1.2fr]">
      <div>
        <p class="text-sm font-semibold uppercase text-background/60">{l.main.playground_kicker}</p>
        <h2 class="mt-3 font-serif text-4xl font-semibold sm:text-5xl">{l.main.playground_title}</h2>
        <div class="mt-8 grid gap-3 text-sm text-background/75">
          <p class="flex items-center gap-2"><Route size={16} /> {l.main.route_label}: {localizedPlayground}</p>
          <p class="flex items-center gap-2"><Cookie size={16} /> {l.main.cookie_label}: LINGUINI_SITE_LOCALE={linguini.locale}</p>
        </div>
      </div>

      <div class="grid gap-4">
        <div class="grid gap-3 rounded-3xl border border-white/10 bg-white/[0.06] p-5 sm:grid-cols-2 lg:grid-cols-3">
          <label class="grid gap-2 text-sm">
            {l.main.count_label}
            <input class="rounded-2xl border border-white/10 bg-white/10 px-3 py-2" type="number" min="0" bind:value={count} />
          </label>
          <label class="grid gap-2 text-sm">
            {l.main.fruit_label}
            <select class="rounded-2xl border border-white/10 bg-white/10 px-3 py-2" bind:value={fruit}>
              <option value="apple">apple</option>
              <option value="pear">pear</option>
              <option value="orange">orange</option>
            </select>
          </label>
          <label class="grid gap-2 text-sm">
            {l.main.size_label}
            <select class="rounded-2xl border border-white/10 bg-white/10 px-3 py-2" bind:value={size}>
              <option value="small">small</option>
              <option value="big">big</option>
            </select>
          </label>
          <label class="grid gap-2 text-sm">
            {l.main.gender_label}
            <select class="rounded-2xl border border-white/10 bg-white/10 px-3 py-2" bind:value={gender}>
              <option value="male">male</option>
              <option value="female">female</option>
              <option value="neuter">neuter</option>
              <option value="other">other</option>
            </select>
          </label>
          <label class="grid gap-2 text-sm">
            {l.main.amount_label}
            <input class="rounded-2xl border border-white/10 bg-white/10 px-3 py-2" type="number" step="0.01" bind:value={amount} />
          </label>
          <label class="grid gap-2 text-sm">
            {l.main.date_label}
            <input class="rounded-2xl border border-white/10 bg-white/10 px-3 py-2" type="date" bind:value={dateInput} />
          </label>
        </div>

        <div class="rounded-3xl border border-white/10 bg-white/[0.08] p-5">
          <p class="mb-4 text-sm uppercase text-background/55">{l.main.localized_path_label}</p>
          <p class="mb-6 font-mono text-sm text-background/80">{localizedRoot}</p>
          <div class="grid gap-3">
            {#each playgroundLines as line (line)}
              <p class="rounded-2xl bg-background px-4 py-3 text-foreground">{line}</p>
            {/each}
          </div>
        </div>
      </div>
    </div>
  </section>
</main>
