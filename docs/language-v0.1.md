# Linguini language specification v0.1

Status: frozen preview language contract. Version: `0.1`.

This document fixes the source-language boundary implemented by the `0.1.x` toolchain. The
detailed examples and behavioral explanations in [Language Reference](./reference.md) are
normative where they describe this version. A source-breaking grammar or semantic change requires
a new language version and a migration entry; adding diagnostics that reject a previously invalid
program does not.

## Source units

Linguini source is UTF-8. A schema unit has the `.lgs` extension. A locale unit has the `.lgl`
extension. Comments start with `//`; consecutive `///` comments attach documentation to the next
declaration only when no blank line or ordinary comment intervenes.

The maximum brace nesting depth is 64. Identifiers, delimiters, and keywords use the grammar
implemented by `linguini-syntax`; source spans are UTF-8 byte ranges with a source identity.
Malformed input may produce a recovered tree for diagnostics, but recovered declarations are not
valid program input.

## Schema grammar

A schema file is a sequence of these declarations:

- `enum Name { variant, ... }` declares a finite enum. Enum names are PascalCase and variants are
  lowercase identifiers.
- `type Name = Target` declares an alias. It may carry one formatter annotation.
- `message` or `message(name: Type, ...)` declares a message signature. Parameterless messages use
  a bare name; empty parentheses are invalid.
- `group { ... }` recursively contains messages and groups. Schema groups may be empty.

Primitive types are exactly `String`, `Number`, `Decimal`, `Date`, and `Boolean`. `Plural` is a
selector type produced from numeric values, not a host-language primitive. Formatter annotations
are `@number`, `@currency(code = "AAA")`, and `@date(style = "full|long|medium|short")` with the
type restrictions documented in the reference.

Sibling message and group names share one namespace. Duplicate declarations, duplicate enum
variants, duplicate parameters, alias cycles, unknown types, and message/group collisions are
invalid.

## Locale grammar

A locale file is a sequence of these declarations:

- `enum Name { variant, ... }` declares locale-owned selector metadata.
- `override` before a declaration explicitly replaces an inherited declaration of the same kind.
- `let name = pattern` declares a reusable string value.
- `form Name(Type, ...) { branches }` declares a typed selector function without named string
  inputs.
- `fn Name(Type, name: Type, ...) { branches }` declares a typed function. Unnamed leading inputs
  dispatch; named trailing inputs are lexical string/value bindings.
- `impl Enum { variant { ... } }` associates properties and forms with enum variants.
- `message = pattern` implements a schema message.
- `group { ... }` recursively contains message implementations and groups. Locale groups may not
  be empty.

Branch keys are enum variants, CLDR plural categories, or `_`. A wildcard is last and covers all
remaining values. Nested branch blocks represent the next dispatch input. Multi-key form branches
are preserved as independent selectors.

Patterns contain text and `{expression}` placeholders. Expressions distinguish references from
calls, support qualified paths, formatter annotations, and the inline `fn(inputs) { branches }`
expression. Inline inputs are selector expressions followed by optional named bindings. Inline
functions form lexical closures over surrounding parameters and bindings.

Inline text ends at its structural line boundary. `"""..."""` is a dedented semantic block;
`raw"""..."""` preserves content bytes. `{{` emits a literal opening brace. Quoted fragments can
contain structural delimiters as text.

## Names and resolution

The schema file path relative to `paths.schema`, including its stem, forms the filesystem
namespace. A locale file's parent path relative to `paths.locale` forms the same namespace; its
stem is the locale tag. Recursive source groups extend message paths. `project.name` never prefixes
language symbols.

The filesystem namespace scopes every declaration kind. Canonical dotted paths identify messages,
enums, aliases, variables, forms, and functions across analysis, IR, diagnostics, LSP operations,
and code generation. Linguini v0.1 has no source-level import or export declaration.

## Static semantics

Valid projects satisfy all of these rules before code generation:

- every locale implements required schema messages in its matching namespace;
- every reference resolves to the correct declaration kind;
- calls satisfy arity and all resolvable argument types;
- enum and plural selector trees are exhaustive;
- wildcard and duplicate arms are reachable and unambiguous;
- implementations name declared enum variants and required forms;
- formatter names, arguments, and input types are valid;
- variable and helper dependency graphs are acyclic.

The named diagnostics are `param_order`, `unreachable_arm`, `collapsible_arms`,
`fn_without_strings`, `incomplete_impl`, `redundant_wildcard`, and `unused_message`.
`incomplete_impl` is an error; the others are warnings. There is no inline suppression syntax in
v0.1.

## Generated host contract

The v0.1 JavaScript boundary is ESM-only. CommonJS and a module-format configuration switch are not
part of this language version. Parameterless messages are string values. Parameterized messages
have positional and named-object TypeScript/JSDoc call forms backed by one generated implementation.
Generated declarations, runtime modules, and the bundler namespace transform use the same schema
types and canonical paths.

Language semantics end at validated IR. Framework adapters, locale persistence, routing, package
layout, and output-file ownership are integration contracts documented separately.

## Conformance

`tests/fixtures/golden/syntax/all.lgs` and `all.lgl` are the positive grammar corpus for this
version. `tests/fixtures/invalid` is the negative recovery corpus. Parser tests execute both, and
the documentation conformance gates execute every registered Linguini block in the reference and
guides. The formatter additionally proves semantic preservation and idempotence over the supported
syntax.
