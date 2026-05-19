# Getting Started

> **Preview status.** Linguini is functional end-to-end — the language, analyzer,
> LSP, and TypeScript codegen all work. The codebase is being actively cleaned up
> and the syntax is stabilizing. Expect rough edges, and feel free to open issues.

---

## Install the CLI

```bash
cargo install linguini-cli --version 0.1.0-alpha.3
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
module      = "esm"
declaration = true
```

## Write a schema

Define your messages and the types they work with:

```lgs
// linguini/schema/main.lgs

/// Greeting shown on the home page.
hello(name: String)

/// Error shown when a field is empty.
field_required(field: String)
```

## Write a locale

Implement the schema for each locale:

```lgl
// linguini/locale/main/en.lgl

hello = Hello, {name}!
field_required = {field} is required.
```

```lgl
// linguini/locale/main/ru.lgl

hello = Привет, {name}!
field_required = Поле «{field}» обязательно для заполнения.
```

## Check for errors

```bash
linguini check
```

The analyzer reports missing implementations, unresolved references, incomplete
match branches, and type errors. Run this in CI.

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
l.main.field_required("Email"); // → "Email is required."
```

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
