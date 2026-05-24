<script lang="ts">
  import { base } from '$app/paths';
  import {
    ArrowRight,
    Code2,
    Cookie,
    Route,
  } from '@lucide/svelte';
  import CsharpIcon from '@iconify-svelte/skill-icons/cs';
  import AngularIcon from '@iconify-svelte/skill-icons/angular-dark';
  import AstroIcon from '@iconify-svelte/skill-icons/astro';
  import DartIcon from '@iconify-svelte/skill-icons/dart-dark';
  import ElixirIcon from '@iconify-svelte/skill-icons/elixir-dark';
  import FlutterIcon from '@iconify-svelte/skill-icons/flutter-dark';
  import GoIcon from '@iconify-svelte/skill-icons/golang';
  import JavaIcon from '@iconify-svelte/skill-icons/java-dark';
  import KotlinIcon from '@iconify-svelte/skill-icons/kotlin-dark';
  import NextIcon from '@iconify-svelte/skill-icons/nextjs-dark';
  import NodeIcon from '@iconify-svelte/skill-icons/nodejs-dark';
  import NuxtIcon from '@iconify-svelte/skill-icons/nuxtjs-dark';
  import PhpIcon from '@iconify-svelte/skill-icons/php-dark';
  import PythonIcon from '@iconify-svelte/skill-icons/python-dark';
  import ReactIcon from '@iconify-svelte/skill-icons/react-dark';
  import RustIcon from '@iconify-svelte/skill-icons/rust';
  import SolidIcon from '@iconify-svelte/skill-icons/solidjs-dark';
  import SvelteIcon from '@iconify-svelte/skill-icons/svelte';
  import SwiftIcon from '@iconify-svelte/skill-icons/swift';
  import TypeScriptIcon from '@iconify-svelte/skill-icons/typescript';
  import VueIcon from '@iconify-svelte/skill-icons/vuejs-dark';
  import ZigIcon from '@iconify-svelte/skill-icons/zig-dark';
  import Button from '$lib/components/button.svelte';
  import CodeBlock from '$lib/components/code-block.svelte';
  import { l, linguini, setLocale } from '$lib/generated/linguini/svelte';
  import { locales, type Locale } from '$lib/generated/linguini';
  import type { Fruit, Gender, Size } from '$lib/generated/linguini/locales/en';
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();

  let count = $state(3);
  let fruit = $state<Fruit>('apple');
  let size = $state<Size>('small');
  let gender = $state<Gender>('neuter');
  let amount = $state(1299.5);
  let dateInput = $state('2026-05-19');

  const dateValue = $derived(dateInput);
  const localizedRoot = $derived(linguini.localizeHref('/'));

  const nav = $derived([
    { id: 'why', label: l.main.nav_why },
    { id: 'codegen', label: l.main.nav_codegen },
    { id: 'language', label: l.main.nav_language },
    { id: 'web', label: l.main.nav_web }
  ]);

  const shippedTargets = $derived([
    {
      icon: TypeScriptIcon,
      title: l.main.codegen_ts_title,
      description: l.main.codegen_ts_desc
    },
    {
      icon: SvelteIcon,
      title: l.main.codegen_svelte_title,
      description: l.main.codegen_svelte_desc
    }
  ]);

  const plannedTargets = $derived([
    { icon: RustIcon, label: l.main.codegen_rust },
    { icon: SwiftIcon, label: l.main.codegen_swift },
    { icon: GoIcon, label: l.main.codegen_go },
    { icon: CsharpIcon, label: l.main.codegen_csharp },
    { icon: KotlinIcon, label: l.main.codegen_kotlin },
    { icon: PythonIcon, label: l.main.codegen_python },
    { icon: JavaIcon, label: 'Java' },
    { icon: PhpIcon, label: 'PHP' },
    { icon: DartIcon, label: 'Dart' },
    { icon: ElixirIcon, label: 'Elixir' },
    { icon: ZigIcon, label: 'Zig' },
    { icon: NodeIcon, label: 'Node.js' },
    { icon: ReactIcon, label: 'React' },
    { icon: VueIcon, label: 'Vue' },
    { icon: AngularIcon, label: 'Angular' },
    { icon: SolidIcon, label: 'Solid' },
    { icon: NextIcon, label: 'Next.js' },
    { icon: NuxtIcon, label: 'Nuxt' },
    { icon: AstroIcon, label: 'Astro' },
    { icon: FlutterIcon, label: 'Flutter' }
  ]);

  const fruitOptions = $derived([
    { value: 'apple' as const, label: l.main.fruit_apple_label },
    { value: 'pear' as const, label: l.main.fruit_pear_label },
    { value: 'orange' as const, label: l.main.fruit_orange_label }
  ]);

  const sizeOptions = $derived([
    { value: 'small' as const, label: l.main.size_small_label },
    { value: 'big' as const, label: l.main.size_big_label }
  ]);

  const genderOptions = $derived([
    { value: 'male' as const, label: l.main.gender_male_label },
    { value: 'female' as const, label: l.main.gender_female_label },
    { value: 'neuter' as const, label: l.main.gender_neuter_label },
    { value: 'other' as const, label: l.main.gender_other_label }
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
  <title>{l.main.hero_title} | typed localization language</title>
  <meta name="description" content={l.main.hero_copy} />
</svelte:head>

<div class="grain"></div>

<main class="relative overflow-visible">
  <nav class="sticky top-0 z-50 border-b border-border/70 bg-background/88 backdrop-blur-xl">
    <div class="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-4 px-5 py-4 sm:px-8">
    <a href={localizedRoot} class="flex items-center gap-3 font-semibold tracking-normal">
      <span class="brand-nav-mark">
        <img src="/icons/favicon.svg" alt="linguini logo" />
      </span>
      <span class="text-lg tracking-[0.28em]">{l.main.hero_title.toUpperCase()}</span>
    </a>

    <div class="hidden items-center gap-6 text-sm text-muted-foreground md:flex">
      {#each nav as item (item.id)}
        <a href={`#${item.id}`} class="transition hover:text-foreground">{item.label}</a>
      {/each}
    </div>

    <div class="locale-nav flex max-w-full flex-wrap items-center">
      <span class="px-3 text-xs text-muted-foreground">{l.main.locale_label}</span>
      {#each locales as item (item)}
        <button
          type="button"
          aria-pressed={linguini.locale === item}
          class={[
            'flex h-8 cursor-pointer appearance-none items-center bg-transparent px-2.5 text-xs font-medium transition sm:px-3 sm:text-sm',
            linguini.locale === item
              ? 'text-primary'
              : 'text-muted-foreground hover:text-foreground'
          ]}
          onclick={(event) => {
            event.preventDefault();
            chooseLocale(item);
          }}
        >
          {localeLabel(item)}
        </button>
      {/each}
    </div>
    </div>
  </nav>

  <section class="hero-shell mx-auto min-h-[calc(100vh-4.5rem)] max-w-7xl px-5 pb-20 pt-10 sm:px-8 lg:pt-14">
    <svg class="brand-waves" viewBox="0 0 1400 390" aria-hidden="true">
      <path d="M-80 76C92 72 156 162 323 154C486 147 528 82 686 65C856 47 948 87 1119 66C1267 48 1340 12 1480 -3" />
      <path d="M-90 151C71 134 158 225 314 225C486 225 531 157 692 134C862 111 952 138 1119 120C1278 103 1349 76 1484 58" />
      <path d="M-92 285C73 264 176 288 323 237C480 183 545 134 711 142C886 151 991 206 1152 246C1289 280 1362 282 1482 270" />
      <path d="M-82 340C92 321 179 350 336 300C489 252 563 202 731 216C901 230 997 286 1165 322C1300 351 1373 344 1480 323" />
    </svg>

    <div class="brand-stage">
      <div class="brand-panel">
        <div class="brand-icon" aria-hidden="true">
          <img src="/icons/favicon.svg" alt="linguini logo" />
        </div>

        <div class="brand-copy">
          <h1>{l.main.hero_title}</h1>
          <p class="brand-subtitle">{l.main.hero_tagline}</p>
        </div>
      </div>

      <div class="brand-code-card">
        <div class="mb-3 flex items-center justify-between gap-3 border-b border-border/70 pb-3">
          <span class="font-mono text-xs text-muted-foreground">linguini/locale/main/en.lgl</span>
        </div>
        <CodeBlock html={data.codeBlocks.heroLocale} />
      </div>
    </div>

    <div class="hero-explain">
      <div>
        <div class="term-line">
          <span>{l.main.hero_term}</span>
          <b>{l.main.hero_term_kind}</b>
          <p>{l.main.hero_term_hint}</p>
        </div>
        <p class="explain-copy">{l.main.hero_intro}</p>
      </div>

      <div class="mt-8 flex flex-col gap-3 sm:flex-row lg:mt-0">
        <Button href="https://github.com/antepodeum/Linguini/blob/main/docs/getting-started.md">
          {l.main.primary_cta}
          <ArrowRight size={17} />
        </Button>
        <Button href="https://github.com/antepodeum/Linguini" variant="secondary">
          <Code2 size={17} />
          {l.main.secondary_cta}
        </Button>
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
    <div class="pipeline">
      <article class="pipeline-panel">
        <p class="pipeline-label">schema</p>
        <CodeBlock html={data.codeBlocks.schema} />
      </article>

      <article class="pipeline-panel">
        <p class="pipeline-label">locale</p>
        <CodeBlock html={data.codeBlocks.locale} />
      </article>

      <article class="pipeline-panel pipeline-panel-output">
        <p class="pipeline-label">SvelteKit</p>
        <CodeBlock html={data.codeBlocks.sveltekit} />
      </article>
    </div>

    <div class="grid gap-4 lg:grid-cols-2">
      {#each shippedTargets as target (target.title)}
        {@const Icon = target.icon}
        <article class="group rounded-2xl border border-border bg-muted/25 p-6 transition hover:border-primary/30 hover:bg-muted/40">
          <div class="mb-5 flex items-start justify-between gap-4">
            <span class="flex h-12 w-12 items-center justify-center">
              <Icon width="40" height="40" />
            </span>
          </div>
          <h3 class="text-xl font-semibold">{target.title}</h3>
          <p class="mt-3 leading-7 text-muted-foreground">{target.description}</p>
        </article>
      {/each}
    </div>

    <div class="mt-10 planned-strip">
      <p class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">{l.main.codegen_planned_title}</p>
      <p class="mt-2 max-w-3xl text-sm text-muted-foreground">{l.main.codegen_planned_intro}</p>
      <div class="planned-icons" aria-label={l.main.codegen_planned_title}>
        {#each plannedTargets as target, index (target.label)}
          {@const Icon = target.icon}
          <span
            class="planned-icon"
            title={target.label}
            aria-label={target.label}
            style:z-index={plannedTargets.length - index}
          >
            <Icon width="34" height="34" />
          </span>
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
          <p class="mt-4 max-w-3xl text-lg leading-8 text-muted-foreground">{l.main.web_intro}</p>
        </div>
        <Button href="https://github.com/antepodeum/Linguini/blob/main/docs/web-sveltekit.md" variant="ghost">
          {l.main.primary_cta}
          <ArrowRight size={17} />
        </Button>
      </div>

      <div class="web-grid">
        <article>
          <p>01</p>
          <h3>{l.main.web_routing}</h3>
        </article>
        <article>
          <p>02</p>
          <h3>{l.main.web_cookie}</h3>
        </article>
        <article>
          <p>03</p>
          <h3>{l.main.web_fallback}</h3>
        </article>
        <article>
          <p>04</p>
          <h3>{l.main.web_reactivity}</h3>
        </article>
        <article>
          <p>05</p>
          <h3>{l.main.web_links}</h3>
        </article>
      </div>
    </div>
  </section>

  <section id="web" class="border-t border-border bg-muted/30 px-5 py-16 sm:px-8">
    <div class="mx-auto grid max-w-7xl gap-8 lg:grid-cols-[0.8fr_1.2fr]">
      <div>
        <p class="text-sm font-semibold uppercase tracking-wide text-primary">{l.main.playground_kicker}</p>
        <h2 class="mt-3 font-serif text-4xl font-semibold sm:text-5xl">{l.main.playground_title}</h2>
        <div class="mt-8 grid gap-3 text-sm text-muted-foreground">
          <p class="flex items-center gap-2"><Route size={16} class="text-primary" /> {l.main.route_label}: {localizedRoot}</p>
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
              {#each fruitOptions as option (option.value)}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </label>
          <label class="grid gap-2 text-sm">
            {l.main.size_label}
            <select class="rounded-2xl border border-border bg-muted/40 px-3 py-2 text-foreground" bind:value={size}>
              {#each sizeOptions as option (option.value)}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </label>
          <label class="grid gap-2 text-sm">
            {l.main.gender_label}
            <select class="rounded-2xl border border-border bg-muted/40 px-3 py-2 text-foreground" bind:value={gender}>
              {#each genderOptions as option (option.value)}
                <option value={option.value}>{option.label}</option>
              {/each}
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
              <p class="px-1 py-1 text-foreground">{line}</p>
            {/each}
          </div>
        </div>
      </div>
    </div>
  </section>
</main>
