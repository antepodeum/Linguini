# Linguini

**A localization language for products that outgrew JSON, ICU strings, and runtime i18n glue.**

Linguini turns localization from a pile of fragile translation keys into a typed, compiled, grammar-aware layer of your product.

> i18n should not be string replacement.  
> i18n should be a language.

Linguini is built to solve the whole i18n stack at once: schemas, locale files, plural rules, grammatical forms, typed placeholders, validation, code generation, formatting, editor tooling, and runtime safety.

---

## Why i18n is still painful

Most localization systems look simple until the product becomes real.

You start with this:

```json
{
  "delivery": "Delivered"
}
```

Then you need plurals, gender, grammatical cases, currency formatting, placeholders, missing translation checks, namespace overrides, generated frontend APIs, safe refactors, and editor support.

So the stack turns into JSON/YAML files, ICU MessageFormat strings, runtime validators, custom scripts, linter hacks, spreadsheet workflows, and tribal knowledge.

That is why i18n breaks:

- translation keys drift from code;
- placeholders are missing or mistyped;
- plural branches are incomplete;
- grammar logic is duplicated in strings;
- translators edit unreadable mini-programs;
- errors appear at runtime instead of build time;
- refactoring localization is dangerous.

Linguini exists to end that entire class of problems.

---

## The Linguini idea

Linguini splits localization into two explicit layers:

- **`.lgs` schema files** define what the product can say.
- **`.lgl` locale files** define how each locale says it.

The schema is the contract.  
The locale is the implementation.

That means translations can be parsed, checked, formatted, compiled, and edited with real language tooling.

---

## Project structure

```text
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

Global locale files live at `linguini/locale/{locale}.lgl`. Namespace-specific locale files live under the matching namespace, for example `linguini/locale/shop/ru.lgl`.

This keeps localization modular without turning it into an unstructured key-value dump.

---

## Example

### Schema

```linguini
// linguini/schema/shop/delivery.lgs

enum Gender {
  male
  female
  neuter
}

delivery(count: Number, gender: Gender)
```

### Locale

```linguini
// linguini/locale/shop/ru.lgl

form Delivered(Plural, Gender) {
  one {
    male   => Доставлен
    female => Доставлена
    neuter => Доставлено
    _      => Доставлено
  }
  _ => Доставлены
}

delivery = {Delivered(count, gender)}
```

This is the core difference: plural and gender agreement are not hidden inside a fragile string. They are modeled directly, so tooling can understand them.

---

## What Linguini gives you

**Typed localization.** Messages have parameters. Parameters have types. Locale files must match the schema.

**Grammar as data.** Plurals, gender, cases, forms, fallbacks, and locale-specific rules become visible and checkable.

**Compile-time safety.** Missing messages, invalid placeholders, incomplete branches, and schema/locale mismatches can be caught before release.

**Generated APIs.** Application code should call typed functions instead of raw translation keys:

```ts
l.shop.delivery({ count: 3, gender: "female" });
```

**Readable locale files.** Translators and developers review structured localization logic instead of decoding ICU spaghetti.

**Editor tooling.** Linguini is designed for syntax highlighting, diagnostics, autocomplete, formatting, hover, references, rename, and quick fixes.

---

## What it replaces

| Old i18n stack            | Linguini                     |
| ------------------------- | ---------------------------- |
| JSON/YAML key dumps       | schema + locale language     |
| ICU string puzzles        | readable forms and branches  |
| runtime missing-key bugs  | analyzer diagnostics         |
| untyped interpolation     | typed placeholders           |
| scattered plural logic    | first-class grammar modeling |
| custom validation scripts | built-in project checks      |
| fragile string keys       | generated APIs               |
| editor guesswork          | LSP + VS Code tooling        |

Linguini is not trying to be a nicer JSON format.  
It is trying to make i18n a real programming-language problem — and then solve it properly.

---

## Status

Linguini is under active development.

Already started or partially implemented:

- `.lgs` schema files;
- `.lgl` locale files;
- `linguini.toml` project config;
- parser and analyzer;
- enums, messages, forms, branches, placeholders;
- missing locale/message checks;
- formatter prototype;
- TypeScript codegen prototype;
- CLDR plural groundwork;
- LSP prototype;
- VS Code extension prototype.

Planned:

- production-grade formatter;
- complete semantic analyzer;
- full CLDR integration;
- richer grammatical modeling;
- stable TypeScript API;
- framework adapters;
- translation workflow tooling;
- safe project-wide refactors;
- dead key detection;
- package/module support;
- mature editor experience.

Some features described here are the target design, not a claim that every piece is already production-ready.

---

## The promise

Localization should have the same engineering guarantees as code: types, validation, formatting, refactoring, generated APIs, and build-time errors.

**Linguini is the language for that.**

No more magic keys.  
No more ICU spaghetti.  
No more silent broken translations.

Just correct text, generated from a system that understands language.
