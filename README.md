<p align="center">
  <img src="https://github.com/antepodeum/.github/blob/main/logos/Linguini.png?raw=true" alt="Linguini" width="760" />
</p>

<p align="center">
  <img alt="Status" src="https://img.shields.io/badge/status-preview-orange?style=flat">
  <img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat">
</p>

No more JSON archaeology, string-key roulette, and runtime localization surprises.

> [!NOTE]
> **Repository state:** this was built quickly, but it is built. The current goal is to finish the baseline end-to-end feature set. After that, the whole codebase will be manually reviewed, corrected, optimized, and cleaned up.

## Example

The schema is the public contract your app can call:

```lgs
// linguini/schema/checkout.lgs

enum Item {
  pasta
  sauce
  cookbook
}

type Money = Decimal @currency
type DeliveryDate = Date @date(style = "long")

order_ready(
  customer: String,
  item: Item,
  amount: Number,
  total: Money,
  delivery: DeliveryDate,
)

cart_summary(amount: Number, total: Money)
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

form Ready(Plural, Gender) {
  one {
    masculine => готов
    feminine  => готова
    neuter    => готово
    _         => готово
  }

  _ => готовы
}

form Product(Plural) {
  one => товар
  few => товара
  _   => товаров
}

order_ready = {customer}, ваш заказ {Ready(amount, item.Gender)}: {amount} {item.acc(amount)} на сумму {total @currency(code = "RUB")}. Доставка до {delivery @date(style = "long")}.

cart_summary = В корзине {amount} {Product(amount)} на сумму {total @currency(code = "RUB")}
```

The app receives a generated native API.

```ts
import { configureLinguini } from "./generated/linguini";

const l = configureLinguini({ language: () => getRequestLocale() });

l.checkout.order_ready("Artemy", "pasta", 3, 1290, "2026-05-17");
l.checkout.cart_summary(3, 1290);
```

What this gives you:

- typed message arguments instead of unchecked string keys;
- plural forms, enum attributes, grammatical agreement, and helper forms;
- source-level formatter annotations like `@currency` and `@date`;
- analyzer diagnostics for missing messages, unresolved references, bad branches, and malformed source;
- generated native modules for the target runtime.

```txt
.lgs schema + .lgl locale -> analyze -> IR -> compile -> native code
                             │
                             ├─ diagnostics / quick fixes
                             ├─ formatter
                             └─ LSP / editor tooling
```

---

## Features

| Area      | Status                                                                                                             |
| --------- | ------------------------------------------------------------------------------------------------------------------ |
| Language  | `.lgs` schemas and `.lgl` locale implementations                                                                   |
| Grammar   | CLDR plural categories, forms, enum metadata, nested selectors, local helpers                                      |
| Analyzer  | project checks, missing implementations, invalid references, incomplete branches, diagnostics, quick fixes         |
| Formatter | `.lgs` / `.lgl` formatting and `--check` mode                                                                      |
| LSP       | diagnostics, completion, hover, definition, references, symbols, semantic tokens, formatting, rename, code actions |
| Codegen   | TypeScript                                                                                                         |
| Editor    | VS Code extension support                                                                                          |

Linguini is intended to support many targets: TypeScript, JavaScript, Rust, Kotlin/JVM, Swift, Go, Python, C#/.NET, and other mainstream application ecosystems.

---

## Project layout

```txt
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

A schema file becomes a namespace:

```txt
linguini/schema/checkout.lgs     -> checkout
linguini/locale/checkout/ru.lgl  -> ru implementation for checkout
```

---

## CLI

### Installation

```bash
cargo install linguini-cli
```

```
Usage: linguini <COMMAND>

Commands:
  init      Create a Linguini project skeleton
  check     Parse configured schema and locale files and report diagnostics
  fix       Apply analyzer quick fixes such as missing locale files and message stubs
  build     Build the localization project and write configured codegen outputs
  generate  Generate rendered sample data for configured locales and enum variants
  format    Format `.lgs` and `.lgl` files
  lsp       Start the Linguini language server over stdio
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

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

Vite plugin:

```bash
cd plugins/vite
npm test
```
