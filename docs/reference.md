# Language Reference

Linguini has two file types:

- **`.lgs`** — schema. Defines message signatures, enums, and type aliases.
- **`.lgl`** — locale. Implements messages for one locale.

---

## Schema (`.lgs`)

### `enum`

Declares a set of named variants. Names are PascalCase. Variants are lowercase.

```lgs
enum Fruit  { apple, pear, orange }
enum Size   { small, big }
```

### Type aliases

Built-in primitive types are `String`, `Number`, `Decimal`, `Date`, and
`Boolean`. They are represented internally by `TypeKind`, so parser, analyzer,
IR, and codegen share one canonical type list.

Attach a formatter to a primitive type to make formatting part of the schema
contract. Locale files can then interpolate the value directly; generated code
applies the schema formatter automatically.

```lgs
type Money     = Decimal @currency(code = "EUR")
type ShortDate = Date    @date(style = "short")

checkout_total(amount: Money, created: ShortDate)
```

```lgl
checkout_total = Total {amount} on {created}
```

Locale authors can still override formatting at the interpolation site:

```lgl
checkout_total = Total {amount @number} on {created @date(style = "long")}
```

Primitive `Number`, `Decimal`, and `Date` parameters also get schema-owned
defaults even without aliases:

```lgs
summary(count: Number, total: Decimal, created: Date)
```

```lgl
summary = {count} items, {total}, {created}
```

Generated TypeScript emits the locale CLDR data once per locale module and
passes a shared `FORMATTER_DATA` constant into each formatter call, so repeated
interpolations do not inline the full number/date/currency table.

The formatter list is canonical:

| Formatter   | Applies to                |
| ----------- | ------------------------- |
| `@number`   | `Number`, `Decimal`       |
| `@currency` | `Number`, `Decimal` alias |
| `@date`     | `Date`                    |

### Messages

A message is a named entry the app can call. Parameters are typed.

```lgs
delivery(fruit: Fruit, size: Size, count: Number)
greeting(name: String)
```

Doc comments attach to the next declaration and appear in codegen output and LSP hover.

```lgs
/// Shown on the delivery confirmation card.
delivery(fruit: Fruit, size: Size, count: Number)
```

### Parameterless messages

A bare identifier inside a namespace block is a message with no parameters.

```lgs
email_input {
  label
  placeholder
  aria
}
```

### Namespaces and qualified paths

Linguini has two kinds of namespace segment:

1. A **filesystem namespace** comes from a schema path relative to
   `paths.schema`. It includes the schema file stem. The matching locale
   namespace comes from the locale file's parent path relative to
   `paths.locale`; the locale file stem is a locale tag, not a namespace
   segment.
2. A **source group** is a recursive `{ ... }` block. It extends message paths
   inside one schema or locale file.

`project.name` is not a namespace segment. It never prefixes schema identities
or the generated `l` object.

Given these files:

```text
linguini/schema/shop/checkout.lgs
linguini/locale/shop/checkout/en.lgl
```

both files have filesystem namespace `shop.checkout`. The schema and locale can
then declare the same recursive group tree:

```lgs
receipt {
  header {
    title
    order_number(value: String)
  }
}
```

```lgl
receipt {
  header {
    title = Receipt
    order_number = Order {value}
  }
}
```

The canonical message identities are
`shop.checkout.receipt.header.title` and
`shop.checkout.receipt.header.order_number`. Generated application access uses
the same segments:

```ts
l.shop.checkout.receipt.header.title;
l.shop.checkout.receipt.header.order_number("A-104");
```

Canonical paths are used by diagnostics, generated APIs, and fully qualified
configuration entries such as `targets.ts.messages`. They are not source-level
imports. Linguini source currently has no `import` or `export` syntax; symbols
declared in a file are resolved in that file's filesystem namespace.

The filesystem namespace scopes every declaration lowered from a file:
schema messages, enums, and type aliases, plus locale messages, enums,
variables, forms, and functions. Source groups contain only messages and child
groups, so group segments extend message identities only.

#### Path derivation

| Path below its configured root | Derived value |
| --- | --- |
| schema `main.lgs` | namespace `main` |
| schema `shop/cart.lgs` | namespace `shop.cart` |
| locale `main/en.lgl` | namespace `main`, locale `en` |
| locale `shop/cart/pt-BR.lgl` | namespace `shop.cart`, locale `pt-BR` |

Configured roots are project-relative and use `/`. Empty and `.` path
components are normalized away; absolute paths, backslashes, `..`, overlapping
source/output roots, source-tree symlinks, and non-UTF-8 namespace components
are rejected.

Filesystem components are joined with `.`. A literal dot therefore creates the
same canonical namespace as a directory boundary:
`shop.checkout.lgs` and `shop/checkout.lgs` both derive
`shop.checkout` and cannot coexist. Namespace collisions are checked
case-insensitively for portable output, while a schema and its locale directory
must use the same exact spelling. Prefer `lowercase_snake_case` filesystem
components for readable generated properties. Locale file stems must be valid
BCP 47 tags and use the canonical spelling configured in `project.locales`.

#### Group and declaration collisions

Groups are recursive. Schema groups may be empty to reserve a namespace; locale
groups must contain a message or child group. The language-wide brace nesting
limit is 64.

Sibling messages and child groups share one member namespace. Reusing a member
name for two messages, two groups, or one message and one group is invalid:

```lgs
account {
  profile
  profile { title }
}
```

Canonical identities must also remain unique after filesystem and group
segments are combined. For example, these two schema declarations both produce
`shop.checkout.title` and therefore collide:

```text
schema/shop.lgs              contains: checkout { title }
schema/shop/checkout.lgs     contains: title
```

The same source-local declaration name may be reused in distinct namespaces
when the final canonical paths differ, such as `shop.title` and `admin.title`.
Renaming or moving a schema file changes every canonical identity owned by that
file; move its locale directory and update generated API/configuration paths in
the same migration.

---

## Locale (`.lgl`)

A locale file implements all messages declared in the schema with the same
filesystem namespace. Its nested group tree reproduces the schema's canonical
message paths.

### Simple messages

```lgl
greeting = Hello, {name}!
sign_out  = Sign out
```

### Interpolation

Reference a parameter by name inside `{ }`:

```lgl
greeting = Hello, {name}!
price    = Total: {amount} {currency}
```

### Local variables

Use `let` in locale files for reusable local text fragments. Local variables are
not part of the schema API and can be referenced from messages, forms, and
functions in the same locale module.

```lgl
let cart_label = Cart

cart_summary = {cart_label}: {count} {item.nom(count)}
```

### `Plural` — built-in

`Plural` is always available with variants `one | few | many | other`.
Any `Number` passed where `Plural` is expected converts automatically
using CLDR plural rules for the active locale. No wrapper function needed.

---

### `impl` — words with grammar

`impl` declares the linguistic implementation of an enum. Each block corresponds
to one variant and may contain typed fields, `form` declarations, and static
string blocks.

```lgl
enum Gender { masculine, feminine, neuter, other }

impl Fruit {
  apple {
    Gender = neuter
    emoji  = 🍎

    form nom(Plural) {
      one => яблоко
      few => яблока
      _   => яблок
    }

    form gen(Plural) {
      one => яблока
      _   => яблок
    }

    display {
      short = ябл.
      long  = спелое яблоко
    }
  }

  pear {
    Gender = feminine

    form nom(Plural) {
      one => груша
      few => груши
      _   => груш
    }
  }
}
```

**Enum-valued properties** use `TypeName = value`. When the PascalCase key
resolves to an enum, locale-expression analysis treats property access as that
enum type. The current analyzer does not yet validate the property's text value
against the enum variants.

```lgl
Gender = neuter   // property access is inferred as Gender
emoji  = 🍎       // plain string property
```

**Accessing fields and forms in templates:**

```lgl
fruit.Gender      // inferred enum-valued property access
fruit.nom(count)  // form call — count auto-converts to Plural
```

---

### `form` — grammatical agreement

`form` matches on enum variants and returns a string. It cannot take `String`
parameters — use `fn` for that.

Parameters are types only, no names. Each nesting level dispatches on one
parameter top-down. `_` matches all remaining variants at the current level.

```lgl
form Delivered(Plural, Gender) {
  one {
    masculine => Доставлен
    feminine  => Доставлена
    neuter    => Доставлено
    _         => Доставлено
  }
  _ => Доставлены
}
```

Multiple parameters, deeper nesting:

```lgl
form SizeAdj(Size, Plural, Gender) {
  small {
    one {
      masculine => маленький
      feminine  => маленькая
      neuter    => маленькое
      _         => маленький
    }
    _ => маленьких
  }
  big {
    one {
      masculine => большой
      feminine  => большая
      neuter    => большое
      _         => большой
    }
    _ => больших
  }
}
```

**Ordering:** parameters should go from the enum with fewest variants to the most.
This keeps the top levels of the dispatch tree narrow (`param_order` lint).

**Branch coverage:** `linguini check` and `linguini build` require every dispatch
level they can resolve against a schema/locale enum or `Plural` to cover each
variant explicitly or include `_`. A missing branch is a blocking diagnostic.
This is scoped analyzer coverage, not a whole-program exhaustiveness proof.

---

### `fn` — forms with string interpolation

`fn` works like `form` but can accept named `String` parameters and interpolate
them into output values.

```lgl
fn delivery_note(item: String, Plural, Gender) {
  one {
    feminine  => Доставлена {item}
    _         => Доставлен {item}
  }
  _ => Доставлены {item}
}
```

Use `form` when output depends only on grammatical categories.
Use `fn` when output embeds a dynamic string value.

Functions are module declarations. Inline `fn` expressions are not part of the
language grammar; declare the function once and call it from message
interpolation.

---

### Wildcard `_`

`_` matches all remaining variants of the current parameter. Must be last.

```lgl
form F(Gender) {
  feminine => она
  _        => он
}
```

---

## Errors and warnings

### Errors (block codegen)

- Missing branch in a resolved enum or `Plural` dispatch
- Type mismatch in a locale function/form call when both types resolve
- Unresolved reference in interpolation
- Missing message implementation in locale
- `impl` variant not declared in its enum
- Unknown formatter annotation

### Warnings

- Style and reachability lints listed below

Linguini does not scan application source for message usage, so it does not
currently report schema messages that are unused by the application.

### Machine-readable diagnostics

`check` and `build` keep terminal-oriented output by default. CI and editor integrations can
select JSON or SARIF 2.1.0 explicitly:

```bash
linguini check --format json
linguini check --format sarif > linguini.sarif
linguini build --format json
```

JSON diagnostics include stable codes, categories, severities, project-relative paths, byte and
line/column ranges, related locations, notes, and quick fixes. SARIF replacement fixes use standard
`artifactChanges` and `replacements`; command-style Linguini fixes are also retained in result
properties. Machine-readable diagnostics are written to stdout even when diagnostics make the
command exit unsuccessfully, so redirected output remains a complete JSON document.

---

## Lints (`linguini check`)

Lints are named diagnostics reported by `linguini check`. Five are warnings;
`incomplete_impl` is a blocking error. Inline lint suppression syntax is not
part of the current language.

| Lint                 | Severity | Description                                      |
| -------------------- | -------- | ------------------------------------------------ |
| `param_order`        | warning  | Order `form` params from fewest variants to most |
| `unreachable_arm`    | warning  | Arm after `_` at the same level                  |
| `collapsible_arms`   | warning  | Multiple arms with identical output              |
| `fn_without_strings` | warning  | `fn` with no `String` params — use `form`        |
| `incomplete_impl`    | error    | `impl` missing variants declared in enum         |
| `redundant_wildcard` | warning  | All variants covered, `_` is unreachable         |

---

## Config (`linguini.toml`)

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
module      = "esm"                     # esm | cjs
declaration = true
gitignore   = true                      # emit generated .gitignore
```
