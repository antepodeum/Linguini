<p align="center">
  <img src="https://github.com/antepodeum/.github/blob/main/logos/Linguini.png?raw=true" alt="Linguini" width="760" />
</p>

<p align="center">
  <img alt="Status" src="https://img.shields.io/badge/status-preview-orange?style=flat">
  <img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat">
</p>

No more JSON archaeology, string-key roulette, and runtime localization surprises.

**[Why Linguini](./docs/why.md)** · **[Language Reference](./docs/reference.md)** · **[Getting Started](./docs/getting-started.md)**

---

## Example

The schema is the public contract your app can call:

```lgs
// linguini/schema/checkout.lgs

enum Item { pasta, sauce }

/// Shown on the delivery confirmation card.
you_ordered(
  customer: String,
  item: Item,
  amount: Number,
  total: Number,
  delivery: Date,
)

cart_summary(amount: Number, total: Number)
```

The locale implements that contract with real language logic:

```lgl
// linguini/locale/checkout/ru.lgl

enum Gender { masculine, feminine, neuter, other }

impl Item {
  pasta {
    Gender = feminine

    form acc(Plural) {
      one => пасту
      few => пасты
      _   => паст
    }
  }

  sauce {
    Gender = masculine

    form acc(Plural) {
      one => соус
      few => соуса
      _   => соусов
    }
  }
}

form Rubles(Plural) {
  one => рубль
  few => рубля
  _   => рублей
}

form Pronoun(Plural, Gender) {
  one {
    masculine => Его
    feminine  => Её
    _         => Их
  }
  _ => Их
}

you_ordered = {customer}, вы заказали: {amount} {item.acc(amount)} на сумму {total} {Rubles(total)}. {Pronoun(amount, item.Gender)} доставка будет {delivery}.

cart_summary = В корзине {amount} {Product(amount)} на сумму {total} {Rubles(total)}
```

The app gets a generated native API:

```ts
import { configureLinguini } from "./generated/linguini";

const l = configureLinguini({ language: () => getRequestLocale() });

l.checkout.you_ordered("Artemy", "pasta", 3, 1290, "2026-05-17");
// → "Artemy, вы заказали: 3 пасты на сумму 1290 рублей. Их доставка будет 17.05.2026."

l.checkout.cart_summary(3, 1290);
// → "В корзине 3 товара на сумму 1290 рублей"
```

Typed arguments. Plural forms, grammatical gender, and case agreement. Analyzer diagnostics for everything that can go wrong. Generated native modules for each target runtime.

---

## Features

| Area      | Status                                                                                                             |
| --------- | ------------------------------------------------------------------------------------------------------------------ |
| Language  | `.lgs` schemas and `.lgl` locale implementations                                                                   |
| Grammar   | CLDR plural categories, forms, enum metadata, nested selectors, local helpers                                      |
| Analyzer  | missing implementations, invalid references, incomplete branches, diagnostics, quick fixes                         |
| Formatter | `.lgs` / `.lgl` formatting and `--check` mode                                                                      |
| LSP       | diagnostics, completion, hover, definition, references, symbols, semantic tokens, formatting, rename, code actions |
| Codegen   | TypeScript (JS, Rust, Kotlin, Swift, Go, Python, C# planned)                                                       |
| Editor    | VS Code extension                                                                                                  |

---

## Project layout

```
linguini.toml
linguini/
  schema/
    checkout.lgs
  locale/
    checkout/
      en.lgl
      ru.lgl
```

```toml
[project]
name           = "shop"
default_locale = "en"
locales        = ["en", "ru"]

[paths]
schema = "linguini/schema"
locale = "linguini/locale"

[targets.ts]
out         = "src/generated/linguini"
module      = "esm"
declaration = true
```

A schema file becomes a namespace. `checkout.lgs` → namespace `checkout`.
`locale/checkout/ru.lgl` → Russian implementation for that namespace.

---

## CLI

```bash
cargo install linguini-cli --version 0.1.0-alpha.3
```

```
linguini init      Create a project skeleton
linguini check     Report diagnostics across schema and locale files
linguini fix       Apply quick fixes — missing files, message stubs
linguini build     Build and write codegen outputs
linguini generate  Render sample output for all locales and enum variants
linguini format    Format .lgs and .lgl files
linguini lsp       Start the language server over stdio
```

---

## Development

```bash
cargo test --workspace
```

VS Code extension:

```bash
cd editors/vscode
npm install
npm run compile
npm run build:server
npm run open:dev
```

---

## License

Apache 2.0 — see [LICENSE](./LICENSE).

Built by [Antepod](https://github.com/antepodeum).
