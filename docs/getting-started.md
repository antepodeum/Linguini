# Getting Started

> **Preview status.** Linguini is functional end-to-end — the language, analyzer,
> LSP, and TypeScript codegen all work. The codebase is being actively cleaned up
> and the syntax is stabilizing. Expect rough edges, and feel free to open issues.

---

## Install the CLI

```bash
cargo install linguini-cli --version 0.1.0-alpha.4
```

## Scaffold a project

```bash
linguini init
```

This creates the default project structure:

```
linguini.toml
linguini/
  schema/
    main.lgs
  locale/
    main/
      en.lgl
```

## Configure

Edit `linguini.toml` to match your project:

```toml
[project]
name           = "my-app"
default_locale = "en"
locales        = ["en", "ru"]

[paths]
schema = "linguini/schema"
locale = "linguini/locale"

[targets.ts]
out         = "src/generated/linguini"
declaration = true
gitignore   = true
```

### Paths and namespaces

`paths.schema`, `paths.locale`, and `targets.ts.out` are relative to the project
directory. Use `/` as the separator. Linguini trims surrounding whitespace and
normalizes repeated `/`, leading `./`, and trailing `/`. Absolute paths, `\`,
parent traversal (`..`), and overlapping source/output roots are rejected.

`project.name` is project metadata and supplies default cookie/local-storage
names when those web features are enabled. It is not a source namespace and
does not appear in generated message paths.

Each schema file owns the filesystem namespace formed from its path below
`paths.schema`, including its file stem. A locale implementation mirrors that
path below `paths.locale`; its file stem is the locale tag:

| Schema | English locale | Namespace | Generated prefix |
| --- | --- | --- | --- |
| `linguini/schema/main.lgs` | `linguini/locale/main/en.lgl` | `main` | `l.main` |
| `linguini/schema/shop/cart.lgs` | `linguini/locale/shop/cart/en.lgl` | `shop.cart` | `l.shop.cart` |

Case must match exactly between the schema path and locale directory. A locale
file stem must use a valid BCP 47 tag and the canonical spelling from
`project.locales` (`pt-BR`, not `pt-br`). Source groups add more segments after
the filesystem namespace.

When migrating a layout that treated all schema files as one shared namespace,
move each locale file beneath the path of its schema. For example,
`schema/shop/cart.lgs` pairs with `locale/shop/cart/en.lgl`, not
`locale/en.lgl` or `locale/shop/en.lgl`. Remove a redundant outer source group
if it would repeat the file namespace (`shop.lgs` plus `shop { ... }` produces
`l.shop.shop...`). Moving a schema file is a generated-API rename, so update
application access paths and move every matching locale directory together.

## Write a schema

Define your messages and the types they work with:

```lgs
// linguini/schema/main.lgs

type Money = Decimal @currency(code = "USD")
type ShortDate = Date @date(style = "short")

/// Greeting shown on the home page.
hello(name: String)

/// Checkout total with schema-owned formatting.
checkout_total(amount: Money, created: ShortDate)

/// Error shown when a field is empty.
field_required(field: String)
```

## Write a locale

Implement the schema for each locale:

```lgl
// linguini/locale/main/en.lgl

hello = Hello, {name}!
checkout_total = Total {amount} on {created}
field_required = {field} is required.
```

You only need aliases when the schema should carry specific options like
currency code or date width. Plain `Number`, `Decimal`, and `Date` parameters
are formatted automatically from the active locale, and a locale can override a
single interpolation when needed:

```lgl
checkout_total = Total {amount @number} on {created @date(style = "long")}
```

```lgl
// linguini/locale/main/ru.lgl

hello = Привет, {name}!
checkout_total = Итого {amount} на {created}
field_required = Поле «{field}» обязательно для заполнения.
```

## Check for errors

```bash
linguini check
```

The analyzer reports missing implementations and unresolved references. It also
checks branch coverage for enum/`Plural` dispatches and locale-call argument
types when it can resolve the participating declarations. Run this in CI.

## Apply quick fixes

```bash
linguini fix
```

Generates stubs for any messages that exist in the schema but are missing from
a locale file. Useful when adding a new message — run `fix` and fill in the strings.

## Build

```bash
linguini build
```

Writes generated code to the paths configured in `linguini.toml`.

## Use in your app

```ts
import { configureLinguini } from "./generated/linguini";

const l = configureLinguini({ language: () => getRequestLocale() });

l.main.hello("Artemy"); // → "Hello, Artemy!"
l.main.field_required({ field: "Email" }); // → "Email is required."
```

The generated object follows the schema's nested paths. A parameterless message
is a value (`l.main.title`), not a zero-argument function. A parameterized message
accepts either its original positional arguments or one named object. Named
properties use schema parameter names, may be reordered, and are all required;
generated types reject missing and unknown properties.

Both call forms use the same generated ESM function and message body. Schema
`///` documentation is emitted on both overloads, and `declaration = true`
generates matching `.d.ts` overloads from the same signature model. In
Svelte/Vite projects, static message paths can be transformed to the exact
generated module for each message.

Bundler mode is configured under `[targets.ts.bundler]` (and requires a Svelte
or SvelteKit framework target). The default dynamic-access policy is strict:
computed message paths fail the build. If a finite dynamic escape is required,
list canonical message paths explicitly:

```toml
[targets.ts]
framework = "sveltekit"

[targets.ts.bundler]
sources = ["src"]

[targets.ts.bundler.dynamic]
mode = "bundle"
allow = ["main.title", "main.hello"]
```

`locale_loading = "dynamic"` is optional under `[targets.ts.bundler]`; the
default is eager locale loading. With dynamic loading, client locale chunks are
loaded before a locale switch while SSR remains synchronous and static.

## VS Code extension

Install the extension from the marketplace for inline diagnostics, completions,
hover, go-to-definition, and code actions — all backed by the same analyzer
the CLI uses. Install Linguini CLI separately; the extension runs `linguini lsp`
from PATH by default.

Or run it locally from source:

```bash
cd editors/vscode
npm install
npm run compile
npm run open:dev
```

## Next steps

- [Language Reference](./reference.md) — full syntax for schemas, locales, forms, and functions
- [Why Linguini](./why.md) — how Linguini compares to ICU, Fluent, and Paraglide
