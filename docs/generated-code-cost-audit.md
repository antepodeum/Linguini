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

A follow-up against `3051bdb` compared complete tree-shaken Russian consumers on Vite 7.3.3
and Node 24.13.1. The prototype compiled branch-specific classifiers and shared a proven scalar
integer suffix; it introduced no operand objects. Both consumers executed their expected outputs.

| Consumer | Existing minified / gzip / Brotli bytes | Prototype minified / gzip / Brotli bytes |
| --- | ---: | ---: |
| `Delivered` only | 1,296 / 665 / 603 | 1,286 / 645 / 576 |
| `Delivered` plus callable fruit forms | 3,312 / 1,019 / 910 | 3,489 / 1,046 / 934 |

The mixed consumer still grew by 177 minified bytes and 24 Brotli bytes. Seven timing batches
of 500,000 calls showed only small differences; they do not establish a useful speed gain.
The implementation was discarded. GEN-A15 remains open: specialization must account for
coexisting selector shapes and whole-consumer costs, including normalization sharing, before
it replaces the current classifier. A smaller isolated classifier is insufficient evidence.

## Shared date name tables

The `f5052c8` skeleton fixture repeated the same abbreviated month array for `GyMMMd` and
`yMMMd`. Sharing the complete equal vector across the locale runtime removes that duplicate
array initialization. Distinct widths and Russian grammatical contexts retain their own values;
supplied `Date` month and weekday getter counts are unchanged.

A Vite 7.3.3 bundle exporting both date formatters shrinks from 1,598 to 1,502 minified bytes,
694 to 684 gzip bytes, and 651 to 634 Brotli bytes. Both versions execute the expected CE/BCE
dates. This is a two-formatter fixture measurement. Separate real bundles prove that retaining
the table does not retain the unused era formatter and that time-only output contains no month
tables. No runtime table registry, option dispatcher, or CLDR pattern parser is introduced.

## Generated locale text references

Shared build-time `StringResultFacts` identifies locale text variables before artifact projection.
Initializers, imported globals, form branches, functions, and messages can use those strings
directly; caller parameters and unproven member references retain coercion. Executed ordinary
and physical modules record the required raw conversions, string coercion hints, and thrown
errors. No runtime fact table or lookup is emitted.

A Vite 7.3.3 consumer exporting the baseline Russian `counted` message shrinks from 5,445 to
5,437 minified bytes, 1,829 to 1,826 gzip bytes, and 1,636 to 1,633 Brotli bytes. Both bundles
execute the same expected text. This one-consumer measurement supports removing its redundant
conversion; it does not establish a general speed improvement or close GEN-A11.

## Compiled date digit substitution

The former date helper constructed and checked a symbol array from a fixed CLDR alphabet on
every numeric field, then allocated a capturing replacement callback. Shared Rust planning now
validates the ten Unicode scalars. The emitter uses direct BMP string indexing or one static
supplementary-symbol table, and each field passes the same compiled callback to replacement.
A shared proof coalesces substitution into one pass when literals and all possible textual
symbols are free of ASCII digits. Literal digits and quarter names such as `Q1` retain numeric-
field substitution, with executed BMP and supplementary-symbol cases. ASCII-only date demand
emits no digit helper.

A native-generated Arabic consumer retaining short date, long date, and date-time messages
shrinks on Vite 7.3.3 from 3,911 to 3,557 minified bytes, 1,345 to 1,298 gzip bytes, and 1,166 to
1,127 Brotli bytes. Both versions execute identical pinned output. Instrumented execution of the
three messages records zero symbol arrays and one callback identity, replacing eleven arrays
and eleven callback identities. Three whole-result substitutions replace eleven field substitutions.
Supplementary digits, Date/string/timestamp inputs, and supplied
Date getter counts also execute explicitly. This does not establish a general timing improvement.

## Findings and owners

| ID | Status | Verified evidence | Required remediation |
| --- | --- | --- | --- |
| GEN-A15 | Open | `js-baseline/locales/ru/_globals.js` compares the full `pluralRu` result with `one` in `Delivered`, `DeliveryNote`, and `SizeAdj`; callable fruit forms distinguish one/few/fallback. `expr.rs` and `requirements.rs` currently request a single complete classifier. | Add shared selector-outcome demand analysis; merge equivalent category outcomes and specialize only the required rules/operands. Evaluate numeric internal outcomes with complete consumer/page bytes, execution, and allocation evidence. Preserve public category names and distinguish counts from already classified internal values. |
| GEN-A16 | Fixed | `shared.jinja::normalizeMessageArgs` requests `.filter(...).map(String).sort()` over own keys even when a valid named argument has no unknown keys. This shape occurs in eight scanned fixture copies. | Unknown keys are collected lazily; conversion and sorting occur only on errors. Successful named calls retain only the own-key snapshot and result array. Executed arities 0–128 count collection operations, and observation tests preserve separate classification/conversion phases, sorted errors, symbols/non-enumerable keys, getter/Proxy order, missing-key positional fallback, and frozen inputs. A local seven-batch Node comparison covered arities 1, 2, 8, 32, and 128; it does not establish a general speed improvement. No persistent sets/tables are added. |
| GEN-A17 | Fixed | `codegen-js-formatters-ru.js` tested `fraction ? ... : integer` after a fixed two-digit currency round. Integer-only plans also retained fraction/formatted scaffolding. Owner: `formatters/number.jinja`. | Mandatory fractions now concatenate directly; non-scientific integer-only plans omit fraction/formatted bindings and localize/group the integer directly. Identical sign prefixes are selected during generation. Executed USD/JPY/KWD, optional/significant fractions, negative zero, localized digits, equal prefixes, and the existing grouping/padding/rounding corpus cover the change. |
| GEN-A18 | Fixed | `date_helpers.jinja` validates an ISO date-time's calendar fields by constructing a `Date` it then discards, before constructing the final date-time. Instrumented generated code constructs two dates for `2024-01-02T13:04:05Z`, one for a date-only value, and one before rejecting February 30. | Gregorian field validation now uses arithmetic before constructing the returned date. Executed tests cover all 10,000 four-digit years against native calendar rollover, three host time zones, year zero, leap centuries, offsets, midnight rollover, malformed/time-invalid inputs, and observable supplied Date methods. Valid text values construct one Date; invalid calendar fields construct none; supplied Date objects retain identity and construct none. |
| GEN-A19 | Fixed for proven carry plans | An executed scientific number-plan probe renders `9.9` as `1.0E01` with zero allowed fraction digits. Owner: `formatters/number.jinja`. | Shared `NumberRoundingPlan` proves when rounding a normalized mantissa can only carry to ten: significant rounding, unit rounding, and increments dividing the scaled ten boundary. The emitter uses a literal one and compiled zero suffix, preserving selected precision without runtime digit slicing/concatenation or shift calculation. Executed fixed/optional/minimum/significant/increment plans cover precisions 0–6, signed carries, positive/negative exponents, zero, and number/bigint/string inputs. The build-time proof is checked against exact powers for 38 scales × 256 increments and at 255 fraction digits. Unproven increments retain their general normalization path; this does not establish full engineering-notation support. |
| GEN-A20 | Fixed | Russian plural output tested the complete `many` predicate and then returned `other`, although its earlier fraction guard and one/few branches leave only many. | Shared plural planning proves exhaustive integer-modulo coverage across a complete finite period, with a conservative 4,096-residue build budget. The emitter replaces the final test with a return and recomputes operand/modulus demand after removing it. A mixed-modulus execution probe drops an unused modulo-30 operation while preserving 1,001 counts across number, bigint, string, negative, and fractional representations, category-name passthrough, and invalid-input rejection. Mixed/noninteger/unbounded operands and excessive periods retain their original test. The existing 24-locale/operator and huge/exponent-input corpus passes. |

The allocation changes pass all 746 workspace Rust tests on stable 1.99.1, strict workspace
Clippy, minimum-version verification on Rust 1.87, all 23 checked JS/JSDoc/TypeScript consumer
gates, and the eight site gates including real-browser SSR/hydration and payload requests.

Repeated plural normalization across calls and exact composed-page delivery remain the earlier
GEN-A11 and GEN-A10 findings. Repeated date getters cannot be blindly cached: the current date
coercer returns a supplied `Date` object, so overridden getters can be observable. Raw input
coercion and validation also remain necessary; they are not runtime parsing of CLDR patterns.

This audit supplements the [fixture review](./generated-code-review.md). It does not close the
remaining optimization, payload-delivery, or calendar/time-zone/skeleton requirements.
