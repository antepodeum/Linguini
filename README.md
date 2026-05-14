# Linguini

**A compiled localization language for applications that need grammar, type safety, and zero runtime parsing.**

Linguini is an experimental localization ecosystem built around two small DSLs:

- `.lgs` — a schema language that describes the public localization contract of your product.
- `.lgl` — a locale language that implements translations, grammar, pluralization, forms, and formatting.

Instead of shipping JSON dictionaries, ICU strings, and runtime parsers to production, Linguini compiles localization sources into native application code: TypeScript first, with JavaScript and Rust targets planned.

```txt
.lgs schema + .lgl locale files
        ↓
static analysis + CLDR plural rules + formatter metadata
        ↓
typed, tree-shakable generated application code
```

> Status: Linguini is experimental and under active development. Some features below are implemented today, some are partially implemented, and some are part of the roadmap. The project intentionally documents the full product vision, not only the current prototype surface.

---

## Why Linguini exists

Modern localization is still too stringly typed.

Common localization stacks often make developers choose between:

- simple key-value dictionaries that cannot model grammar;
- ICU MessageFormat strings that are powerful but hard to type-check, refactor, and autocomplete;
- runtime parsers that add bundle weight and failure modes;
- translation files that drift from application code until production catches the mismatch;
- generated APIs that still force locale plumbing through every call site.

Linguini takes a compiler-first approach.

You describe what messages exist, what parameters they accept, which grammatical categories matter, and how each locale realizes those messages. The compiler then validates the project and emits ordinary code that your app can import directly.

---

## Core idea

A localization system should know more than “there is a string at this key”.

It should know:

- which messages are public API;
- which arguments a message requires;
- whether an argument is a number, date, decimal, enum, or custom scalar;
- which enum values must be translated;
- which plural categories exist for the target locale;
- which grammatical form of a noun or adjective is needed;
- whether a translation is missing;
- whether a placeholder is unknown;
- whether a function call has the wrong arity;
- whether generated code can be tree-shaken safely.

Linguini makes that information explicit.

---

## A quick example

Define the public contract in `.lgs`:

```lgs
/// Displayed in the cart summary.
counted(count: Number, fruit: Fruit)

/// Displayed on the delivery confirmation card.
delivery(fruit: Fruit, size: Size, count: Number)

enum Fruit {
  apple
  pear
}

enum Size {
  small
  big
}
```

Implement a Russian locale in `.lgl`:

```lgl
enum Gender { male, female, neuter, other }

impl Fruit {
  apple {
    Gender = neuter

    form nom(Plural) {
      one => яблоко
      few => яблока
      _   => яблок
    }

    form gen(Plural) {
      one => яблока
      _   => яблок
    }
  }

  pear {
    Gender = female

    form nom(Plural) {
      one => груша
      few => груши
      _   => груш
    }

    form gen(Plural) {
      one => груши
      _   => груш
    }
  }
}

form SizeAdj(Size, Plural, Gender) {
  small {
    one {
      male   => маленький
      female => маленькая
      neuter => маленькое
      _      => маленький
    }
    _ => маленьких
  }

  big {
    one {
      male   => большой
      female => большая
      neuter => большое
      _      => большой
    }
    _ => больших
  }
}

counted = В корзине {count} {fruit.gen(count)}
delivery = Доставлено {count} {SizeAdj(size, count, fruit.Gender)} {fruit.gen(count)}
```

Use generated TypeScript in application code:

```ts
import { createLinguini } from "./generated";

const lgl = createLinguini("ru");

lgl.counted(3, "apple");
lgl.delivery("pear", "small", 1);
```

The application calls normal typed functions. No runtime DSL parser is required.

---

## What makes it different

### 1. Localization is a typed contract

`.lgs` files define the public API of your localized product.

```lgs
price(amount: Money)
type Money = Decimal @currency
```

A message is not “some string key”. It is a function signature. Generated code can expose it as a real typed function.

Planned developer experience:

- autocomplete for message names;
- autocomplete for parameters;
- hover docs from schema comments;
- generated `.d.ts` declarations;
- safe refactors for schema symbols;
- diagnostics when locale files drift from schema.

### 2. Locales can model grammar

Many languages require grammatical agreement. Linguini treats that as a first-class localization problem.

```lgl
form Delivered(Plural, Gender) {
  one {
    male   => Доставлен
    female => Доставлена
    neuter => Доставлено
    _      => Доставлено
  }
  _ => Доставлены
}
```

Forms and local functions let each locale express its own grammatical logic without leaking that complexity into application code.

### 3. CLDR plural rules are compiled, not guessed

`Plural` is a built-in grammatical dimension. A numeric argument can be passed directly to a `Plural` parameter:

```lgl
counted = {count} {fruit.nom(count)}
```

The compiler/runtime uses locale-specific CLDR plural rules. The project roadmap includes compiled CLDR tables for plural categories, number formatting, currency formatting, date formatting, time formatting, and units.

### 4. Formatting is part of the language

Schema-level defaults:

```lgs
type Money = Decimal @currency
type ShortDate = Date @date(style = "short")
```

Locale-level overrides:

```lgl
price = Цена: {amount @currency(code = "USD")}
created = Создано: {date @date(style = "long")}
```

The goal is to compile only the formatters actually used by generated messages.

### 5. Generated code is native application code

Linguini is designed around static output:

- no runtime Linguini file parsing;
- no runtime translation lookup DSL;
- deterministic generated files;
- tree-shakable message functions;
- locale-specific modules;
- generated type declarations;
- generated facade/provider for locale selection.

Example generated shape:

```ts
import { createLinguiniProvider } from "./generated";

export const lgl = createLinguiniProvider({
  resolveLanguage: () => getRequestLanguage(),
});

lgl.price(1200);
lgl.cart.counted(3, "apple");
```

Application code should not need to pass `locale` into every message call.

### 6. Editor tooling is part of the product

The roadmap includes a complete language tooling story:

- lexer and parser with spans;
- CLI diagnostics;
- formatter;
- LSP diagnostics;
- hover;
- completion;
- go to definition;
- references;
- rename;
- semantic tokens;
- quick fixes;
- missing branch generation;
- missing message generation;
- VS Code extension;
- optional tree-sitter grammar.

Linguini is meant to feel like a small programming language, not a pile of loose translation files.

---

## Project layout

A Linguini project contains a config file and localization sources:

```txt
linguini.toml
linguini/
  schema/
    shop/
      types.lgs
      delivery.lgs
  locale/
    ru.lgl
    en.lgl
    shop/
      ru.lgl
      en.lgl
```

Example config:

```toml
[project]
name = "my-app"
default_locale = "ru"
locales = ["ru", "en"]

[paths]
schema = "linguini/schema"
locale = "linguini/locale"
```

Schema namespaces are derived from paths:

```txt
linguini/schema/shop/delivery.lgs → shop.delivery
```

Locale scope is inherited top-down:

```txt
linguini/locale/ru.lgl
linguini/locale/shop/ru.lgl
linguini/locale/shop/delivery/ru.lgl
```

That lets teams put shared locale helpers at parent levels and keep feature-specific translations close to the feature.

---

## Language overview

### Schema files: `.lgs`

Schema files describe public declarations.

```lgs
/// Product fruit shown to users.
enum Fruit {
  apple
  pear
  orange
}

/// Price formatted with locale currency rules.
type Money = Decimal @currency

/// Cart item count message.
counted(count: Number, fruit: Fruit)

email_input {
  label
  placeholder
  aria
}
```

Supported and planned schema concepts:

- public enums;
- public message signatures;
- grouped messages;
- custom scalar aliases;
- formatter annotations;
- doc comments;
- generated declaration docs;
- public structs on the roadmap.

### Locale files: `.lgl`

Locale files implement translations and locale-specific grammar.

```lgl
label = Email
price = Цена: {amount @currency(code = "USD")}
body = """
Здравствуйте, {name}

Ваш заказ доставлен.
"""
```

Locale files can contain:

- message implementations;
- grouped message implementations;
- local enums;
- enum implementations;
- typed forms;
- local functions;
- aliases and imports on the roadmap;
- formatter overrides;
- package imports on the roadmap.

### Forms

Forms express grammatical variation.

```lgl
form NounCase(Plural) {
  one => товар
  few => товара
  _   => товаров
}
```

Nested forms express agreement across multiple axes:

```lgl
form ButtonText(Plural, Gender) {
  one {
    male   => Выбран
    female => Выбрана
    neuter => Выбрано
    _      => Выбрано
  }
  _ => Выбраны
}
```

### Local functions

Local functions are private helpers inside a locale.

```lgl
fn DeliveryNote(item: String, Plural, Gender) {
  one {
    female => Доставлена {item}
    _      => Доставлен {item}
  }
  _ => Доставлены {item}
}
```

Use `form` when the result depends only on grammatical categories. Use `fn` when the helper also interpolates text parameters.

### Placeholders

Supported placeholder style:

```lgl
{name}
{fruit.nom}
{fruit.nom(count)}
{fruit.Gender}
{SizeAdj(size, count, fruit.Gender)}
{amount @currency(code = "USD")}
{date @date(style = "long")}
```

The parser builds syntax. The analyzer resolves names, types, arity, plural arguments, and formatter compatibility.

---

## Generated TypeScript vision

The initial production target is TypeScript.

Generated output is designed to be:

- ESM-first;
- type-safe;
- deterministic;
- tree-shakable;
- split by locale;
- friendly to framework adapters;
- compatible with generated `.d.ts` files;
- free of runtime `.lgs`/`.lgl` parsing.

Example output structure:

```txt
generated/
  shared.ts
  shared.d.ts
  index.ts
  index.d.ts
  locales/
    ru.ts
    ru.d.ts
    en.ts
    en.d.ts
```

Planned generated API:

```ts
import { lgl, createLinguini, createLinguiniProvider } from "./generated";

const ru = createLinguini("ru");
ru.price(1200);

const current = createLinguiniProvider({
  resolveLanguage: () => getLocaleFromRequest(),
});

current.delivery("apple", "big", 5);
```

---

## Runtime integration vision

Linguini is designed to support a framework-aware TypeScript runtime layer.

Planned runtime capabilities:

- `locales` and `baseLocale` metadata;
- `getLocale()` / `setLocale()` or equivalent locale source API;
- server-safe per-request locale context;
- `AsyncLocalStorage` support for Node servers;
- edge/serverless-friendly fallback mode;
- URL localization helpers;
- locale redirect helpers;
- text direction detection;
- rich-text component interpolation;
- Vite plugin for watching `.lgs`, `.lgl`, and `linguini.toml`;
- Svelte/SvelteKit adapter;
- static-site friendly locale switcher links;
- `%lang%` and `%dir%` replacement for app shells.

Default locale detection strategy is planned as:

```ts
["url", "cookie", "preferredLanguage", "localStorage", "baseLocale"];
```

---

## CLI vision

Core commands:

```bash
linguini init
linguini check
linguini build
linguini generate
linguini format
linguini fill
linguini status
linguini lsp
```

CLDR commands:

```bash
linguini cldr fetch
linguini cldr update
linguini cldr status
linguini cldr clean
```

Later package-management commands:

```bash
linguini package init
linguini package publish
linguini package add
linguini package update
linguini package audit
```

### What these commands are for

| Command    | Purpose                                                                    |
| ---------- | -------------------------------------------------------------------------- |
| `init`     | Create a valid Linguini project structure.                                 |
| `check`    | Run analysis without generating application code.                          |
| `build`    | Analyze and generate target code.                                          |
| `generate` | Render sample outputs for configured locales and representative arguments. |
| `format`   | Format `.lgs` and `.lgl` sources.                                          |
| `fill`     | Create missing locale files and message stubs.                             |
| `status`   | Show locale completion and missing coverage.                               |
| `lsp`      | Start the language server for editors.                                     |

---

## Editor tooling vision

Linguini should be pleasant to write by hand.

Planned and partially implemented editor features:

- syntax highlighting for `.lgs` and `.lgl`;
- semantic highlighting;
- parser diagnostics;
- semantic diagnostics;
- hover documentation from schema comments;
- message and placeholder completion;
- form property completion;
- local enum branch completion;
- go to definition;
- find references;
- document symbols;
- workspace symbols;
- format document;
- rename schema symbols;
- quick fixes for missing branches/messages/forms;
- VS Code extension;
- optional tree-sitter grammar.

Examples of intended quick fixes:

- add missing enum branches;
- add missing `other` branch;
- add missing locale message;
- add missing form variant;
- add explicit plural argument;
- create missing locale file.

---

## Packages vision

Package support is a later-stage feature.

A Linguini package may provide:

- schema types;
- schema enums;
- locale forms;
- local functions;
- formatter presets;
- CLDR extensions;
- reusable dictionaries;
- domain-specific terminology.

Example use cases:

- common product nouns;
- Russian food forms;
- country and currency display names;
- grammatical adjective sets;
- ecommerce terminology;
- design-system UI copy.

Planned import style:

```lgl
import @linguini/ru-food/FruitForms
```

Package content must remain analyzable before code generation.

---

## Current implementation highlights

The repository already contains the Rust workspace shape for the full compiler pipeline:

```txt
crates/
  linguini-cli/
  linguini-config/
  linguini-syntax/
  linguini-schema/
  linguini-locale/
  linguini-analyzer/
  linguini-cldr/
  linguini-cldr-macros/
  linguini-ir/
  linguini-codegen-ts/
  linguini-codegen-js/
  linguini-codegen-rust/
  linguini-format/
  linguini-lsp/
  linguini-package/
  linguini-test-support/
```

Implemented or partially implemented areas include:

- project config parsing;
- schema and locale path discovery;
- path-derived namespaces;
- top-down locale scope collection;
- lexer with source spans;
- code/raw-text/multiline/placeholder lexer modes;
- parser for schema and locale syntax;
- parser recovery;
- schema and locale AST models;
- CLDR plural support groundwork;
- IR and TypeScript codegen groundwork;
- formatter crate;
- LSP crate;
- VS Code extension skeleton;
- test fixtures and snapshot-style validation;
- CI-oriented scripts for generated output, file-size checks, spec gates, and coverage.

---

## Roadmap

### Stage 1 — Project model and discovery

- `linguini.toml` parser;
- schema and locale file discovery;
- namespace derivation;
- top-down locale scope chain;
- `linguini init`.

### Stage 2 — Syntax parser

- lexer modes;
- `.lgs` parser;
- `.lgl` parser;
- recoverable parse errors;
- spans for diagnostics and LSP;
- golden syntax fixtures.

### Stage 3 — Semantic analyzer

- unknown type diagnostics;
- duplicate declaration diagnostics;
- missing locale message diagnostics;
- unknown placeholder diagnostics;
- wrong arity diagnostics;
- invalid enum selector diagnostics;
- missing branch diagnostics;
- cyclic reference detection;
- formatter compatibility checks.

### Stage 4 — CLDR support

- CLDR plural rule ingestion;
- compiled plural functions;
- locale-specific categories;
- number/date/currency formatting data;
- cache commands;
- no implicit downloads during normal builds.

### Stage 5 — Code generation

- TypeScript output;
- `.d.ts` declarations;
- locale modules;
- generated facade;
- grouped messages as nested objects;
- deterministic output;
- tree-shaking support.

### Stage 6 — Runtime integration

- provider contract;
- request-aware locale resolution;
- Vite plugin;
- Svelte/SvelteKit adapter;
- localized URLs;
- rich text interpolation;
- text direction helpers.

### Stage 7 — Formatter

- stable output;
- idempotent formatting;
- comment preservation;
- structural whitespace normalization;
- match/form arm alignment;
- configurable enum sorting;
- max line width;
- no semantic changes.

### Stage 8 — LSP and editor support

- diagnostics;
- completion;
- hover;
- semantic tokens;
- formatting;
- definition/reference navigation;
- quick fixes;
- VS Code extension polish.

### Stage 9 — Locale management

- `fill` missing messages;
- `status` completion reports;
- generated sample matrices;
- translation workflow helpers.

### Stage 10 — Formatting data

- decimal formatting;
- percent formatting;
- currency formatting;
- date formatting;
- time formatting;
- date-time formatting;
- later unit formatting.

### Stage 11 — Packages

- package manifests;
- lockfiles;
- explicit imports;
- semantic versioning;
- registry workflows;
- package analysis before codegen.

---

## Development

Build and test the workspace:

```bash
cargo test --workspace
```

Validate generated TypeScript snapshots:

```bash
bash scripts/validate-generated-ts.sh
```

Run repository checks:

```bash
./scripts/check-file-size.sh
./scripts/check-spec-gates.sh
```

Develop the VS Code extension:

```bash
cd editors/vscode
npm install
npm run compile
```

---

## Design principles

### Compile-time over runtime

Localization files should be validated and compiled before production. Generated apps should not discover localization errors by parsing DSL files at runtime.

### Types over string keys

Messages should behave like typed functions. Refactors should be possible. Missing arguments should be compile-time or editor-time errors.

### Locale grammar belongs in locale files

Application code should not know how Russian adjectives agree with nouns, or how Polish plural categories branch. Locale authors should express that in the locale language.

### Generated code should look boring

The output should be plain target-language code: imports, functions, objects, lookup tables, and type declarations.

### Tooling is not optional

A localization language needs formatter, diagnostics, LSP, syntax highlighting, and quick fixes. Otherwise it becomes another fragile config format.

### No hidden runtime magic

Locale selection can be abstracted behind providers, but generated message modules should remain explicit and analyzable.

---

## Comparison with common approaches

| Approach                      | Strength                      | Weakness Linguini targets                                              |
| ----------------------------- | ----------------------------- | ---------------------------------------------------------------------- |
| JSON dictionaries             | Simple and familiar           | Weak typing, poor grammar modeling, runtime key drift                  |
| ICU MessageFormat             | Powerful plural/select syntax | Harder refactors, stringly typed placeholders, runtime parser pressure |
| Hand-written locale functions | Maximum control               | Repetition, inconsistent conventions, no shared analyzer               |
| Framework-only i18n plugins   | Easy integration              | Often tied to runtime lookup model and framework-specific APIs         |
| Linguini                      | Typed DSL + generated code    | Requires compiler/tooling maturity                                     |

---

## Example: from linguistic complexity to application simplicity

Locale file:

```lgl
form Delivered(Plural, Gender) {
  one {
    male   => Доставлен
    female => Доставлена
    neuter => Доставлено
    _      => Доставлено
  }
  _ => Доставлены
}

delivery = {Delivered(count, fruit.Gender)} {count} {fruit.gen(count)}
```

Application code:

```ts
lgl.delivery("apple", 3);
```

The app does not branch on Russian grammar. The locale does.

---

## License

Apache-2.0

---

## One-line pitch

**Linguini is a compiler for localization: write typed language-aware localization sources, get fast native application code, editor tooling, and fewer production translation bugs.**
