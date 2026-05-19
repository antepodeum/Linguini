# Linguini Technical Specification

Version: 0.8

## 1. Goal

Linguini is a compiled localization ecosystem.

It provides:

- a schema DSL for public localization contracts: `.lgs`
- a locale DSL for localized implementations: `.lgl`
- a TOML project configuration file
- CLDR-based plural and formatting support
- static analysis and diagnostics
- project scaffolding and locale-management tooling
- syntax highlighting
- an LSP server
- code generation for TypeScript
- a TypeScript runtime integration library for locale detection, localized URLs, server middleware, and rich text interpolation
- later-stage package management for reusable localization resources

The output must be native target-language code. Generated applications must not parse Linguini files at runtime.

---

## 2. Technology Stack

### 2.1 Implementation language

Use Rust for all production components.

Reasons:

- fast CLI binaries
- strong typed compiler pipeline
- good diagnostics tooling
- good LSP ecosystem
- safe native code generation
- suitable for publishing a single binary

### 2.2 Rust crates

Core crates:

| Area                  | Crate                                  |
| --------------------- | -------------------------------------- |
| CLI                   | `clap`                                 |
| Config parsing        | `serde`, `toml`                        |
| JSON parsing          | `serde_json`                           |
| Lexer/parser          | `chumsky`                              |
| Diagnostics           | `ariadne`                              |
| Error types           | `thiserror`                            |
| Workspace paths       | `camino`                               |
| Glob discovery        | `ignore` or `globwalk`                 |
| Hash maps             | `indexmap`                             |
| LSP                   | `tower-lsp-server`, `lsp-types`        |
| Async runtime for LSP | `tokio`                                |
| Snapshot tests        | `insta`                                |
| CLI integration tests | `assert_cmd`, `predicates`, `tempfile` |
| Property tests        | `proptest`                             |
| Tracing/logging       | `tracing`, `tracing-subscriber`        |

Optional development/reference crates:

| Area                    | Crate                                         |
| ----------------------- | --------------------------------------------- |
| CLDR reference checks   | ICU4X crates, only in tests or optional tools |
| Archive download/unpack | `reqwest`, `zip`                              |
| Regex validation        | `regex`                                       |

Generated target code must not depend on Rust crates.

### 2.3 Repository state handoff

Keep a concise repository state handoff in `.codex`.

The handoff must be updated before ending or committing a completed implementation slice. It must let a future LLM continue without rediscovering basic state.

Include:

- current completed slice
- important implementation decisions
- tests last run
- known blockers or deferred work
- next recommended task

### 2.4 Rust workspace layout

```txt
crates/
  linguini-cli/
  linguini-config/
  linguini-core/
  linguini-syntax/
  linguini-schema/
  linguini-locale/
  linguini-analyzer/
  linguini-cldr/
  linguini-cldr-macros/
  linguini-ir/
  linguini-codegen-ts/
  linguini-format/
  linguini-lsp/
  linguini-package/
  linguini-test-support/
```

Responsibilities:

| Crate                   | Responsibility                                         |
| ----------------------- | ------------------------------------------------------ |
| `linguini-cli`          | command dispatch, user-facing CLI output               |
| `linguini-config`       | `linguini.toml` model and validation                   |
| `linguini-core`         | shared enums such as `TypeKind` and `FormatterKind`    |
| `linguini-syntax`       | lexer, parser, CST/AST, spans                          |
| `linguini-schema`       | schema AST model and schema symbol table               |
| `linguini-locale`       | locale AST model and path-based scope loading          |
| `linguini-analyzer`     | semantic analysis, type checking, diagnostics          |
| `linguini-cldr`         | CLDR ingestion, caching, plural parser, formatter data |
| `linguini-cldr-macros`  | procedural macros for compiled CLDR Rust table output  |
| `linguini-ir`           | target-independent localization IR                     |
| `linguini-codegen-ts`   | TypeScript output                                      |
| `linguini-format`       | `.lgs` and `.lgl` formatter                            |
| `linguini-lsp`          | language server                                        |
| `linguini-package`      | package import/export and registry support             |
| `linguini-test-support` | fixtures, golden tests, fake projects                  |

---

## 3. Project Layout

### 3.1 Root

```txt
linguini.toml
linguini/
  schema/
  locale/
```

### 3.2 Example

```txt
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

      forms/
        fruit/
          ru.lgl
          en.lgl

        size/
          ru.lgl
          en.lgl

      delivery/
        ru.lgl
        en.lgl
```

### 3.3 Namespace rules

Schema namespaces are derived from paths under `schema/`.

```txt
schema/shop/delivery.lgs -> shop.delivery
schema/shop/types.lgs    -> shop.types
```

Locale namespaces are derived from paths under `locale/`, excluding the final locale file.

```txt
locale/shop/delivery/ru.lgl -> locale ru, namespace shop.delivery
locale/shop/forms/fruit/ru.lgl -> locale ru, namespace shop.forms.fruit
```

Locale filenames must be BCP 47 locale tags with the `.lgl` extension.

Examples:

```txt
ru.lgl
en.lgl
en-US.lgl
pt-BR.lgl
zh-Hant.lgl
```

### 3.4 Top-down locale scope

Locale scope is inherited from parent directories.

For:

```txt
locale/shop/delivery/ru.lgl
```

the visible scope is loaded in this order:

```txt
locale/ru.lgl
locale/shop/ru.lgl
locale/shop/delivery/ru.lgl
```

For:

```txt
locale/shop/forms/size/ru.lgl
```

the visible scope is loaded in this order:

```txt
locale/ru.lgl
locale/shop/ru.lgl
locale/shop/forms/ru.lgl
locale/shop/forms/size/ru.lgl
```

A parent scope file may contain:

- local enums
- local functions
- local forms
- aliases
- imports
- formatter overrides
- package imports

---

## 4. Schema Types And Formatters

The canonical primitive schema types are represented by `TypeKind`:

| Type      | Meaning                         |
| --------- | ------------------------------- |
| `String`  | localized or user-provided text |
| `Number`  | numeric value                   |
| `Decimal` | decimal numeric value           |
| `Date`    | date/time-compatible value      |
| `Boolean` | boolean value                   |

`FormatterKind` is also canonical and shared across parser, IR, and codegen.
Known formatter kinds are `number`, `currency`, and `date`.

Schema type aliases may attach formatter annotations. This lets the schema own
the default formatting contract:

```lgs
type Money = Decimal @currency(code = "EUR")
type ShortDate = Date @date(style = "short")

checkout_total(amount: Money, created: ShortDate)
```

```lgl
checkout_total = Total {amount} on {created}
```

Generated code applies the alias formatter to `{amount}` and `{created}`. A
locale implementation can override formatting by writing an explicit formatter:

```lgl
checkout_total = Total {amount @number} on {created @date(style = "long")}
```

A child file may use declarations from parent scope files.

A child file may shadow a parent declaration only when explicitly marked with `override`.

---

## 4. Configuration

The project config file is `linguini.toml`.

```toml
[project]
name = "my-app"
default_locale = "ru"
locales = ["ru", "en"]

[paths]
schema = "linguini/schema"
locale = "linguini/locale"

# CLDR plural rules are compiled into the Linguini binary during cargo build.
# Project builds must not configure or populate a CLDR cache.

[targets.ts]
out = "src/generated/linguini"
module = "esm"
declaration = true

# JS and Rust targets are reserved for later backend implementation.

[format]
line_width = 100

[limits]
max_source_file_lines = 500
warn_source_file_lines = 400
```

---

## 5. Schema DSL `.lgs`

### 5.1 Schema file contents

A schema file may define:

- public enums
- public structs
- custom scalar aliases
- public message signatures
- grouped public messages
- schema doc comments

### 5.2 Doc comments

Schema declarations may contain doc comments.

```lgs
/// Displayed on the product delivery confirmation card.
/// The adjective must agree with the fruit name.
delivery(fruit: Fruit, size: Size, count: Number)

/// Shown near cart item count.
counted(count: Number, fruit: Fruit)
```

Doc comments must be available in:

- diagnostics
- `linguini fill`
- LSP hover
- LSP completion details
- generated TypeScript declarations

### 5.3 Enums

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
```

### 5.4 Custom scalar types

```lgs
type Money = Decimal @currency
type ShortDate = Date @date(style = "short")
type Percent = Number @percent
```

Schema-level formatter annotations define default formatting behavior.

### 5.5 Message signatures

```lgs
delivery(fruit: Fruit, size: Size)
counted(count: Number, fruit: Fruit)
price(amount: Money)
```

### 5.6 Grouped messages

```lgs
email_input {
  label
  placeholder
  aria
}
```

Parameterless grouped messages are written as bare identifiers. Parentheses are
used only when a message has parameters.

---

## 6. Locale DSL `.lgl`

### 6.1 Top-level declarations

A locale file may contain:

- local enums
- enum implementations
- typed forms
- local functions
- public message implementations
- grouped message implementations
- imports
- aliases

### 6.2 Local enums

```lgl
enum Gender { male, female, neuter, other }
```

Local enums are locale-specific. They are not public API.
Enum names are PascalCase. `Plural` is built in with `one`, `few`, `many`, and
`other` and must not be redeclared.

### 6.3 Enum implementations

`impl` defines localized data and forms for schema enum values.

```lgl
impl Fruit {
  apple {
    Gender = neuter

    form nom(Plural) {
      one => яблоко
      few => яблока
      _ => яблок
    }

    form gen(Plural) {
      one => яблока
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

    form gen(Plural) {
      one => груши
      _ => груш
    }
  }
}
```

An implementation variant may contain:

- typed enum attributes: `Gender = neuter`
- text attributes: `emoji = 🍎`
- local forms, such as `form nom(Plural) { ... }`
- nested attributes

### 6.4 Typed forms

Top-level `form` declarations model grammatical agreement over enum-like
parameters. Dispatch is nested in parameter order and `_` is the wildcard arm.
`Plural` parameters accept numeric call-site arguments and convert through CLDR
plural rules at runtime.

```lgl
form SizeAdj(Size, Plural, Gender) {
  small {
    one {
      male => маленький
      female => маленькая
      neuter => маленькое
      _ => маленький
    }
    _ => маленьких
  }

  big {
    one {
      male => большой
      female => большая
      neuter => большое
      _ => большой
    }
    _ => больших
  }
}
```

### 6.5 Local implementation forms

A local `form` inside an `impl` pattern-matches on the declared parameter type.

```lgl
form nom(Plural) {
  one => яблоко
  few => яблока
  _ => яблок
}
```

The analyzer validates plural categories against the current locale.

### 6.6 Local functions

Local functions are private locale helpers.

```lgl
fn DeliveryNote(item: String, Plural, Gender) {
  one {
    female => Доставлена {item}
    _ => Доставлен {item}
  }
  _ => Доставлены {item}
}
```

`fn` uses the same nested dispatch as `form`, but may also accept named `String`
parameters and interpolate them into branch text. Use `form` when the output
depends only on grammatical categories.

### 6.7 Messages

```lgl
delivery = {Delivered(count, fruit.Gender)} {SizeAdj(size, count, fruit.Gender)} {fruit.nom(count)}

counted = {count} {fruit.nom}

price = Цена: {amount}
```

Field access for typed enum attributes uses the type name, such as
`fruit.Gender`. Explicit `plural(count)` calls are not part of the locale DSL;
pass the numeric value directly to a `Plural` parameter.

Raw text after `=` and `=>` is trimmed at the outer edges when it is not quoted.

```lgl
big, neuter => большое
label = Email
```

These values are `большое` and `Email`, not values with leading spaces.

Quoted raw text preserves spaces inside the quotes.

```lgl
label = "  Email  "
```

This value is `  Email  `.

### 6.8 Placeholders

Supported placeholder expressions:

```lgl
{name}
{fruit.nom}
{fruit.nom(count)}
{fruit.gender}
{size(fruit.gender)}
{delivered(fruit.gender)}
{amount}
{amount @currency(code = "USD")}
{date @date(style = "long")}
```

Expression resolution is semantic. The parser only builds the AST.

### 6.9 Text rules

Raw text is allowed after `=` and `=>`.

```lgl
label = Email
male => Доставлен
```

Multiline text uses triple quotes.

```lgl
body = """
Здравствуйте, {name}

Ваш заказ доставлен.
"""
```

---

## 7. CLDR Support

### 7.1 CLDR input

The compiler must support CLDR JSON as the primary input format.

Required CLDR domains:

- plural rules
- number symbols
- decimal formatting
- percent formatting
- currency formatting
- date formatting
- time formatting
- date-time formatting
- unit formatting, later stage

Project-level CLDR caches may stay selective, but the compiler development build uses a pinned shallow checkout of the full `cldr-json` repository. The checkout is local-only, ignored by git, and verified against the pinned ref/commit metadata before build-time CLDR tables are generated.

Production binaries must contain the CLDR rules they need as compiled data, not as runtime JSON files and not as raw `include_str!` JSON blobs that are parsed at startup. The `linguini-cldr-macros` crate must provide a procedural macro/code-generation step, using `syn` and `quote`, that reads the required files from the `cldr-json` repository layout and emits Rust source for compact, typed CLDR rule/data tables. Runtime code must use those generated tables directly.

The upstream repository URL is:

```txt
https://github.com/unicode-org/cldr-json.git
```

The required upstream file paths currently used by Linguini are:

```txt
cldr-json/cldr-core/supplemental/plurals.json
cldr-json/cldr-numbers-full/main/{locale}/numbers.json
cldr-json/cldr-dates-full/main/{locale}/ca-gregorian.json
cldr-json/cldr-misc-full/main/{locale}/layout.json
```

### 7.2 CLDR source

The compiled CLDR tables are generated by the `linguini-cldr-macros` procedural macro crate.
`linguini-cldr` exposes the typed runtime API and does not own CLDR ingestion or code generation.

The macro reads CLDR data from `crates/linguini-cldr-macros/vendor/cldr-json`. If the pinned checkout is missing, the macro may fetch it with `git fetch --depth=1` using `crates/linguini-cldr-macros/cldr-json.toml`. Set `LINGUINI_CLDR_AUTO_FETCH=0` to force an explicit `./scripts/fetch-cldr-json.sh` step instead.

There is no public `linguini cldr` cache command surface.

### 7.3 CLDR plural expression parser

The `linguini-cldr` crate must parse CLDR plural rule expressions into an internal AST.

Required operands:

```txt
n, i, v, w, f, t, c, e
```

Required operators:

```txt
or
and
not
=
!=
%
in
within
..
,
```

Required output:

```rust
enum PluralRule {
    Or(Vec<PluralRule>),
    And(Vec<PluralRule>),
    Relation(Relation),
    True,
}
```

The compiler must lower plural rules into target-language code.

### 7.4 Formatting

Formatting is resolved at compile time into generated target-language formatting helpers.

Schema default:

```lgs
type Money = Decimal @currency
```

Locale override:

```lgl
price = Цена: {amount @currency(code = "USD")}
```

The generated code must include only formatters used by generated messages.

---

## 8. Lexer and Parser

### 8.1 Lexer

The lexer must produce tokens with source spans.

Token classes:

- identifiers
- locale tags
- string literals
- raw text segments
- braces
- parentheses
- commas
- colons
- equals
- arrows
- dots
- at-sign annotations
- newlines
- comments
- doc comments

### 8.2 Lexer modes

Use lexer modes:

| Mode           | Purpose                                        |
| -------------- | ---------------------------------------------- |
| code           | declarations, identifiers, punctuation         |
| raw text       | text after `=` or `=>` until newline/block end |
| multiline text | text inside triple quotes                      |
| placeholder    | expressions inside `{...}`                     |

### 8.3 Parser

Use `chumsky` for syntax parsing.

The parser must produce:

- a syntax tree with spans
- recoverable parse errors
- separate ASTs for `.lgs` and `.lgl`

The parser must not perform type checking.

### 8.4 Diagnostics

Use `ariadne` for CLI diagnostics.

Diagnostics must include:

- file path
- line and column
- highlighted span
- message
- optional note
- optional quick-fix hint
- related spans where relevant

---

## 9. Semantic Analysis

### 9.1 Analyzer inputs

The analyzer receives:

- project config
- schema ASTs
- locale ASTs
- path-derived namespaces
- CLDR data
- package imports
- target configuration

### 9.2 Checks

Required checks:

- unknown schema type
- unknown enum variant
- missing public message implementation
- unknown message implementation
- duplicate declaration
- invalid local enum selector
- missing selector branch
- missing `other` branch where required
- invalid plural category for locale
- missing plural fallback
- unknown variable
- unknown form property
- wrong function arity
- ambiguous implicit plural number
- unresolved import
- cyclic function/form/message references
- formatter mismatch
- package version conflict

### 9.3 Implicit plural argument

When a plural map is accessed without an explicit numeric argument, the analyzer selects the numeric argument only if exactly one numeric message parameter is in scope.

Example:

```lgs
counted(count: Number, fruit: Fruit)
```

```lgl
counted = {count} {fruit.nom}
```

If more than one numeric parameter exists, the analyzer must require an explicit argument:

```lgl
summary = {apples} {fruit.nom(apples)}
```

---

## 10. IR

The IR must be target-independent.

Required nodes:

```txt
Text
Concat
Argument
FormAccess
FunctionCall
Selector
PluralSelect
Format
Message
Group
```

IR must be fully resolved:

- no unknown references
- no unresolved selector names
- no implicit plural arguments
- no source-only syntax

---

## 11. Code Generation

### 11.1 Targets

Initial target:

- TypeScript

JavaScript and Rust code generation are deferred and are not part of the current implementation requirements.

### 11.2 TypeScript output

Generated output:

- ESM by default
- `.d.ts` declarations
- typed function arguments
- tree-shakable message functions
- locale-specific modules
- plural and formatter helpers imported explicitly by locale modules
- grouped messages emitted as nested objects, not flattened function names
- static messages emitted as constants or object properties when no interpolation or formatting is needed
- a generated locale map/facade for selecting a locale module

Example:

```ts
// shared.ts
export function selectBranch(
  key: string,
  branches: Record<string, string>,
): string;

// locales/ru.ts
import { selectBranch } from "../shared";
import { pluralRu } from "../plurals";

export function delivery(fruit: Fruit, size: Size, count: number): string;
export function counted(count: number, fruit: Fruit): string;

export const email_input = {
  label: "Email",
  placeholder: "name@example.com",
  aria: "Адрес электронной почты",
} as const;

const lgl = {
  delivery,
  counted,
  email_input,
} as const;

export default lgl;

// index.ts
import ru from "./locales/ru";

export type Linguini = typeof ru;
export const locales: readonly ["ru"];
export function createLinguini(language: "ru"): Linguini;
export function createLinguiniProvider(options: {
  resolveLanguage: () => "ru";
}): Linguini;
export const lgl: Linguini;
```

The public generated API must be structured so application code can switch the active output language by changing one locale source variable or provider, without changing every message call site. For example, a SvelteKit application must be able to connect that source variable to cookies, route data, or the UI language, while user code continues to call `lgl.delivered(...)` or another generated facade method. Locale selection belongs at the generated facade/provider boundary, not in every message call.

Generated TypeScript must also be shaped so a separate Linguini TypeScript runtime library can wrap it without rewriting generated message modules. The generated package must export enough static metadata and stable hooks for that library:

- `locales`: a readonly array of all available locale tags, in deterministic order;
- `baseLocale`: the configured default locale;
- locale module imports or loaders that the runtime can resolve by locale;
- a stable locale provider contract used by `getLocale()` and generated message facades;
- message functions that can be called through the current locale context without adding locale arguments at every call site.

### 11.3 TypeScript runtime integration library

The TypeScript generator must be compatible with a separate framework-aware runtime library that provides Paraglide-style application integration.

The runtime library must be able to use generated metadata and message modules to provide:

- `locales`: all available locales exported from generated output;
- `baseLocale`: the configured fallback locale;
- `getLocale()` and `setLocale()` or an equivalent locale-source API;
- `localizeHref(href, options?)`, converting paths between locales, for example `/en/about` to `/ru/about`;
- `shouldRedirect(requestOrLocation, options?)`, reporting when navigation should be resynchronized to the active locale URL;
- `getTextDirection(locale?)`, returning `ltr` or `rtl` for the current or supplied locale;
- a rich-text `<Trans>` component API for component interpolation, such as links or buttons inside translated text;
- a Vite plugin that watches Linguini source files and regenerates output when translations change;
- framework adapters, including Svelte/SvelteKit bindings, without changing generated locale module internals.

Locale detection must be configurable as an ordered strategy chain. The default chain is:

```ts
["url", "cookie", "preferredLanguage", "localStorage", "baseLocale"];
```

The order determines precedence. Each detector must be optional, environment-aware, and skipped when unavailable. The runtime must support at least:

- `url`: locale prefix or route parameter;
- `cookie`: persisted locale cookie;
- `preferredLanguage`: `Accept-Language` on the server or `navigator.languages` in the browser;
- `localStorage`: persisted browser locale;
- `baseLocale`: final fallback.

Localized URL support must be driven by generated locale metadata and must not require user-authored route tables for basic locale-prefix routing. The runtime must expose enough configuration to support non-prefixed base locales and custom URL patterns later.

The Vite plugin must be framework-agnostic. During development it must watch `.lgs`, `.lgl`, and relevant `linguini.toml` files, regenerate TypeScript output after changes, and trigger the Vite module graph updates needed for locale HMR.

Server middleware must detect the locale from each request and provide a per-request locale context so `getLocale()` and generated message facade calls resolve the correct locale during concurrent requests. The default Node/server implementation should use `AsyncLocalStorage`. The runtime must also provide a `disableAsyncLocalStorage` option for edge and serverless environments where that API is unavailable or unnecessary.

For SvelteKit and static-site generation, the runtime adapter must:

- integrate with server middleware/hooks;
- replace `%lang%` and `%dir%` placeholders in `app.html` with the current language and text direction;
- generate locale switcher links that allow static builds to discover all localized pages;
- keep generated message calls usable in components without passing the locale into every call.

The `<Trans>` component must support rich text and component interpolation while preserving type safety for message parameters. The message source format may continue to produce plain strings for normal calls, but generated metadata or helper output must be sufficient for the runtime to map named rich-text placeholders to framework components.

### 11.4 Optimization requirements

Generated code must:

- emit only used locale modules
- emit only used messages when TypeScript tree-shaking mode is enabled with an explicit message list
- emit only used forms
- emit only used plural functions
- emit only used formatting helpers
- deduplicate identical static strings
- avoid runtime parsing
- avoid runtime CLDR lookup
- avoid loading all locales when only one is imported
- inline small selector maps where beneficial
- use static lookup tables for large enum selectors
- avoid string allocation for static messages
- use target-native formatting when configured
- support deterministic output for snapshot testing

---

## 12. CLI

Required commands:

```bash
linguini init
linguini check
linguini build
linguini generate
linguini format
linguini fill
linguini status
linguini cldr fetch
linguini cldr status
linguini lsp
```

Later-stage commands:

```bash
linguini package init
linguini package publish
linguini package add
linguini package update
linguini package audit
```

### 12.1 `init`

Creates project structure.

### 12.2 `check`

Runs full analysis without code generation.

### 12.3 `build`

Runs analysis and code generation.

### 12.4 `generate`

Generates rendered, colorized sample output for each configured locale.

It must cover:

- every configured locale
- every schema enum variant used by message parameters
- representative numeric values for plural branches
- grouped messages and standalone messages
- default-locale fallback output for incomplete secondary locales

The command is for inspecting how generated localization contracts behave with realistic argument matrices.

### 12.5 `fill`

Creates missing locale files and missing message stubs.

It must use schema doc comments as context.

### 12.6 `status`

Shows locale completion.

---

## 13. Formatter

The formatter must support `.lgs` and `.lgl`.

Required:

- stable output
- idempotent formatting
- max line width from config
- preserve doc comments
- preserve ordinary comments where possible
- sort enum variants only when explicitly configured
- no semantic changes

---

## 14. LSP

Use `tower-lsp-server`.

Required features:

- diagnostics
- hover
- completion
- go to definition
- find references
- semantic tokens
- document symbols
- workspace symbols
- code actions
- quick fixes
- formatting
- rename for schema symbols
- missing branch generation
- missing message generation
- missing form variant generation
- placeholder completion from schema args
- form property completion
- local enum branch completion

Quick-fix examples:

- add missing enum branches
- add missing `other` branch
- add missing locale message
- add missing form variant
- add explicit plural argument
- create missing locale file

---

## 15. Syntax Highlighting

Deliverables:

- TextMate grammar for `.lgs`
- TextMate grammar for `.lgl`
- VS Code extension
- optional tree-sitter grammar
- semantic token mapping from LSP

---

## 16. Packages

Package support is a later-stage feature.

Packages may provide:

- schema types
- schema enums
- locale forms
- local functions
- formatter presets
- CLDR extensions
- reusable dictionaries

Example use cases:

- common product nouns for multiple locales
- grammatical adjective sets
- country/currency display names
- domain-specific UI terminology

Package declarations must support semantic versioning and lockfiles.

```txt
linguini.lock
```

Imports must be explicit.

```lgl
import @linguini/ru-food/FruitForms
```

Package content must be analyzable before code generation.

---

## 17. Engineering Constraints

### 17.1 File size

Non-generated source files must stay below 500 lines.

Rules:

- warn at 400 lines
- fail CI at 500 lines
- generated files are exempt
- vendored files are exempt
- tests should prefer small focused files

### 17.2 Modularity

Each module must have one responsibility.

Required module boundaries:

- lexing
- parsing
- AST
- path discovery
- schema symbol table
- locale scope table
- CLDR loading
- CLDR plural parsing
- semantic diagnostics
- IR lowering
- TypeScript backend
- LSP handlers

### 17.3 Code quality

Required:

- implementation must follow the technology stack in this specification unless the specification is explicitly updated first
- no large catch-all modules
- no business logic in `main.rs`
- no target-specific logic in analyzer
- no parser logic in analyzer
- no CLDR download during normal build
- deterministic output
- no feature is complete until its automated tests are committed
- no bug fix is accepted without a regression test
- no implementation slice may simplify, skip, or omit specified behavior in order to mark a stage complete

### 17.4 Testing policy

All production code must be covered by automated tests.

Required test layers:

| Layer                 | Required coverage                                                                                   |
| --------------------- | --------------------------------------------------------------------------------------------------- |
| Unit tests            | lexer, parser, analyzer, CLDR plural parser, formatter, IR lowering, codegen helpers                |
| Golden fixture tests  | `.lgs` and `.lgl` source files, expected AST summaries, expected diagnostics, expected IR summaries |
| Snapshot tests        | diagnostics, formatted output, generated TypeScript                                                 |
| CLI integration tests | `init`, `check`, `build`, `fmt`, `fill`, `status`, `map update`, package commands                   |
| LSP tests             | hover, completion, diagnostics, semantic tokens, formatting, code actions, quick fixes              |
| Generated-code tests  | compile or run generated output for TypeScript                                                      |
| Regression tests      | one focused test for every fixed bug                                                                |

Coverage requirements:

- core crates must target at least 80% line coverage;
- lexer, parser, analyzer, CLDR plural parser, formatter, and codegen must have behavior-complete rule coverage;
- coverage percentage is a guardrail, not a replacement for semantic tests;
- generated files and vendored files are excluded from coverage metrics;
- every semantic rule in this specification must have at least one test fixture.

Syntax fixtures must be behavior-complete, valid Linguini programs or intentionally invalid programs with a precise diagnostic purpose. Golden `.lgs` and `.lgl` fixtures must exercise complete declarations and realistic message bodies, not abbreviated fragments that only satisfy the current parser shape.

Generated output validation:

| Target     | Required validation                                           |
| ---------- | ------------------------------------------------------------- |
| TypeScript | generated fixtures must pass `tsc --noEmit` and runtime tests |

Recommended Rust test tools:

- `cargo test` for unit and integration tests;
- `insta` for snapshots;
- `assert_cmd`, `predicates`, and `tempfile` for CLI tests;
- `proptest` for lexer/parser/analyzer invariants;
- Node.js test fixtures for generated TypeScript output.

Completion rule:

A checklist item may be marked complete only when:

1. the implementation exists;
2. tests for the implementation exist;
3. the relevant test command is recorded as evidence;
4. diagnostics or generated outputs have snapshots when applicable.

Sequential delivery rule:

Work may move from one stage or checklist part to the next only after the previous part is fully complete, including tests, snapshots, generated-output validation, and recorded evidence. If a later part is worked on early for dependency discovery, it must not be marked complete and must not be used to bypass unfinished acceptance criteria in the earlier part.

### 17.5 Documentation

Each public crate must provide:

- crate-level docs;
- module-level docs;
- examples for public APIs;
- fixture-based tests.

---

## 18. Delivery Plan

The delivery plan is sequential. A stage is complete only when all of its checkpoint results and checklist acceptance items are complete with evidence. Moving to the next stage before that point is not allowed except for short exploratory spikes that are explicitly left incomplete.

### Stage 1: Project model and file discovery

Checkpoint result:

- `linguini init` creates the structure
- `linguini check` discovers schema and locale files
- namespace and locale are derived from paths

### Stage 2: Syntax parser

Checkpoint result:

- `.lgs` and `.lgl` parse into ASTs
- diagnostics include spans
- invalid syntax recovers where possible

### Stage 3: Schema and locale analyzer

Checkpoint result:

- public messages are matched to implementations
- forms and local functions resolve
- path-based top-down scope works

### Stage 4: CLDR plural support

Checkpoint result:

- CLDR JSON is cached
- plural rules parse into IR
- plural categories validate per locale

### Stage 5: TypeScript codegen and runtime integration

Checkpoint result:

- generated TS functions run in a sample app
- generated output exposes locale metadata required by the runtime integration library
- static messages allocate no dynamic structures
- only used locale code is emitted

### Stage 6: TypeScript application integration

Checkpoint result:

- detection strategy resolves locales in browser and server contexts
- middleware provides concurrent request-safe locale context
- `localizeHref`, `shouldRedirect`, and `<Trans>` work in a SvelteKit sample

### Stage 7: Formatter

Checkpoint result:

- `.lgs` and `.lgl` formatting is idempotent
- comments are preserved

### Stage 8: LSP and editor support

Checkpoint result:

- diagnostics, completion, hover, and quick fixes work in VS Code

### Stage 9: Locale management

Checkpoint result:

- `fill` and `status` work on real projects
- missing messages can be generated from schema docs

### Stage 10: Formatting data

Checkpoint result:

- number/date/currency formatting works without runtime CLDR lookup

### Stage 11: Packages

Checkpoint result:

- packages can be imported, locked, analyzed, and used in codegen
