# Why Linguini

Most i18n tools were designed for English first. They handle simple plurals and
variable interpolation well. They fall apart when a language requires words to
agree with each other in gender, case, and number simultaneously.

Linguini was designed for that problem from the start.

---

## What everyone else does

### JSON + runtime libraries (i18next, react-i18next)

You write keys. You look up keys at runtime. The library parses and interpolates.

The problems compound quickly: keys are stringly typed, arguments are unchecked,
plural forms are either missing or handled through separate keys, and the runtime
ships to every user even when most of it goes unused. There is no compile-time
guarantee that a message exists or that its arguments are correct.

### ICU MessageFormat

ICU handles plurals and gender selection through nested `{select}` and `{plural}`
blocks. It works, but it forces you to write out every combination by hand.

_"Delivered 3 small pears"_ in Russian — verb, adjective, and noun must agree
across gender × plural. In ICU:

```
{size, select,
  small {{count, plural,
    one  {{gender, select, male {маленький} female {маленькая} neuter {маленькое} other {маленький}}}
    few  {{gender, select, male {маленьких} female {маленьких} neuter {маленьких} other {маленьких}}}
    many {{gender, select, male {маленьких} female {маленьких} neuter {маленьких} other {маленьких}}}
    other {{gender, select, male {маленьких} female {маленьких} neuter {маленьких} other {маленьких}}}
  }}
  big {{ ... }}
}
```

24 rows for one adjective. Add a case and it doubles. The format scales
with the size of the combinatorial product, not with the complexity of the
actual language logic.

### Fluent

Fluent introduced a better model: terms carry attributes, and messages can
reference grammatical properties of words. The syntax is more expressive than ICU
for morphologically rich languages.

The issues: Fluent does not compile to typed output. There is no codegen.
Calling a message is a runtime key lookup, not a function call. The Rust library
in particular is difficult to use in practice. And the tooling ecosystem is thin.

### Paraglide

Paraglide got the codegen right. Messages compile to typed ESM functions,
tree-shaking works, SvelteKit integration is first-class. The developer experience
for simple cases is excellent.

The ceiling is low: Paraglide explicitly does not support grammatical agreement.
Pluralization is handled through separate message variants, not through a
language model. For Russian, Polish, Arabic, or any language with non-trivial
morphology, you hit that ceiling immediately.

---

## What Linguini does differently

**Words carry their own grammar.**

Instead of passing gender as a separate argument and managing the mapping
externally, an `impl` block ties a word's grammatical properties and all its
inflected forms together:

```lgl
impl Item {
  pasta {
    Gender = feminine

    form acc(Plural) {
      one => пасту
      few => пасты
      _   => паст
    }
  }
}
```

`item.Gender` is a typed field. `item.acc(amount)` calls an inflection form.
The word knows itself. Nothing leaks into the call site.

**Forms nest by grammatical category, not by combinatorial product.**

Each level of a `form` dispatches on exactly one parameter. `_` collapses all
remaining variants. The 24-row ICU matrix for one adjective:

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

**`Plural` is built-in.**

Pass a `Number` anywhere a `Plural` is expected. CLDR plural rules for the
active locale apply automatically. No wrapper function, no explicit conversion.

**Hard errors, not silent fallbacks.**

Non-exhaustive match expressions are compile-time errors. Type mismatches at
call sites are compile-time errors. Nothing falls back to a wrong string silently.

**The output is native typed code.**

Messages compile to typed functions. Arguments are validated at the call site
by the type system of the target language. Unused messages are eliminated by
the bundler. There is no runtime parser.

---

## Summary

|                             | JSON + runtime | ICU     | Fluent  | Paraglide | Linguini |
| --------------------------- | -------------- | ------- | ------- | --------- | -------- |
| Typed arguments             | ✗              | ✗       | ✗       | ✓         | ✓        |
| Compiled output             | ✗              | ✗       | ✗       | ✓         | ✓        |
| Grammatical gender          | ✗              | ✓       | ✓       | ✗         | ✓        |
| Morphological agreement     | ✗              | verbose | partial | ✗         | ✓        |
| Words carry own grammar     | ✗              | ✗       | partial | ✗         | ✓        |
| Compile-time exhaustiveness | ✗              | ✗       | ✗       | ✗         | ✓        |
