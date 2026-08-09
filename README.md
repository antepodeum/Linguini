<p align="center">
  <img src="https://github.com/antepodeum/.github/blob/main/logos/Linguini.png?raw=true" alt="Linguini" width="760" />
</p>

<p align="center">
  <img alt="Status" src="https://img.shields.io/badge/status-preview-orange?style=flat">
  <img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat">
  <a href="https://marketplace.visualstudio.com/items?itemName=antepod.linguini-vscode"><img alt="VS Code Marketplace" src="https://img.shields.io/visual-studio-marketplace/v/antepod.linguini-vscode?label=VS%20Code&style=flat"></a>
  <a href="https://open-vsx.org/extension/antepod/linguini-vscode"><img alt="Open VSX" src="https://img.shields.io/open-vsx/v/antepod/linguini-vscode?label=Open%20VSX&style=flat"></a>
</p>

No more JSON archaeology, string-key roulette, and runtime localization surprises.

> Linguini is in a very early stage of development. APIs, generated output, and package boundaries can change quickly while the core model is still being hardened. [Check repository state](./REPOSITORY_STATE.md)

**[Why Linguini](./docs/why.md)** · **[Language Reference](./docs/reference.md)** · **[Getting Started](./docs/getting-started.md)** · **[Web/SvelteKit](./docs/web-sveltekit.md)**

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

cart_summary(amount: Number, item: Item, total: Number)
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

cart_summary = В корзине {amount} {item.acc(amount)} на сумму {total} {Rubles(total)}
```

The app gets a generated native API:

```ts
import { configureLinguini } from "./generated/linguini";

const l = configureLinguini({ language: () => getRequestLocale() });

l.checkout.you_ordered("Artemy", "pasta", 3, 1290, "2026-05-17");
// → "Artemy, вы заказали: 3 пасты на сумму 1290 рублей. Их доставка будет 17.05.2026."

l.checkout.cart_summary(3, "pasta", 1290);
// → "В корзине 3 пасты на сумму 1290 рублей"
```

The generated `l` object mirrors the nested schema paths. Parameterless messages
are values (`l.checkout.title`); messages with parameters are callable leaves
(`l.checkout.you_ordered(...)`). With the Svelte/Vite bundler target, static
paths are transformed to imports of the exact per-message modules they use.

Typed arguments. Plural forms, grammatical gender, and case agreement. Analyzer diagnostics for
schema/locale mismatches, invalid references, missing branches in resolved enum/`Plural`
dispatches, and type mismatches in locale calls whose argument types can be resolved. Generated
modules that can be used from app code.

---

## Features

| Area      | Status                                                                                                             |
| --------- | ------------------------------------------------------------------------------------------------------------------ |
| Language  | `.lgs` schemas and `.lgl` locale implementations                                                                   |
| Grammar   | CLDR plural categories, forms, enum metadata, nested selectors, local helpers                                      |
| Analyzer  | missing implementations, invalid references, resolved branch coverage and locale-call types                        |
| Formatter | `.lgs` / `.lgl` formatting and `--check` mode                                                                      |
| LSP       | diagnostics, completion, hover, definition, references, symbols, semantic tokens, formatting, rename, code actions |
| Codegen   | TypeScript and generated Svelte 5/SvelteKit adapters today; other targets are not part of the current release      |
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
declaration = true
```

Namespaces come from paths relative to the configured source roots:

| Schema file | Locale file for `ru` | Filesystem namespace |
| --- | --- | --- |
| `schema/checkout.lgs` | `locale/checkout/ru.lgl` | `checkout` |
| `schema/shop/checkout.lgs` | `locale/shop/checkout/ru.lgl` | `shop.checkout` |

The schema file stem is part of its namespace; the locale file stem is the locale
tag and is not. Nested source groups extend the same canonical path, so
`receipt { title }` in `schema/shop/checkout.lgs` is generated as
`l.shop.checkout.receipt.title`.

`project.name` identifies the project and seeds web-persistence defaults. It does
not prefix schema symbols or the generated `l` API. Moving or renaming a schema
file therefore changes its public namespace and requires moving the matching
locale directory. See the [namespace contract](./docs/reference.md#namespaces-and-qualified-paths).

---

## CLI

Linguini CLI must be installed. The VS Code extension and generated tooling use
the `linguini` command by default and do not include a bundled binary.

```bash
cargo install linguini-cli --version 0.1.0-alpha.4
```

Preview builds use vendored CLDR JSON data from the repository. If that data is
missing from a source archive, building the CLDR support crate may need network
access to fetch the pinned CLDR source.

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

The root runner provides repository-wide local verification:

```bash
pnpm test:quick   # Rust formatting plus focused Vite and CLI tests
pnpm test:full    # Rust, Vite, CLI, VS Code, and real-site gates
pnpm test:site    # generate, test, check, build, and inspect the site graph
```

VS Code extension:

```bash
cd editors/vscode
npm install
npm run compile
npm run open:dev
```

---

## License

Apache 2.0 — see [LICENSE](./LICENSE).

Built by [Antepod](https://github.com/antepodeum).
