# Generated-code representation and allocation audit

The reviewed baseline is `ee5a575` (2026-10-28). The snapshot inventory contains 901 files,
including 197 JavaScript files with 125 distinct byte contents. A targeted scan of 71 message,
formatter, declaration-companion, and complete-project JavaScript fixture candidates was traced
back to the current generators. Retired web/Svelte implementations and historical TypeScript
comparison fixtures are not evidence of current production costs.

## Plural names versus numeric results

A prototype replaced the Russian classifier's category results with codes `0` through `5`,
while retaining acceptance of the six category names as inputs. It was compared with the
checked classifier on Node 24.13.1. Each timing used seven batches of 500,000 calls after
warmup, cycling through integers 0–199. The dispatch consumer selected one/few/fallback
results; it did not measure an entire application.

| Measurement | Category names | Numeric prototype |
| --- | ---: | ---: |
| Classifier median | 0.140 µs | 0.138 µs |
| Classifier plus dispatch median | 0.146 µs | 0.142 µs |
| Heap retained by two million results in an ordinary array | 15,986,328 bytes | 16,002,064 bytes |
| Minified module, Vite 7.3.3 | 1,098 bytes | 1,180 bytes |
| Minified module, gzip | 580 bytes | 598 bytes |
| Minified module, Brotli | 547 bytes | 547 bytes |

These local measurements do not establish a meaningful memory saving from replacing string
literals with numeric returns. The small timing difference is insufficient to justify a general
speed claim. They also do not rule out a better implementation of numeric dispatch. Minified
size includes the classifier's public category-name input handling, not its consumers.

The useful next boundary is a compiled selector outcome, rather than a global representation
change. For example, `Delivered` distinguishes `one` from its fallback, but the shared classifier
still evaluates and distinguishes `few`, `many`, and `other`. A selector plan can merge categories
that reach the same branch, remove irrelevant category rules/operands, and assign compact internal
outcome codes. Equivalent plans should share an implementation when that saves total page bytes.

Public `Plural(value)` results and accepted category-name inputs must keep their documented
meaning. Raw numeric counts and internal category codes cannot share an unqualified numeric
argument: code `3` and count `3` are different concepts. Lowered inline selectors, callable forms,
custom imported plural functions, named functions, and string interpolation all need explicit
representation/provenance handling. Simply changing the return literals would break them.

## Findings and owners

| ID | Status | Verified evidence | Required remediation |
| --- | --- | --- | --- |
| GEN-A15 | Open | `js-baseline/locales/ru/_globals.js` compares the full `pluralRu` result with `one` in `Delivered`, `DeliveryNote`, and `SizeAdj`; callable fruit forms distinguish one/few/fallback. `expr.rs` and `requirements.rs` currently request a single complete classifier. | Add shared selector-outcome demand analysis; merge equivalent category outcomes and specialize only the required rules/operands. Evaluate numeric internal outcomes with complete consumer/page bytes, execution, and allocation evidence. Preserve public category names and distinguish counts from already classified internal values. |
| GEN-A16 | Open | `shared.jinja::normalizeMessageArgs` requests `.filter(...).map(String).sort()` over own keys even when a valid named argument has no unknown keys. This shape occurs in eight scanned fixture copies. | Keep the successful path free of unknown-key result arrays and error-only sorting. Preserve the separate validation/conversion phases, sorted errors, symbols/non-enumerable keys, getter/Proxy order, missing-key positional fallback, and frozen inputs. Benchmark small and larger arities before adding persistent per-message sets or tables. |
| GEN-A17 | Fixed | `codegen-js-formatters-ru.js` tested `fraction ? ... : integer` after a fixed two-digit currency round. Integer-only plans also retained fraction/formatted scaffolding. Owner: `formatters/number.jinja`. | Mandatory fractions now concatenate directly; non-scientific integer-only plans omit fraction/formatted bindings and localize/group the integer directly. Identical sign prefixes are selected during generation. Executed USD/JPY/KWD, optional/significant fractions, negative zero, localized digits, equal prefixes, and the existing grouping/padding/rounding corpus cover the change. |
| GEN-A18 | Open | `date_helpers.jinja` validates an ISO date-time's calendar fields by constructing a `Date` it then discards, before constructing the final date-time. Instrumented generated code constructs two dates for `2024-01-02T13:04:05Z`, one for a date-only value, and one before rejecting February 30. | Validate Gregorian calendar fields with direct arithmetic and create only the returned date. Retain leap-century/year-zero behavior, invalid-date exceptions, offsets, midnight boundaries, native Date input behavior, and host-time-zone independence. Prove constructor counts and executed output together. |

Repeated plural normalization across calls and exact composed-page delivery remain the earlier
GEN-A11 and GEN-A10 findings. Repeated date getters cannot be blindly cached: the current date
coercer returns a supplied `Date` object, so overridden getters can be observable. Raw input
coercion and validation also remain necessary; they are not runtime parsing of CLDR patterns.

This audit supplements the [fixture review](./generated-code-review.md). It does not close the
remaining optimization, payload-delivery, or calendar/time-zone/skeleton requirements.
