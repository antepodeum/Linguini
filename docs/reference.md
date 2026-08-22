# Language Reference

The committed JSON Schema for `linguini.toml` is exported as
`linguini_config::CONFIG_SCHEMA_JSON`. CI snapshots this schema, generated declarations, CLI help,
and executable examples so public contract drift requires an explicit reviewed update.

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

#### Formatter scope and fallback

The generated formatter is intentionally smaller than the full ICU/ECMA-402
surface. Current guarantees are:

- `@number` formats finite `number`, `bigint`, or decimal/exponent strings. It
  preserves decimal precision up to 8,192 expanded digits and applies the
  locale's CLDR decimal affixes, minimum/maximum fraction digits, primary and
  secondary grouping, decimal separator, group separator, and the locale's
  default numeric numbering-system digits from pinned CLDR data. Percent,
  compact, scientific, significant-digit, and caller-selected numbering-system
  styles are not exposed.
- `@currency` requires a three-letter code. It applies CLDR standard or
  accounting affixes, currency fraction digits, rounding increments, spacing,
  and locale separators. The host `Intl.NumberFormat` supplies only the display
  symbol. Currency display-name/plural forms and cash-rounding selection are not
  exposed.
- `@date` accepts a valid `Date`, epoch-millisecond number, ISO `YYYY-MM-DD`, or
  ISO date-time string. Date-only and zone-less date-time input is interpreted
  in UTC, and output is host-time-zone independent. The supported option is
  `style = "full" | "long" | "medium" | "short"`. Rendering uses the pinned
  Gregorian date patterns plus wide/abbreviated month and weekday names for
  `y`, `M`/`L`, `d`, and `E`. Time fields, time zones, eras, flexible day
  periods, calendars, contexts beyond the compiled symbols, and skeletons are
  not implemented.

Locale lookup first canonicalizes the BCP 47 tag, then walks the pinned CLDR
component fallback chain. Generation fails when a used formatter has no
compiled number, currency, or date data before reaching the allowed locale
fallback; it does not silently substitute host-locale formatting. Malformed or
over-limit decimal strings and invalid dates fail with `RangeError` instead of
rendering an environment-dependent result. JavaScript `NaN` and infinities are
left as their ordinary string spellings.

### Messages

A message is a named entry the app can call. Parameters are typed.

```lgs
delivery(fruit: Fruit, size: Size, count: Number)
greeting(name: String)
```

One or more consecutive `///` comments attach to the declaration immediately
below them. A blank line or ordinary `//` comment detaches the documentation and
is rejected instead of silently moving prose to another symbol. Schema docs are
canonical API prose: they appear in LSP hover for both schema symbols and their
locale implementations, and codegen emits them as escaped JSDoc on generated
groups, values, and both callable overloads. Generated `.d.ts` output preserves
the same documentation.

```lgs
/// Shown on the delivery confirmation card.
delivery(fruit: Fruit, size: Size, count: Number)
```

Generated parameterized messages have two equivalent call forms. The positional
form preserves the schema parameter order. The named-object form uses the same
parameter names, allows properties in any order, and requires exactly the declared
properties:

```ts
l.delivery("apple", "big", 2);
l.delivery({ count: 2, fruit: "apple", size: "big" });
```

This is a generated API overload only; it does not add a second message syntax or
duplicate the message implementation.

### Parameterless messages

A bare identifier inside a namespace block is a message with no parameters.
Empty parentheses are not an alternate spelling: write `label`, not `label()`.
Generated parameterless leaves are string values and cannot be called.

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

```lgs fragment=invalid-duplicate-group-member
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

### Text blocks and literal braces

Inline message text ends at the line break. Use `"""` for a multiline block
whose layout should follow source indentation:

```lgl
receipt = """
  Order {order_id}
    {item_count} items
  Thank you.
"""
```

Dedented blocks remove an opening whitespace-only line, a closing
whitespace-only line, and the exact common horizontal-whitespace prefix of every
non-empty content line. They normalize `CRLF` and bare `CR` line endings to
`LF`; whitespace remaining after the common prefix, including trailing spaces,
is message data.

Use `raw"""` when every whitespace and line-ending byte between the delimiters
is data. Raw blocks do not remove structural edge lines, dedent, or normalize
line endings. Both block modes still parse `{expression}` placeholders.

In inline, dedented, and raw message text, write `{{` for a literal `{` and `}}`
for a literal `}`. A single `{` begins a placeholder. Quoted string fragments
inside inline-function branch text can also carry a literal closing brace.

### Local variables

Use `let` in locale files for reusable local text fragments. Local variables are
not part of the schema API and can be referenced from messages, forms, and
functions in the same locale module.

```lgl
let cart_label = Cart

cart_summary = {cart_label}: {count} {item.nom(count)}
```

### `Plural` — built-in

`Plural` is always available. CLDR plural rules may return
`zero | one | two | few | many | other`, depending on the active locale.
Any `Number` passed to a typed `Plural` parameter converts automatically using
CLDR plural rules for the active locale. In a value-based inline selector, use
`Plural(value)` to request that conversion explicitly; a value already typed as
`Plural` can be selected directly. Intrinsic names are case-sensitive.

Categories are locale rules, not universal meanings. For Russian cardinal
plurals, integer examples include `1` and `21` in `one`; `2`, `3`, `4`, and `22`
in `few`; and `0`, `5`–`20`, and `25` in `many`. A decimal such as `1.5` is
`other`. The `other` arm (or `_`) is still required as the fallback even when an
example deals only with integers.

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

```lgl fragment=property-declarations
Gender = neuter   // property access is inferred as Gender
emoji  = 🍎       // plain string property
```

**Accessing fields and forms in templates:**

```lgl fragment=expressions
fruit.Gender      // inferred enum-valued property access
fruit.nom(count)  // form call — count auto-converts to Plural
```

An `impl` branch-map attribute always uses `form name(Type)`. The `form`
keyword and its single dispatch type are required; implicit plural maps and
bare `name(Type)` spellings are invalid.

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

**Branch coverage:** `linguini check` and `linguini build` require every enum
dispatch level they can resolve to cover each variant explicitly or include
`_`. A `Plural` level must contain `other` or `_`; `other` is the fallback for
CLDR categories without an exact arm. Unknown enum variants and plural category
names are blocking diagnostics. This is scoped analyzer coverage, not a
whole-program exhaustiveness proof.

---

### `fn` — forms with string interpolation

`fn` works like `form`, then adds named payload parameters that can be
interpolated or passed to other locale callables. Leading unnamed enum or
`Plural` parameters are dispatch inputs; other primitive types are rejected as
dispatch dimensions. Named parameters form one trailing payload group and
never add dispatch levels, regardless of their type.

```lgl
fn delivery_note(Plural, Gender, item: String) {
  one {
    feminine  => Доставлена {item}
    _         => Доставлен {item}
  }
  _ => Доставлены {item}
}

delivery = {delivery_note(count, gender, item)}
```

Use `form` when output depends only on grammatical categories.
Use `fn` when output embeds a dynamic string value.

### Inline functions

An inline `fn` is the ordinary locale `fn` without a name. It reuses the same
branch grammar, evaluates immediately, and returns selected text. Its input
list contains expressions rather than type declarations:

- leading unnamed expressions are selectors and define dispatch order;
- trailing `name: expression` entries are local payload bindings and never
  dispatch.

```lgs
enum Gender { masculine, feminine, other }
greeting(name: String, gender: Gender, count: Number)
```

```lgl
greeting = {fn(gender, Plural(count)) {
  masculine {
    one => Dear {name}: one item
    other => Dear {name}: many items
  }
  feminine {
    one => Hello {name}: one item
    other => Hello {name}: many items
  }
  _ {
    one => Hi {name}: one item
    other => Hi {name}: many items
  }
}}
```

Selector types are inferred from their expressions. Here `gender` dispatches as
`Gender`, while `Plural(count)` explicitly converts a numeric value using the
active locale's CLDR rules. If a surrounding locale-function payload already
has type `Plural`, write it directly; inline dispatch still accepts either a
numeric operand or a pre-classified plural category.

Bindings can hold call results, for example
`fn(gender, Plural(count), greet: Greeting(gender, count))`. Binding names are
available in every branch, but bindings are simultaneous: each right-hand
expression sees the surrounding callable scope, not earlier bindings in the
same list. All selectors must precede the first binding. A selector after a
binding is invalid.

Surrounding message or function parameters form the inline lexical closure.
Locale variables, forms, and functions remain ordinary lexical symbols and can
be referenced or called in input expressions and branch text. They are not
turned into ad-hoc selectors. A selector or binding can be omitted when its
value is not needed; unlisted surrounding parameters remain available through
the lexical closure.

Inline functions are expressions, not declarations or JavaScript function
values. Their result type is `String`; enum selector levels must be exhaustive
or contain a final `_`, while `Plural` levels require `other` or `_`. Branch
arms use the same newline and block boundaries as a named locale function;
commas in branch text are ordinary output. Put a literal closing brace in a
quoted fragment (for example, `"result }"`). `{{` still emits a literal opening
brace, consistently with ordinary message text. `fn()` and binding-only inline
functions are valid; with zero selectors, their body has only the structural
`_` branch.

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
- Schema messages with no reference found by the configured application scan

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

Lints are named diagnostics reported by `linguini check`. Six are warnings;
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
| `unused_message`     | warning  | Configured scan finds no reference to message     |

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
declaration = true
gitignore   = true                      # emit generated .gitignore
framework   = "sveltekit"              # enables the bundler integration

[targets.ts.bundler]
sources = ["src"]
# locale_loading = "dynamic"           # eager is the default
```

Generated message access is nested by canonical path. Parameterless leaves are
values (`l.main.title`); parameterized leaves remain callable with their schema
signature (`l.main.greeting("Artemy")`). The Svelte/Vite transform rewrites
static `l` paths to imports of the exact per-message modules used by the source.

Bundler dynamic access is strict by default. To permit a finite computed access,
use the bounded escape below; `allow` entries are exact canonical message paths,
not globs or namespace wildcards:

```toml fragment=dynamic-policy
[targets.ts.bundler.dynamic]
mode = "bundle"
allow = ["main.title", "admin.notice"]
```

`locale_loading = "dynamic"` loads client locale modules on demand and waits
for the selected locale chunk before switching locales. SSR keeps synchronous,
static locale imports. Omit the field for eager loading.

The generated runtime index imports only the base locale statically. Direct
synchronous use of `createLinguini()` for another locale requires an earlier
`await prepareLinguini(locale)`; otherwise it fails with a preparation error.
Svelte and SvelteKit integrations perform this preparation for their normal
switch and request flows. Pending loads are deduplicated, and non-base locale
loaders use bundler-visible dynamic `import()` edges.

Web options use nested tables such as `[web.routing]`, `[web.locale]`,
`[web.cookie]`, `[web.local_storage]`, `[web.links]`, `[web.routes]`, and the
optional `[web.switch_route]`; the legacy flat web fields are not part of the
current config format. Generated projects emit only the selected source, link,
route-matcher, server-cookie, and switch-route capability modules. See
[`web-sveltekit.md`](web-sveltekit.md) for defaults, validation, transport, and
no-JavaScript behavior.

The public web runtime mirrors this structure through `routing`, `locale`,
`cookie`, `localStorage`, `links`, and `routes` objects. Removed flat runtime
properties are not accepted as compatibility aliases. SvelteKit adapters pass
the application base from `$app/paths`; request/browser context supplies the
origin, and SvelteKit remains the sole trailing-slash authority.

### Opt-in unused-message analysis

Application usage is a project-level input, so unused-message analysis is off
until its source boundary is configured explicitly:

```toml fragment=unused-message-policy
[analysis.unused_messages]
sources = ["src", "tests/ui"]
exclude = ["src/generated", "src/vendor"]
ignore  = ["admin.runtime_selected", "experiments"]
```

`linguini check` and the pre-write phase of `linguini build` recursively scan
configured `.js`, `.jsx`, `.mjs`, `.cjs`, `.ts`, `.tsx`, `.mts`, `.cts`,
`.svelte`, `.vue`, and `.astro` files. A configured source may also be one
supported file.
Excludes match an exact project-relative path and every child below it. The
configured TypeScript output directory is excluded automatically. A source
root fully covered by an exclude is invalid. Missing or unreadable sources and
non-excluded symbolic links fail the command instead of producing an unsafe
unused result.

The scanner understands `l`, `lgl`, `messages`, their named-import aliases,
member access such as `linguini.l`, the generated `createLinguini`,
`createLinguiniProvider`, and `configureLinguini` factories, static bracket
access, and template interpolations. Computed access records the narrowest
known prefix; passing a message group or the whole message root as a value
conservatively marks that subtree or project as dynamically used. Add canonical
message paths or group prefixes to `ignore` for dynamic resolution the scanner
cannot see. This is a bounded lexical project scan, not a whole-program
JavaScript type or data-flow proof.
