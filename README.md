# Linguini

**The localization language for products that outgrow strings.**

Linguini is a compiled localization system for modern apps. It replaces JSON key bags, runtime message parsers, plural hacks, unsafe placeholders, manual formatter wiring, and framework glue with one typed language pipeline.

The goal is direct: **solve the i18n stack as one system** — schema, translations, grammar, formatting, diagnostics, generated code, editor tooling, and app integration.

> Experimental status: parts of this are implemented today; parts are the roadmap. The concept is the important thing: i18n should be compiled, typed, and analyzable.

---

## Why i18n is broken

Most localization stacks start like this:

```json
{
  "cart.count": "{count} items"
}
```

Then the real product needs plural rules, gender, grammatical case, typed placeholders, currency/date formatting, fallbacks, missing-translation checks, route localization, editor support, generated types, and framework integration.

At that point localization is no longer “strings”. It is a small programming language hiding inside translation files.

Linguini makes that language explicit.

---

## The idea

Linguini splits localization into two layers:

- **`.lgs` schema** — the public localization contract your app can call.
- **`.lgl` locale** — the implementation for a real human language.

The compiler checks both and generates native TypeScript.

No runtime parsing of translation files. No unchecked keys. No guessing whether `{count}` exists. No shipping missing plural branches by accident.

---

## Project shape

A `.lgs` file becomes a namespace. The matching `.lgl` files live in a folder with the same namespace.

```txt
linguini.toml
linguini/
  schema/
    shop.lgs
    checkout.lgs

  locale/
    shop/
      en.lgl
      ru.lgl

    checkout/
      en.lgl
      ru.lgl
```

Mapping:

```txt
linguini/schema/shop.lgs          -> namespace shop
linguini/locale/shop/ru.lgl       -> ru implementation for shop

linguini/schema/shop/delivery.lgs -> namespace shop.delivery
linguini/locale/shop/delivery/ru.lgl
```

Config:

```toml
[project]
name = "shop"
default_locale = "en"
locales = ["en", "ru"]

[paths]
schema = "linguini/schema"
locale = "linguini/locale"

[targets.ts]
out = "src/generated/linguini"
module = "esm"
declaration = true
```

---

## Schema: the contract

`linguini/schema/shop.lgs`

```lgs
enum Fruit {
  apple
  pear
  orange
}

enum Size {
  small
  big
}

type Money = Decimal @currency
type ShortDate = Date @date(style = "short")

/// Displayed on the product delivery confirmation card.
delivery(fruit: Fruit, size: Size, count: Number)

counted(count: Number, fruit: Fruit)
price(amount: Money, date: ShortDate)

email_input {
  label
  placeholder
  aria
}
```

The schema defines what the application may ask for: messages, arguments, enums, docs, grouped messages, and formatting intent.

---

## Locale: the language logic

`linguini/locale/shop/ru.lgl`

```lgl
enum Gender { male, female, neuter, other }

impl Fruit {
  apple {
    Gender = neuter

    form nom(Plural) {
      one => яблоко
      few => яблока
      _ => яблок
    }
  }

  pear {
    Gender = female

    form nom(Plural) {
      one => груша
      few => груши
      _ => груш
    }
  }
}

form Delivered(Plural, Gender) {
  one {
    male   => Доставлен
    female => Доставлена
    neuter => Доставлено
    _      => Доставлено
  }
  _ => Доставлены
}

delivery = {Delivered(count, fruit.Gender)} {fruit.nom(count)}
counted = В корзине {count} {fruit.nom(count)}
price = Цена {amount @currency(code = "RUB")} на {date @date(style = "short")}

email_input {
  label = Email
  placeholder = name@example.com
  aria = Адрес электронной почты
}
```

This is the main point: Linguini does not pretend every language is English with different words. A locale can describe its own grammar, private enums, forms, attributes, and formatter overrides.

---

## Generated TypeScript

Linguini generates code like this:

```txt
src/generated/linguini/
  index.ts
  index.d.ts
  shared.ts
  shared.d.ts
  locales/
    en.ts
    en.d.ts
    ru.ts
    ru.d.ts
```

Generated declarations are typed:

```ts
export type Fruit = "apple" | "pear" | "orange";
export type Size = "small" | "big";
export type Money = number;
export type ShortDate = string;

export declare function delivery(
  fruit: Fruit,
  size: Size,
  count: number,
): string;

export declare function counted(count: number, fruit: Fruit): string;
export declare function price(amount: Money, date: ShortDate): string;
```

Application code uses functions, not string keys:

```ts
import { configureLinguini } from "./generated/linguini";

const lgl = configureLinguini({
  language: () => getRequestLocale(),
});

lgl.delivery("apple", "small", 5);
lgl.counted(5, "apple");
lgl.price(1200, "2026-05-14");
```

---

## What it solves

Linguini is designed to remove the usual split between translation files, type generation, plural libraries, formatter helpers, runtime locale state, editor tooling, and framework adapters.

It brings together:

- schema-first message contracts;
- typed generated TypeScript;
- CLDR plural rules;
- grammatical forms and agreement;
- locale-private enums, attributes, and functions;
- currency/date/number formatter annotations;
- missing-message and missing-locale diagnostics;
- default-locale fallback;
- generated runtime facade;
- formatter, LSP, and VS Code support;
- future Vite/SvelteKit/runtime integration;
- future reusable localization packages.

The goal is not a better translation file. The goal is a complete i18n compiler.

---

## Why not Fluent?

Fluent is a great message format. Linguini targets the whole product localization pipeline.

| Area                 | Fluent-style systems                         | Linguini                                            |
| -------------------- | -------------------------------------------- | --------------------------------------------------- |
| Public API           | Message files are the source of truth        | Separate `.lgs` schema contract                     |
| App calls            | Usually key/string oriented or extra typegen | Generated typed functions by default                |
| Grammar              | Expressive messages                          | First-class forms, enum attributes, local functions |
| Project layout       | File/message oriented                        | Namespace-aware schema/locale mapping               |
| Missing locale files | Tooling-dependent                            | Analyzer-level project check                        |
| Formatters           | Usually runtime/library wiring               | Source-level annotations like `@currency`           |
| Runtime              | Message resolution/parsing model             | Native generated target code                        |
| Framework glue       | Separate ecosystem concern                   | Planned as part of the Linguini stack               |

Fluent helps write better localized messages.

Linguini aims to make localization a compiled, typed, end-to-end application layer.

---

## Status

Already present or partially present:

- `.lgs` / `.lgl` syntax;
- `linguini.toml` project config;
- path-derived namespaces;
- CLI: `init`, `check`, `fix`, `build`, `generate`, `format`, `lsp`;
- TypeScript code generation with `.d.ts`;
- generated locale modules and facade;
- CLDR plural generation;
- currency/date formatter calls;
- diagnostics for missing locale files and messages;
- formatter prototype;
- LSP and VS Code prototypes.

Roadmap:

- production-grade formatter;
- deeper semantic analysis;
- safe rename and quick fixes;
- missing branch generation;
- richer formatter annotations;
- runtime integration library;
- Vite plugin;
- Svelte/SvelteKit adapter;
- localized routing helpers;
- rich text component interpolation;
- package registry and reusable grammar packs;
- JS and Rust backends.

---

## Commands

```bash
linguini init
linguini check
linguini fix --all
linguini build
linguini generate
linguini format
linguini lsp
```

---

## Philosophy

Localization should be compiled, typed, analyzable, and editor-aware.

Languages have grammar. Products have contracts. Applications need generated code. Teams need diagnostics before release.

Linguini puts all of that into one system.
