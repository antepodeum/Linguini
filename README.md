# Linguini

Linguini is an experimental localization toolkit. Current repo state has Rust crates for syntax, config, IR lowering, CLDR plural support, and early TypeScript code generation.

## Current Workflow

Write schema files in `.lqs`:

```lqs
enum Fruit {
  apple
  pear
}

counted(count: Number, fruit: Fruit)
price(amount: Decimal, date: Date)
```

Write locale files in `.lgl`:

```lgl
form Fruit {
  apple {
    gen {
      one => яблока
      few => яблок
      many => яблок
      other => яблока
    }
  }
}

counted = В корзине {count} {fruit.gen(count)}
price = Цена {amount} на {date}
```

Generated TypeScript is organized as:

```txt
shared.ts
shared.d.ts
index.ts
index.d.ts
locales/ru.ts
locales/ru.d.ts
locales/en.ts
locales/en.d.ts
```

`shared.ts` contains runtime helpers shared by locales. Locale files export tree-shakable message functions, inline locale plural helpers, and a default locale object. `index.ts` imports locale modules and owns language selection. Declaration files mirror this tree.

Use generated TypeScript per request:

```ts
import { createLinguini } from "./generated";

export function render(requestLanguage: "ru") {
  const lgl = createLinguini(requestLanguage);
  return lgl.price(1200, "13.05.2026");
}
```

For frameworks where language is read from request context:

```ts
import { createLinguiniProvider } from "./generated";

const lgl = createLinguiniProvider({
  resolveLanguage: () => getRequestLanguage(),
});

export function renderPrice() {
  return lgl.price(1200, "13.05.2026");
}
```

## Development Commands

```sh
cargo test --workspace
bash scripts/validate-generated-ts.sh
./scripts/check-file-size.sh
```

TypeScript generation is currently covered by golden snapshots under `tests/fixtures/golden/snapshots/ts`.
