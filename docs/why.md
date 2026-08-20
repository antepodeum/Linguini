# Why Linguini

Most i18n tools were designed for English first. They handle simple plurals and
variable interpolation well. They fall apart when a language requires words to
agree with each other in gender, case, and number simultaneously.

Linguini was designed for that problem from the start. This page explains that
direction; comparisons are architectural summaries, not a cross-tool
conformance benchmark. Where Linguini's own behavior is not covered by an
executable repository test, treat the stated advantage as a goal rather than a
production guarantee.

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

When the analyzer resolves `Gender` as an enum, `item.Gender` is treated as an
enum-valued property in locale expressions. The property value itself is not
yet checked against the enum variants. `item.acc(amount)` calls an inflection
form, keeping the grammatical lookup with the word.

**Forms nest by grammatical category.**

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

**Scoped checks before generation.**

`linguini check` and `linguini build` block missing implementations and missing
branches in enum/`Plural` dispatches the analyzer can resolve. They also reject
arity and type mismatches in locale function/form calls when both sides have a
known type. Generated TypeScript exposes schema argument types to the host type
checker. When `[analysis.unused_messages]` explicitly defines application
source roots, both commands also perform a conservative project scan and warn
for schema messages that have no static, dynamic-prefix, or ignored use. These
checks remain bounded and are not a whole-program proof.

**Generated TypeScript carries schema signatures.**

Messages compile to typed TypeScript functions and declarations, allowing a
host TypeScript checker to validate application call arguments. There is no
runtime message parser.

For Svelte/Vite targets, the bundler transform rewrites static message paths to
imports of the exact generated module each path uses. Because the browser graph
uses those per-message imports instead of the eager message barrel, standard
bundler tree-shaking can leave unrelated message modules out of the bundle.

### Exact browser module graph

Linguini supplies module boundaries; Vite and Rollup decide final chunk names,
merging, preload, and transport. For each configured application source scope,
the plugin records the canonical static message leaves used by that scope and
rewrites them to physical per-message ESM modules. Computed access is rejected
unless its exact allowed leaves are configured.

With dynamic locale loading, each application scope exposes one virtual locale
entry per effective locale. The route has one literal dynamic-import edge to
each entry, and that locale payload indexes only the messages referenced by the
scope. Locale formatter code is shared instead of copied into every message.
This is the graph checked by the production-site manifest test; it is not a
promise that every bundler version will choose identical output files.

### Evidence and payload claims

No fixed percentage or universal “smaller bundle” claim is part of the public
contract. A payload comparison must identify the repository commit, package and
bundler versions, configuration, locale/message corpus, route, build mode, and
whether raw, minified, gzip, or Brotli bytes are measured. It must preserve the
build manifests and measurement command so another run can reproduce the
result. `pnpm test:site` checks the production graph and exact route message set;
size claims require an additional controlled before/after artifact comparison.

---

## Summary

The Linguini column describes the implemented, scoped behavior documented in
this repository. The other columns are a conceptual comparison, not results
from the repository's conformance suite.

|                             | JSON + runtime | ICU     | Fluent  | Paraglide | Linguini |
| --------------------------- | -------------- | ------- | ------- | --------- | -------- |
| Typed arguments             | ✗              | ✗       | ✗       | ✓         | ✓        |
| Compiled output             | ✗              | ✗       | ✗       | ✓         | ✓        |
| Grammatical gender          | ✗              | ✓       | ✓       | ✗         | ✓        |
| Morphological agreement     | ✗              | verbose | partial | ✗         | ✓        |
| Words carry own grammar     | ✗              | ✗       | partial | ✗         | ✓        |
| Checked locale branches     | ✗              | ✗       | ✗       | ✗         | scoped ✓ |

“Checked locale branches” means schema/locale enum and `Plural` dispatches that
the analyzer can resolve; it does not mean whole-program exhaustiveness.
