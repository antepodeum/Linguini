# Language Reference

This reference describes the frozen [Linguini language specification v0.1](./language-v0.1.md).
Any source-breaking change requires a new language version and migration entry.

The committed JSON Schema for `linguini.toml` is exported as
`linguini_config::CONFIG_SCHEMA_JSON`. CI snapshots this schema, generated declarations, CLI help,
and executable examples so public contract drift requires an explicit reviewed update.

`linguini-codegen` owns backend-independent type models, formatter and plural requirement
analysis, exact formatter requests, closure-free inline-function lowering, static decision trees,
and direct/transitive message dependency selection with source provenance. Other code
generators can consume these semantic plans without depending on an output language.
`date_field_symbols` resolves Gregorian month/weekday contexts and widths, AM/PM values, and era
labels through the pinned calendar data before emission. `CalendarFormatData::symbol` provides
the shared data lookup; generated JavaScript receives only the selected literals/name tables.
`day_period_plan` resolves fixed (`b`) and flexible (`B`) day periods into disjoint intervals and
optional noon/midnight points. Missing labels fall back during generation, and adjacent equal
intervals merge. ECMAScript templates emit direct scalar comparisons and literals, reading only
needed UTC clock fields once. Exact points use the precision displayed by the pattern; quoted
field letters do not change that precision. Midnight refers to the start of the displayed day.
Gregorian `y` uses the year within its era (1 BCE is year 1), shares the original year with `G`,
and pads to the requested minimum width. Only `yy` truncates to the last two digits. Year-only
padding requests select their padding helper during generation.
`date_request_pattern` selects/composes the requested widths, and `compile_date_pattern`
parses CLDR quoting and validates supported field widths before target emission. UTC zone
fields become selected localized name/GMT-zero literals or the specified ISO-zero literals;
adjacent literals merge. Unsupported fields and unclosed quotes return structured errors with
UTF-8 byte offsets. Generated formatters receive no raw pattern or zone-name catalog.
See [UTS #35 v48.2 day-period rules](https://www.unicode.org/reports/tr35/tr35-78/tr35-dates.html#Day_Period_Rules).

Web and framework integrations have separate runtime owners.
`@antepodeum/linguini-web` owns reusable URL, resolver, cookie, and request operations.
`@antepodeum/linguini-sveltekit` supplies runtime hooks and request context. These TypeScript
packages are configured in application code.

`linguini-codegen-svelte` emits project imports and types backed by the reusable
`@antepodeum/linguini-svelte` runtime. The ECMAScript backend remains independent of framework
and web adapters. SvelteKit server hooks are configured through its npm runtime package.

`@antepodeum/linguini-project` owns frontend configuration reading and project-relative path resolution.
Its `./ecmascript` entry derives the same layout for asynchronous Vite and synchronous `tsc` hosts,
including generated/source roots, locale loading and finite dynamic uses. Frameworks
and compiler APIs remain in their adapters. The language-neutral entry has no framework imports.
It reads neutral project paths and target settings. Runtime adapter configuration is loaded by
the application and its ordinary frontend module graph.

`linguini-codegen-vite::VitePlan` owns physical message/semantic leaves and manifest generation.
The host compiler supplies validated application usage and explicit runtime-helper descriptors;
Vite planning has no Svelte or SvelteKit dependency. A plan can emit into multiple output roots,
with source-map paths rebased through the shared portable-path calculation. Integration templates
use `EcmaTemplateEngine` for the same strict context and literal-escaping policy as the backend.

Public parameter types lower into `linguini_codegen::TypeModel`, also re-exported by
`linguini-codegen-ecmascript`. The ECMAScript backend owns identifier escaping, TypeScript and
JSDoc rendering through `render_typescript_type` and `render_jsdoc_type`, templates, and module
emission.

Rust codegen integrations can call `generate_schema_files` to obtain the schema-owned
`shared.js` checked-JavaScript runtime and `messages.js` type boundary together with their `.d.ts`
declaration companions. `EcmaGeneratedFile` records each generated path and its contents.
Runtime compilers emit JavaScript; TypeScript consumers use adjacent declarations from the same
semantic type and signature models. Separate TS implementation emitters have been removed.
Physical message compilers reuse the project emitter's constant/function templates, and module
exports and imported JSDoc typedefs share syntax templates across locale and semantic artifacts.
Rust selects the semantic plan and source spans; executable statement syntax belongs in templates.
Declaration signatures and JSDoc are rendered from the common message-call model through
build-time templates. The schema-owned contract and locale declarations share namespace-type
rendering with per-message and group provenance.
Generated internal bindings retain readable source names and qualified components when safe.
Reserved compiler prefixes and ambiguous identifier spellings are encoded deterministically to
prevent collisions with source declarations. These internal names are not a public import API.

JavaScript output with declaration companions can be consumed by strict TypeScript projects
without `allowJs`. Declaration imports use explicit runtime `.js` paths for NodeNext and
Bundler resolution. A build that emits application code with `tsc` must also deliver the
generated JavaScript files to its runtime output directory.
The [TypeScript build adapter](./typescript.md), `@antepodeum/linguini-tsc`, integrates native
generation with regular, incremental, solution, and watch compilation. It prepares generated
declarations before checking dependent projects and delivers runtime code after successful
compilation, including message-only updates that TypeScript considers up to date.

Build integrations can pass repeatable `--emit-dir DIRECTORY` arguments to `linguini build`.
The configured source output remains available for type checking, and each additional
project-relative directory receives generated modules, declarations, and source maps from the
same validated project. Maps resolve the original Linguini files from each output depth.
Output directories must be disjoint and follow the configured output's ownership and path rules.
Builds preserve unchanged owned files and their timestamps, including declarations and the
ownership manifest. Text-only message edits therefore do not invalidate TypeScript's type inputs.
Changed and stale files still use the ownership-checked transaction and rollback; missing owned
files are restored on the next build.

`linguini build --emit-declaration-dir DIRECTORY` updates only `.d.ts` artifacts and their maps
in an additional output root. It retains existing owned runtime artifacts there, removes stale
owned declarations, and preserves unowned files. This lets a compiler integration prepare types
for referenced projects before compilation, then deliver runtime code after successful checking.
Declarations must be enabled for this option; the usual containment and ownership rules apply.

Custom Rust codegen callers configure an embedded plural implementation through
`EcmaOptions::plural_rules`. The backend renders those canonical rules directly for the
JavaScript export scope; callers use `plural_import` only when the plural
function is supplied by another module.

Bundler integrations enumerate `locale_runtime_artifacts` from a validated project, then compile
each artifact with `compile_locale_runtime_artifact_module`. Runtime artifacts share helper-demand analysis,
canonical CLDR inputs, artifact source identities, and source-map ownership.

Integrations that need aggregate locale symbols enumerate `locale_globals_artifacts`, then compile
each artifact with `compile_locale_globals_artifact_module`. Local enums, variables, forms, functions,
runtime-helper imports, checked JSDoc, and source maps come from the same semantic models used
by physical semantic leaves. Locales without global symbols do not produce an artifact.

Locale implementation integrations enumerate `locale_artifacts`. Each effective locale produces
its selected namespace artifacts followed by one barrel artifact; metadata retains deterministic
paths and ordered fallback-composed source identities. Compile with
`compile_locale_artifact_module` to keep
message values/callables, positional and named calls, recursive namespace exports, runtime/global
imports, defaults, and source maps consistent across direct and bundler emission.
Namespace modules import only the global values directly referenced by their messages.
Re-export-only barrels and static namespaces omit global runtime imports. The shared
`direct_message_dependencies` and `direct_symbol_dependencies` planners distinguish direct
edges from transitive dependencies, follow aliases, and preserve simultaneous inline-input
scope. Alias-typed form references use the resolved form's runtime binding while declarations
retain the source alias.

Project entry integrations enumerate `project_artifacts` on a validated project. The locale
metadata artifact precedes the public index artifact. Compile either with
`compile_project_artifact_module` for
`locale.js`/`index.js` code and source maps. The index loads the base locale eagerly,
prepares other locales on demand, deduplicates concurrent loads, and permits retry after failure.
Generated-only entry maps have no source records. `generate_project_files` renders the same
artifact plan without map companions; `compile_ecmascript_project_files(project, sources)` adds
owned source maps and validates all source identities before rendering.

For selected web route exclusions, `compile_web_routes_module` renders `matchesRoute` behavior with owned
source maps. The generated project keeps its existing `web/routes.js` and declaration paths.
For selected path locale resolution, `compile_web_path_module` renders `resolvePathLocale` behavior with owned
source maps. The generated project keeps its existing `web/path.js` and declaration paths.
For selected cookie locale resolution, `compile_web_cookie_module` renders `resolveCookieLocale` behavior with
owned source maps. The generated project keeps its existing `web/cookie.js` and declaration
paths.
For selected local-storage locale resolution, `compile_web_local_storage_module` renders `resolveLocalStorageLocale` behavior
with owned source maps. The generated project keeps its existing `web/local-storage.js` and
declaration paths.
For selected Accept-Language resolution, `compile_web_accept_language_module` renders `resolveAcceptLanguageLocale`
behavior with owned source maps. The generated project keeps its existing
`web/accept-language.js` and declaration paths.
Preference selection scans the parsed header directly, preserving highest-specificity exclusions,
first matching input order, quality ranking, base-locale preference, and configured locale order.
It normalizes ranges/locales once and allocates no filtered/sorted candidate arrays per locale.
For transformed links, `compile_web_link_transform_module` renders `localizeTransformedHref` behavior
with target-owned imports and source maps. The generated project keeps its existing
`web/link-transform.js` and declaration paths.
For browser runtime links, `compile_web_runtime_links_module` renders the owned observer and bounded traversal from
the shared template. It returns owned source maps; generated project paths remain
`web/runtime-links.js` and `web/runtime-links.d.ts`.

Rust codegen callers can compile Svelte locale state with
`linguini_codegen_svelte::compile_svelte_locale_module`, selecting
`SvelteLocaleMode::{Context, Standalone, SvelteKit}`. JavaScript output retains framework-owned
`.svelte.js` runes, checked JSDoc, and an adjacent source map. The locale-loader registry and state implementation have one canonical JS body.

`compile_svelte_effects_module` accepts
the closed `EcmaWebOptions` policy and explicit `SvelteHost` bindings. It emits selected, guarded browser
reads, framework-owned base/browser imports, runtime-link lifecycle ownership, and HMR disposal
from one implementation. With no locale sources, initialization uses the known base locale and
omits capability readers. Browser imports/checks are selected independently for locale reads and
runtime links. Manual/transform link modes retain no-op refresh/destroy exports while omitting
the unused effects object, locale getter import, and HMR registration.

`compile_svelte_control_module` compiles
lightweight controls from the same closed policy and host bindings. Navigation, cookie writes,
and storage writes are included only for selected capabilities. Controls prepare messages before
mutating locale state, tolerate denied persistence, and do not import the eager message runtime.
Controls without path/cookie capabilities omit switch-option construction; selected transports
receive only their needed defaults. The optional options argument, callable arity, and navigation
state forwarding remain available.

`compile_svelte_module` selects context-only or
web-enabled Svelte facades with a boolean flag. The template owns message providers, live locale getters, forwarded controls, and deterministic
HMR loader disposal.

`compile_web_server_cookie_module`
compiles server cookie delegation. `compile_web_switch_route_module` accepts the closed web policy and returns `None` when
no switch route is configured or its path has no locale marker. Project validation requires exactly
one marker. The compiler splits the route into a static prefix and suffix; requests only prepend
the host base path and extract the encoded locale. Empty suffixes omit suffix checks. These
modules preserve same-origin return safety and selected path/cookie transports.

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

The compiler resolves each used formatter request against pinned CLDR data at build time.
Generated JavaScript contains specialized functions for those requests, with static symbols,
precision, grouping, affixes, and date/time widths compiled into the implementation. Calls pass
only their input value; generated code carries no CLDR pattern interpreter or runtime option
object. Unused formatter requests and helper operations are omitted. Scaling, scientific
notation, significant rounding, and significant padding are selected independently from the
compiled numeric plans. Rounding helpers embed their precision and increment, share identical
plans, and avoid revalidating those static choices at runtime. Unit rounding only constructs a
bigint when a carry is necessary. Fixed-width currency fractions skip optional-digit trimming.
Scale shifts and significant-digit widths are compiled into selected helpers. Date padding and
digit helpers are selected from the actual compiled fields. The regression corpus checks each
operation in isolation against executed generated code and an exact rational rounding oracle.
Significant rounding retains the expansion bound for precision derived from the input value;
static CLDR choices are validated during generation.
Date symbol selection distinguishes formatting `M` from standalone `L`, narrow month/weekday
names, short weekday names, and locale/width-specific AM/PM and Gregorian era labels. Identical
name tables within a formatter share one binding. These choices follow the
[pinned CLDR date-field rules](https://www.unicode.org/reports/tr35/tr35-78/tr35-dates.html#Date_Field_Symbol_Table).
Formatting `E`/named `e` weekday symbols and standalone `c` symbols retain their own contexts.
Full Finnish and Inari Sami dates therefore use their standalone weekday names.
Extended calendar, zone, and skeleton compilation remains under remediation;
the public date request currently selects `style` and `time_style`.
Numeric grouping uses a direct string loop compiled for the selected primary/secondary widths,
with separate output for equal widths and no group array, reversal, or join. Executed cases cover
width boundaries, padding, Unicode separators, negative zero, and the decimal input-size limit.
Known mandatory fractions concatenate directly. Non-scientific integer-only plans omit the
fraction and formatted-value bindings, retaining grouping, digit localization, signs, and padding.
Optional and significant-digit plans retain their required runtime decisions.
Plural predicates also compile during builds: only required scalar operands are produced at their
first use, common guards are shared, and modulo results are calculated once. Comparisons use
direct conditions without per-call operand objects or range arrays. Fraction-digit counts do not
construct decimal strings or bigints. When all integer relations inspect decimal moduli, the shared
plan selects only the needed suffix digits. Suffixes up to 15 digits use exact number arithmetic;
wider suffixes and full integer/fraction values use bigint arithmetic. The largest selected modulus
is omitted because the suffix is already its remainder. Input bounds and visible-fraction semantics
are preserved, and discarded leading digits or large zero suffixes are never expanded into bigints.

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
  and locale separators. Display symbols also come from pinned CLDR data during
  generation, so the output does not query host `Intl.NumberFormat`. Currency
  display-name/plural forms and cash-rounding selection are not exposed.
- `@date` accepts a valid `Date`, epoch-millisecond number, ISO `YYYY-MM-DD`, or
  ISO date-time string. Date-only and zone-less date-time input is interpreted
  in UTC, and output is host-time-zone independent. `style` selects a CLDR date
  width and `time_style` selects a CLDR time width; each accepts `"full"`,
  `"long"`, `"medium"`, or `"short"`. Supplying only `time_style` emits only
  the time, while supplying both combines them with the matching CLDR
  date-time pattern. Rendering uses pinned Gregorian patterns plus
  context/width-specific month and weekday names, eras, and localized fixed/flexible
  day periods. Time fields are evaluated in UTC; zone-bearing widths use pinned localized
  UTC names, falling back to the localized GMT-zero label when the selected name is absent.
  These labels are compiled literals. Unsupported CLDR fields fail generation. Selectable
  time zones, non-Gregorian calendars, and
  skeleton selection remain under remediation.

```lgl
created_at = {created @date(style = "long", time_style = "short")}
```

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
the same documentation. Consecutive comment lines remain one documentation block;
empty `///` lines preserve paragraph breaks, and embedded `*/` text is escaped safely. Codegen adds
typed `@param` and `@returns` tags from the schema signature without treating generated tags as
source prose.

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

### Text, quotes, and escapes

After `=` or a branch's `=>`, the first non-whitespace characters select the text
mode. If the value starts without a quote or multiline delimiter, it is unquoted
text through the line break. Quotes later in that text are ordinary characters,
including an unmatched quote; backslashes do not introduce escapes:

```lgl
said = He said "hello".
screen = The screen is 6"
path = C:\new\test
greeting = Hello {name}, "friend".
```

Unquoted text trims its outer whitespace. Interior whitespace, quotes, `//`,
`=`, and `=>` are text. `{expression}` still interpolates a value, and structural
closing braces still end a containing declaration/inline branch. Write comments
before a declaration rather than after its unquoted value.

Start with `"` to write an explicitly quoted string. Delimiting quotes are
removed, whitespace inside them is preserved, and escapes are decoded. String
fragments can be joined with placeholders outside the quotes:

```lgl
escaped = "\"hello\"\n\t\u{1F34E}"
preserved = "  keep these spaces  "
greeting = "Hello "{name}"!"
literal_braces = "{name} is literal here"
```

| Escape in a quoted string | Result |
| --- | --- |
| `\"` | Double quote |
| `\\` | Backslash |
| `\n`, `\r`, `\t` | Line feed, carriage return, tab |
| `\0` | NUL |
| `\u{HEX}` | Unicode scalar with one to six hexadecimal digits |

Unknown escapes, surrogate/out-of-range Unicode values, and unterminated quoted
strings are errors. A quoted string cannot cross a source line break; use a
multiline block for that. Braces inside a quoted string are literal, so
`"Hello {name}"` does not interpolate `name`. The same string escapes apply to
quoted annotation arguments.

### Multiline text and literal braces

Inline message text ends at the line break. Use `"""` for a multiline block
whose common source indentation should be removed, like `trimIndent`:

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

```lgl
wire = raw"""
  first line
    second line
  """
```

This raw value starts with a newline and retains the two spaces before its
closing delimiter. Replacing `raw"""` with `"""` produces
`first line\n  second line`, with no edge newline. Dedenting removes an exact
common whitespace prefix; tabs are never expanded into spaces. A placeholder
counts as content when determining indentation, before its runtime value is
inserted.

Both block modes preserve ordinary `"`/`""` quotes and backslashes literally:
`\n` inside a block is a backslash followed by `n`. The next `"""` closes the
block. To include that exact delimiter in the output, reference a quoted local
value:

```lgl
let triple_quotes = "\"\"\""
explanation = """
  Delimiter: {triple_quotes}
"""
```

In ordinary inline message text and both multiline modes, write `{{` for a
literal `{` and `}}` for a literal `}`. A single `{` begins a placeholder.
Inside an inline-function branch, a closing brace is structural; use a
quoted-start branch such as `_ => "result }"`, or a multiline block with `}}`,
when the branch's output needs a literal closing brace. Multiline blocks work
in messages, local values, form attributes, and function branches.

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

[targets.ts.bundler]
sources = ["src"]
# locale_loading = "dynamic"           # eager is the default
```

Use the `@antepodeum/linguini-sveltekit/vite` adapter for SvelteKit project bindings.
Configure web resolution and routing through the npm runtime adapters in application code;
see [Web and SvelteKit](./web-sveltekit.md).

Generated named/inline function calls, callable form branches, plural intrinsic calls, and
formatter results already produce strings, so interpolation does not wrap them in `String` again.
Raw user inputs retain runtime coercion, including inputs whose public schema type is `String`.
Form objects and nested dispatch bodies use multiline indentation in generated source.

Message argument keys are initialized once per module. Positional normalization reuses the
existing argument array; named-object normalization validates all own keys before reading values
in schema parameter order. Inherited properties do not satisfy required named parameters.

Generated message access is nested by canonical path. Parameterless leaves are
values (`l.main.title`); parameterized leaves remain callable with their schema
signature (`l.main.greeting("Artemy")`). The Svelte/Vite transform rewrites
static `l` paths to imports of the exact per-message modules used by the source.
Static property or method access after a known message leaf preserves the JavaScript
operation: `l.main.title.length`, `l.main["title"].toUpperCase()`, and
`l.main.items.call(undefined, 3)` still import only their message leaf. Prefix spans come
from native token analysis, preserving quoted members, comments, and UTF-8 source maps.
Unresolved named namespace imports that resolve to the generated entry are diagnosed
instead of retaining a catalog barrel. Imports resolving to unrelated modules are unaffected.
A parameterized leaf can also be captured as a function, for example
`const format = l.main.items; format(2)`. Capturing it does not invoke it;
stateful bindings select the current locale when the captured function is called.

Bundler dynamic access is strict by default. To permit a finite computed access,
use the bounded escape below; `allow` entries are exact canonical message paths,
not globs or namespace wildcards:

```toml fragment=dynamic-policy
[targets.ts.bundler.dynamic]
mode = "bundle"
allow = ["main.title", "admin.notice"]
```

Stateful dynamic locale registries require top-level await. When an adapter selects a
`localeHelper` and `locale_loading = "dynamic"`, the Vite plugin defaults an unset
`build.target` to `es2022` on all supported Vite versions (5–8). Explicit application
targets are preserved and must support top-level await. Eager loading and the default
framework-neutral `lgl` binding retain Vite’s default target. That binding selects the
base locale statically and imports only the referenced compiled message leaves; it
requires no locale helper, lookup, wrapper, or loader. Finite computed access retains
a bounded dispatch object for its explicitly allowed messages.

Generated TypeScript and checked JavaScript use the same dispatch and message-argument
normalization bodies. Dispatch checks own branches before `_` and `other` fallbacks;
named arguments reject unknown own keys, including symbols.
Numeric, currency, date, and digit formatting also share runtime bodies across targets;
each locale artifact includes only its required helpers and pinned CLDR data.
Project locale resolution and lazy message providers likewise share runtime bodies,
with target-specific type annotations and native ESM import paths.
Plural operand extraction uses the same model, retaining decimal precision and CLDR
compact-exponent semantics without separate TypeScript and JavaScript implementations.

`locale_loading = "dynamic"` loads client locale modules on demand and waits
for the selected locale chunk before switching locales. SSR keeps synchronous,
static locale imports. Omit the field for eager loading.

The generated runtime index imports only the base locale statically. Direct
synchronous use of `createLinguini()` for another locale requires an earlier
`await prepareLinguini(locale)`; otherwise it fails with a preparation error.
Svelte and SvelteKit integrations perform this preparation for their normal
switch and request flows. Pending loads are deduplicated, and non-base locale
loaders use bundler-visible dynamic `import()` edges.

Web routing, locale-source selection, and persistence belong to `@antepodeum/linguini-web` and
`@antepodeum/linguini-sveltekit` runtime adapters, configured in application TypeScript and hooks.
Native codegen emits messages, selected CLDR operations, declarations/maps, and thin Svelte
project bindings. It emits no web or SvelteKit runtime implementation. Neutral TOML rejects
`[web]` and `targets.ts.framework`. See [Web and SvelteKit](./web-sveltekit.md).

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

Web and SvelteKit request behavior is implemented in the npm runtime adapters. Their runtime
sources and declarations are TypeScript, with package tests for URL/cookie policy, request
isolation, streaming HTML, typed consumers, and real Svelte rune compilation.

`compile_ecmascript_project_files(project, sources)` enumerates the language-owned JavaScript
artifacts and optional declarations through the common artifact plan. The CLI explicitly composes
thin Svelte import/type bindings from their owner. It emits an owned map beside
every implementation; declarations follow the validated declaration setting. Source paths are
relative to the output root and are rebased to each map directory. Missing/duplicate source
identities and output collisions fail before filesystem writes. The established
`generate_project_files` API renders source-only output from that same artifact plan.
The CLI now consumes mapped project artifacts through its existing atomic ownership transaction.
JavaScript namespace typedefs retain the full message API even when declarations are disabled.
The integrated fixture is `tests/fixtures/golden/snapshots/js-complete-project`.

Locale declaration barrels now re-export their imported namespace bindings directly. This fixes
strict declaration checking that previously reported an import/local declaration collision and a
self-referential namespace type; the public export names and value types remain unchanged.

### Editor and build-time highlighting

Linguini files are highlighted by the recoverable Rust lexer and the shared schema/locale
semantic walk. `linguini-schema::highlight_tokens` classifies those tokens and occurrences;
LSP converts their source spans to UTF-16 positions. VS Code uses these semantic tokens,
including lexical colors while a declaration is incomplete. There is no separate TextMate
language grammar. Highlighting requires the matching server and semantic highlighting enabled.
Markdown Linguini fences forward their content to the same highlighter and map positions back
to the Markdown document.

`linguini highlight example.lgl` (or `.lgs`) emits versioned JSON with a token legend and
zero-based UTF-16 `line`, `start`, and `length` values and named token `type` fields. The
`linguini/highlight` LSP request accepts `language` (`lgs`, `lgl`, or the full language ID) and
`source` strings and returns the same response. LF, CRLF, and CR line endings use the same
coordinates. Both paths enforce document safety limits.

Site builds run the native highlighter and render escaped HTML into the ignored `.svelte-kit`
directory. Neither native token output nor generated message bundles are committed.
`pnpm --dir site highlight:build` generates current tokens and rendered examples together.
Shiki handles the Svelte example only.
