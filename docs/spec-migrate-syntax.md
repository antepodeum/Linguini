# Linguini Syntax Migration Guide

This document describes all syntax changes from the previous version.
Apply these changes to the lexer, parser, analyzer, and code generator.

---

## SCHEMA CHANGES

### 1. Parameterless messages — remove empty parentheses

Messages with no parameters no longer use `()`.
A leaf identifier inside a namespace block is implicitly a parameterless message.

```diff
- email_input {
-   label()
-   placeholder()
-   aria()
- }

+ email_input {
+   label
+   placeholder
+   aria
+ }
```

**Rule:** parentheses are required only when parameters are present.
Disambiguation:

- block `{}` → namespace
- identifier `(params)` → message with parameters
- bare identifier → parameterless message

---

## LOCALE CHANGES

### 2. `enum` — PascalCase name, single-line body allowed

Enum names must be PascalCase. Variants remain lowercase.
Variants may be declared inline on one line.

```diff
- enum gender {
-   male
-   female
-   neuter
-   other
- }

+ enum Gender { male, female, neuter, other }
```

### 3. `Plural` is a built-in enum — do not declare it

`Plural` with variants `one | few | many | other` is always available.
Remove any user-declared `Plural` enum from locale files.

### 4. `form` for noun entries → `impl`

`impl` declares a linguistic implementation for an enum type. Each block inside
corresponds to one variant of that enum. Each variant block may contain:

- typed data fields (`Gender = neuter`, `emoji = 🍎`)
- local `form` declarations that pattern-match on their parameters
- static string blocks (`display { short = ... }`)

The variant name must match a variant declared in the corresponding `enum`.
`impl Fruit` implements the `Fruit` enum — `apple`, `pear`, `orange` must be
valid `Fruit` variants.

```diff
- form Fruit {
-   apple {
-     gender = neuter
-     emoji = 🍎
-
-     nom {
-       one   => яблоко
-       few   => яблока
-       many  => яблок
-       other => яблока
-     }
-   }
- }

+ enum Fruit { apple, pear, orange }
+
+ impl Fruit {
+   apple {
+     Gender = neuter
+     emoji  = 🍎
+
+     form nom(Plural) {
+       one => яблоко
+       few => яблока
+       _   => яблок
+     }
+   }
+ }
```

### 5. Typed fields inside `impl` — `TypeName = value`

Fields whose value is an enum variant are declared with the type name (PascalCase)
as the key. The type is inferred from the capitalized key — no ambiguity with
lowercase string fields.

```diff
- gender = neuter
- emoji = 🍎

+ Gender = neuter
+ emoji  = 🍎
```

### 6. Declension blocks inside `impl` → `form name(Type)`

Bare named blocks for declension now carry an explicit type parameter.
`Plural` auto-converts from `Int` — no wrapping needed at call sites.

```diff
- nom {
-   one   => яблоко
-   few   => яблока
-   many  => яблок
-   other => яблока
- }

+ form nom(Plural) {
+   one => яблоко
+   few => яблока
+   _   => яблок
+ }
```

### 7. `else` → `_`

The catch-all/wildcard arm is now `_` everywhere.

```diff
- else => маленьких
+ _    => маленьких
```

### 8. `form Type { variant:param { ... } }` — removed

The `small:gender` parameterization syntax is removed.
Use a standalone typed `form` instead (see rule 9).

```diff
- form Size {
-   small:gender {
-     male   => маленький
-     female => маленькая
-     neuter => маленькое
-     other  => маленький
-   }
- }
```

### 9. `fn` with flat combinatorial match → `form` with nested dispatch

`fn` is no longer used for grammatical agreement without string parameters.
Replace with `form Name(Type, Type, ...)` using nested enum dispatch.
Each nesting level corresponds to one parameter, matched top-down.
`_` at any level matches all remaining variants of that parameter.

```diff
- fn delivered(gender, plural) {
-   male,   one => Доставлен
-   female, one => Доставлена
-   neuter, one => Доставлено
-   other,  one => Доставлено
-   else        => Доставлено
- }

+ form Delivered(Plural, Gender) {
+   one {
+     male   => Доставлен
+     female => Доставлена
+     neuter => Доставлено
+     _      => Доставлено
+   }
+   _ => Доставлены
+ }
```

```diff
- fn size_label(size, gender, plural) {
-   small, male,   one  => маленький
-   small, female, one  => маленькая
-   small, neuter, one  => маленькое
-   small, other,  one  => маленький
-   small, female, few  => маленькие
-   small, male,   few  => маленьких
-   small, neuter, few  => маленьких
-   small, other,  few  => маленьких
-   small, male,   many => маленьких
-   small, female, many => маленьких
-   small, neuter, many => маленьких
-   small, other,  many => маленьких
-   big,   male,   one  => большой
-   big,   female, one  => большая
-   big,   neuter, one  => большое
-   big,   other,  one  => большой
-   big,   female, few  => большие
-   big,   male,   few  => больших
-   big,   neuter, few  => больших
-   big,   other,  few  => больших
-   big,   male,   many => больших
-   big,   female, many => больших
-   big,   neuter, many => больших
-   big,   other,  many => больших
-   else                => обычные
- }

+ form SizeAdj(Size, Plural, Gender) {
+   small {
+     one {
+       male   => маленький
+       female => маленькая
+       neuter => маленькое
+       _      => маленький
+     }
+     _ => маленьких
+   }
+   big {
+     one {
+       male   => большой
+       female => большая
+       neuter => большое
+       _      => большой
+     }
+     _ => больших
+   }
+ }
```

### 10. `fn` — pattern matching with string parameters

`fn` works like `form` — it can pattern-match on enum variants with nested
dispatch and `_` wildcards. The fundamental difference: `fn` can also accept
string arguments and interpolate them into output values. `form` cannot take
string parameters.

```
// form: only enum types, no string interpolation
form Delivered(Plural, Gender) {
  one {
    female => Доставлена
    _      => Доставлен
  }
  _ => Доставлены
}

// fn: enum types + string parameters, interpolates strings into output
fn delivery_note(item: String, Plural, Gender) {
  one {
    female => Доставлена {item}
    _      => Доставлен {item}
  }
  _ => Доставлены {item}
}
```

Use `form` when output depends only on grammatical categories.
Use `fn` when output embeds dynamic string values.

### 11. Template interpolations — updated call syntax

- `form` and `impl` names are PascalCase at call sites
- `plural(count)` is removed — pass `count` (Int) directly, runtime converts to `Plural` automatically
- Typed field access uses the type name: `fruit.Gender` not `fruit.gender`

```diff
- {delivered(fruit.gender, plural(count))} {size_label(size, fruit.gender, plural(count))} {fruit.nom(count)}

+ {Delivered(count, fruit.Gender)} {SizeAdj(size, count, fruit.Gender)} {fruit.nom(count)}
```

---

## SUMMARY TABLE

| Concept                    | Old syntax                          | New syntax                                              |
| -------------------------- | ----------------------------------- | ------------------------------------------------------- |
| Enum declaration           | `enum gender { male\nfemale\n... }` | `enum Gender { male, female, ... }`                     |
| Built-in plural            | must declare                        | built-in, no declaration needed                         |
| Noun/word entry            | `form Fruit { ... }`                | `impl Fruit { ... }`                                    |
| Typed field                | `gender = neuter`                   | `Gender = neuter`                                       |
| Declension block           | `nom { one => ... }`                | `form nom(Plural) { one => ... }`                       |
| Agreement pattern          | `fn name(args) { flat matrix }`     | `form Name(Type, ...) { nested }`                       |
| Agreement + strings        | `fn name(args) { flat matrix }`     | `fn name(str: String, Type, ...) { nested with {str} }` |
| Gender-param form          | `small:gender { ... }`              | removed, use typed `form`                               |
| Catch-all arm              | `else`                              | `_`                                                     |
| Plural conversion          | `plural(count)` explicit            | automatic from `Int`                                    |
| Field access               | `fruit.gender`                      | `fruit.Gender`                                          |
| Parameterless msg (schema) | `label()`                           | `label`                                                 |

---

## LINTS (`linguini check`)

A clippy-style lint pass runs on top of the analyzer and emits suggestions with
automatic fixes. All lints are `warn` by default and can be suppressed per-block
with `#[allow(lint_name)]`.

---

### `param_order` — order `form` parameters by variant count ascending

Parameters should be ordered from the enum with the fewest variants to the enum
with the most. This keeps the top levels of the dispatch tree narrow and detail
at the bottom, maximising readability.

```
// warn: Gender(4) before Size(2) — suggest reordering
form SizeAdj(Gender, Size, Plural) { ... }

// fix:
form SizeAdj(Size, Plural, Gender) { ... }
```

**Fix:** reorder parameters and restructure nested blocks to match.

---

### `unreachable_arm` — arm after `_` at the same nesting level

Any arm that appears after a `_` wildcard at the same level can never be reached.

```
// warn: `male` arm is unreachable
form F(Gender) {
  _ => foo
  male => bar
}
```

**Fix:** remove the unreachable arm, or move it above `_`.

---

### `collapsible_arms` — multiple arms with identical output

When several variants map to the same value and cover all remaining cases,
they can be collapsed into `_`.

```
// warn: all arms have the same value, collapse into `_`
form F(Gender) {
  male   => маленьких
  female => маленьких
  neuter => маленьких
  other  => маленьких
}

// fix:
form F(Gender) {
  _ => маленьких
}
```

**Fix:** replace the group of identical arms with `_`.

---

### `fn_without_strings` — `fn` with no `String` parameters should be `form`

If an `fn` takes only enum parameters and never interpolates a string value,
it is semantically a `form` and should be declared as one.

```
// warn: no String params, use `form` instead
fn delivered(Plural, Gender) {
  one {
    female => Доставлена
    _      => Доставлен
  }
  _ => Доставлены
}

// fix:
form Delivered(Plural, Gender) {
  one {
    female => Доставлена
    _      => Доставлен
  }
  _ => Доставлены
}
```

**Fix:** replace `fn` with `form` and capitalise the name.

---

### `incomplete_impl` — `impl` does not cover all enum variants

Every variant declared in the `enum` should have a corresponding block in `impl`.
Missing variants will cause a runtime error when that variant is used.

```
enum Fruit { apple, pear, orange }

// warn: missing variant `orange`
impl Fruit {
  apple { ... }
  pear  { ... }
}
```

**Fix:** add a block for each missing variant.

---

### `redundant_wildcard` — `_` when all variants are already covered explicitly

If every variant of the enum is listed as an explicit arm, the `_` arm is
unreachable and should be removed.

```
// warn: `_` is redundant, all Gender variants are covered
form F(Gender) {
  male   => ...
  female => ...
  neuter => ...
  other  => ...
  _      => ...
}
```

**Fix:** remove the `_` arm.
