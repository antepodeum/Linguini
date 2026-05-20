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

| Formatter  | Applies to                 |
| ---------- | -------------------------- |
| `@number`  | `Number`, `Decimal`        |
| `@currency`| `Number`, `Decimal` alias  |
| `@date`    | `Date`                     |

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

### Namespaces

A block groups related messages under a shared prefix.

```lgs
auth {
  sign_in(email: String)
  sign_out
  error_invalid_credentials
}
```

### Imports and exports

Enums and types can be shared across schema files.

```lgs
// shared.lgs
export enum Status { active, inactive, pending }

// orders.lgs
import { Status } from "./shared.lgs"
```

---

## Locale (`.lgl`)

A locale file implements all messages declared in the corresponding schema.

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

**Typed fields** use `TypeName = value`. The type is inferred from the
PascalCase key — no ambiguity with lowercase string fields.

```lgl
Gender = neuter   // typed: Gender enum
emoji  = 🍎       // plain string field
```

**Accessing fields and forms in templates:**

```lgl
fruit.Gender      // typed field access
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

**Exhaustiveness:** all match expressions must be exhaustive. Cover every variant
explicitly or include `_`. A non-exhaustive form is a compile-time error.

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

### Inline forms

A `fn` can be written inline as an interpolation value:

```lgl
greeting = Привет, {fn(Gender) { masculine => дорогой, feminine => дорогая, _ => дорогой }} {name}!
```

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

- Non-exhaustive match without `_`
- Type mismatch at call site
- Unresolved reference in interpolation
- Missing message implementation in locale
- `impl` variant not declared in its enum
- Unknown formatter annotation

### Warnings

- Unused message — in schema, never referenced in source
- `fn` with no `String` parameters — suggest `form`

---

## Lints (`linguini check`)

All lints are `warn` by default. Suppress with `#[allow(lint_name)]`.

| Lint                 | Description                                      |
| -------------------- | ------------------------------------------------ |
| `param_order`        | Order `form` params from fewest variants to most |
| `unreachable_arm`    | Arm after `_` at the same level                  |
| `collapsible_arms`   | Multiple arms with identical output              |
| `fn_without_strings` | `fn` with no `String` params — use `form`        |
| `incomplete_impl`    | `impl` missing variants declared in enum         |
| `redundant_wildcard` | All variants covered, `_` is unreachable         |

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
```
