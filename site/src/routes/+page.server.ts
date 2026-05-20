import { highlightCode } from "$lib/server/highlight";

const heroLocaleCode = `form SizeWord(Size) {
  small => compact
  big => full-size
}

cart_summary = Cart has {count} {SizeWord(size)} {fruit.nom(count)}`;

const schemaExample = `enum Fruit {
  apple,
  pear
}

enum Size {
  small,
  big
}

type Money = Decimal @currency(code = "USD")

checkout(
  count: Number,
  fruit: Fruit,
  size: Size,
  total: Money
)`;

const localeExample = `impl Fruit {
  apple {
    form nom(Plural) {
      one => apple
      _ => apples
    }
  }

  pear {
    form nom(Plural) {
      one => pear
      _ => pears
    }
  }
}

form SizeWord(Size) {
  small => compact
  big => full-size
}

checkout = {count} {SizeWord(size)} {fruit.nom(count)}, {total}`;

const sveltekitExample = `<script lang="ts">
  import { l, setLocale } from '$lib/generated/linguini/svelte';
  import { locales } from '$lib/generated/linguini';
</script>

<a href="/checkout">
  {l.main.checkout(count, fruit, size, total)}
</a>

{#each locales as locale}
  <button
    type="button"
    onclick={() => setLocale(locale)}
  >
    {locale}
  </button>
{/each}`;

export async function load() {
  return {
    codeBlocks: {
      heroLocale: await highlightCode(heroLocaleCode, "linguini-locale"),
      schema: await highlightCode(schemaExample, "linguini-schema"),
      locale: await highlightCode(localeExample, "linguini-locale"),
      sveltekit: await highlightCode(sveltekitExample, "svelte"),
    },
  };
}
