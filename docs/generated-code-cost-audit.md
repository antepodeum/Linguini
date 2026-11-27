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

## Readable argument-key bindings

Argument-key tables reuse the form binding encoder instead of wrapping an already encoded
function name. For example, `__lgl___lgl_name_account$personalized_keys` becomes
`__lgl_keys_account$personalized`. Dotted components remain readable; a separate escaped-name
marker prevents collisions with dollar signs, Unicode, reserved prefixes, and hex-looking names.
The change adds no operations or allocations to generated code. Current declaration and map
snapshots remain unchanged, and the historical TypeScript comparison oracles are untouched.

Complete executed consumers compared with `4451a49`, using Vite 7.3.3 on Node 24.13.1, retain
these exact minified / gzip / Brotli sizes:

| Consumer | Before and after bytes |
| --- | ---: |
| Russian delivery, counted, and price messages with their dependencies | 5,860 / 2,317 / 2,082 |
| Qualified account greeting | 528 / 374 / 309 |
| English and French greetings | 584 / 381 / 334 |

The Russian and bilingual bundles are byte-identical. The account bundle differs only in a
minifier-selected local name and has identical sizes. This supports the naming change without
a runtime speed or general application-size claim. Existing call observation tests also execute
source message and parameter names resembling the compiler prefix.

## Optional web link operations

Link safety now has one shared TypeScript owner, selected through explicit helpers. Routing
metadata, request contexts, and locale controls omit its four methods; the DOM link adapter
imports the same combined helper. Ordinary href/URL localization shares already parsed path
mutation without reading the environment twice. No runtime or native policy generation is added.

Complete executed Vite 7.3.3 consumers compared with `99f25d4`, on Node 24.13.1, give:

| Consumer | Before minified / gzip / Brotli bytes | After minified / gzip / Brotli bytes |
| --- | ---: | ---: |
| Path locale resolution and href localization | 11,190 / 3,576 / 3,178 | 9,661 / 3,232 / 2,879 |
| Browser locale controls with persistence and navigation | 13,659 / 4,186 / 3,701 | 11,846 / 3,808 / 3,365 |

Executed production Vite 8 graphs verify that routing, controls, and metadata-only SSR hooks
omit `links.js`, while explicit combined link localization retains that owner without browser
traversal. Runtime tests preserve the single environment/URL snapshot, exclusion order and
mutation isolation, short-circuited attributes, error identity, and request direction snapshots.
These are scoped byte and structural results, without a general application-size or speed claim.
Broader platform specialization and exact composed-page delivery remain open.

## Native integer plural inputs

Shared planning now proves when zero fractional/compact operands satisfy every common guard
and the remaining rules need only a decimal integer suffix. For suffixes of at most 15 digits,
safe native integer numbers use `Math.abs(value) % modulus` before the shared category decisions.
They allocate no source string or regex match array. Other inputs retain the original parser,
including category passthrough, caller coercion, visible fractions, and bounded huge inputs.
No CLDR rules or patterns are interpreted at runtime.

The following local comparison uses the `a8fa133` baseline and its regenerated output, Vite
7.3.3, and Node 24.13.1. All 200 integer outputs match for each complete consumer. Timings are
medians of seven interleaved 500,000-call batches after 100,000 warmup calls, cycling 0–199.

| Consumer | Before minified / gzip / Brotli bytes | After minified / gzip / Brotli bytes | Before / after integer-call µs |
| --- | ---: | ---: | ---: |
| Classifier | 1,034 / 560 / 515 | 1,168 / 597 / 545 | 0.117 / 0.008 |
| `Delivered` | 1,324 / 673 / 620 | 1,458 / 714 / 642 | 0.119 / 0.012 |
| Complete `delivery` message | 4,793 / 1,505 / 1,306 | 4,927 / 1,536 / 1,328 | 0.409 / 0.071 |

The page consumer trades 22 Brotli bytes for removing parsing from normal integer calls. String
input timing was 0.429 / 0.417 µs; mixed number/string/bigint input was 0.499 / 0.373 µs. These
measurements justify this bounded fast path, without establishing a general application speed
or memory claim. Executed coercion/resource-limit tests and strict JSDoc checks cover the shared
fallback path. Branch-specific category specialization and repeated normalization remain open.

## Constant plural classifiers

A Japanese `label(count: Number) = {Plural(count)}` consumer generated at `85b4c39` still
converted every finite native number to text and matched a decimal regex before returning
`other`. When the shared decision plan has no remaining guards or category predicates, the
numeric result is known. Every finite ECMAScript Number fits the 8,192-digit expansion limit.
The emitted classifier returns that result directly and omits the now unreachable native-number
regex. Canonical string/bigint parsing, bounds, category passthrough, caller coercion, and errors
remain. An unconditional non-`other` rule also uses its actual compiled result.

The complete message consumer, including argument normalization and an exported classifier,
changes from 1,669 / 817 / 736 to 1,607 / 805 / 714 minified / gzip / Brotli bytes with Vite 7.3.3
and Node 24.13.1. Seven alternating 200,000-call batches after warmup produced:

| Complete message input | Before / after µs |
| --- | ---: |
| Integer | 0.1123 / 0.0091 |
| Fraction/exponent | 0.1797 / 0.0099 |
| String | 0.0925 / 0.1017 |
| Mixed number/string/bigint | 0.1315 / 0.0652 |

The local numeric gain and 22-byte Brotli reduction justify this specialization; string timing
shows no comparable gain. Executed tests cover Japanese, Chinese, Korean, Thai, a constant `one`
model, signed/subnormal/maximal finite numbers, compact exponents, coercion hints/error identity,
category passthrough, and both sides of the 8,192-digit boundary. Spies observe no String
conversion or regex match for finite numbers. This does not establish general application speed,
allocation measurements, selector-specific category demand, or repeated-normalization completion.

## Native integer number and currency inputs

Safe integer numbers and bigint inputs already serialize to canonical integer digits. Their
shared decimal parser now returns those digits directly, retaining the negative-zero sign and
8,192-digit bound. It skips the decimal/exponent match array, expansion, and trimming regexes.
Fractional numbers, unsafe numbers, strings, and custom objects keep the validating parser.
Executed tests disable regex execution for native integer parsing and fixed-precision currency
formatting, check both signed bounds, and preserve caller coercion and thrown error identity.

A complete English number/currency consumer compared with `d69b81c` grows from 3,556 / 1,272 /
1,152 to 3,732 / 1,325 / 1,196 minified / gzip / Brotli bytes on Vite 7.3.3 and Node 24.13.1.
The extra 44 Brotli bytes remove repeated parsing on native integer calls without duplicating
the formatter implementation. Seven alternating 200,000-call batches after warmup produced:

| Complete message input | Number before / after µs | Currency before / after µs |
| --- | ---: | ---: |
| Safe integer | 0.2933 / 0.1402 | 0.2479 / 0.0973 |
| Bigint | 0.3124 / 0.1529 | 0.2682 / 0.1059 |
| Fractional number | 0.3731 / 0.3769 | 0.3852 / 0.3886 |
| Decimal string | 0.3047 / 0.3042 | 0.2772 / 0.2738 |
| Mixed representations | 0.3333 / 0.2577 | 0.3250 / 0.2540 |

These local results support the integer path; they do not establish faster fractional/string
handling or an application-wide speed gain. Before/after execution matches for negative zero,
native integer limits, fractional/exponent inputs, non-finite values, and 8,192-digit bigints;
both versions reject oversized or malformed input. Optional fraction display can still trim
its formatted result, and broader repeated normalization remains open under OPT-A7.

## Unchanged fraction rounding

Unit rounding now returns its canonical input when its fraction already fits the selected
precision. The formatter supplies required visible padding, and unchanged digits bypass optional
trimming. No second decimal record, slice, padding, or trimming regex is needed on that number
path. Truncated/rounded results and non-unit increments retain their exact arithmetic. Fixed
currency output still applies its required padding. Fraction display has one focused canonical
template shared by all number/currency requests.

An English two-message consumer compared with the formatter output at `0f041d5` grows from
3,732 / 1,325 / 1,196 to 3,768 / 1,334 / 1,212 minified / gzip / Brotli bytes. On Vite 7.3.3
and Node 24.13.1, seven alternating 200,000-call batches after warmup give:

| Number message input | Before / after µs |
| --- | ---: |
| Safe integer | 0.1438 / 0.0659 |
| Bigint | 0.1605 / 0.0780 |
| Fractional number | 0.3741 / 0.3544 |
| Decimal string | 0.3142 / 0.2508 |
| Mixed representations | 0.2617 / 0.2017 |

The extra 16 Brotli bytes accompany a roughly halved local integer-call time and about 23%
lower mixed-input time. Currency timing shows no comparable gain: integer calls take
0.0945 / 0.0940 µs, and fractional calls 0.3897 / 0.3998 µs. These are local consumer
measurements, without an application-wide speed or memory claim. A 9,038-result differential
corpus preserves complete message output. The independent rational oracle still compares every
result at its requested precision; separate execution checks record identity, input immutability,
visible minima, ties/carries, increments, and native integer formatting with regexes disabled.

## Routing-only web persistence demand

`createWebLocaleI18n` previously attached cookie serialization and server writes to every
metadata/routing object, even when the application used only path resolution. Serialization now
lives in one shared helper, used by browser controls and server persistence. Server sinks are owned
by the existing `persistLocaleCookie` export; the metadata object retains normalized policy.

A complete consumer exporting path resolution and link localization on Vite 7.3.3 at `51a7daa` shrinks from
12,260 to 11,196 minified bytes, 3,882 to 3,585 gzip bytes, and 3,436 to 3,186 Brotli bytes.
Both versions execute the same path/fallback/link results. A separate Vite 8 graph gate proves
zero rendered bytes from persistence, browser/server transports, and unselected resolver modules,
with one chunk and no imports. Cookie writes preserve append-only headers and cookie-sink
precedence; installed strict TypeScript consumers exercise the explicit helpers and finite locales.
The complete browser-control consumer also drops server cookie sinks while retaining browser
cookie serialization: 14,165 to 13,665 minified bytes, 4,298 to 4,189 gzip bytes, and 3,813 to
3,706 Brotli bytes on the same Vite version. Executed Vite 8 controls retain preparation,
publication, storage/cookie writes, navigation, and override cleanup in order, without server
cookie code in their graph.
Generic SvelteKit hooks/browser controls can still retain their optional transports; broader
platform pruning and exact composed-page delivery remain open under WEB-A13 and MOD-A4.

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
validates the ten Unicode scalars. The emitter uses direct BMP string indexing, consecutive
supplementary code-point arithmetic, or direct branches for other mappings. Each date passes
the same compiled callback to replacement.
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

## Supplementary numeric digits

Native `ff-Adlm` generation exposed numeric replacement indexing supplementary digit strings
by UTF-16 unit. The generated number for `123` contained an isolated surrogate. A shared
scalar-range proof now selects direct code-point arithmetic, and number/date templates reuse
one lookup implementation. Mixed-width/nonconsecutive mappings use generated branches rather
than runtime alphabet parsing or per-call arrays. Executed number/currency cases cover all ten
digits as number, bigint, and string inputs, plus native cold/warm builds and strict JSDoc
consumers. Existing BMP snapshots remain byte-identical. This is a correctness repair, without
a timing claim.

## Form attribute demand

Whole-form dependency selection kept `emoji`, `gen`, and their functions in native output even
when no selected message used them. Shared planning now projects requested form paths. Standalone
messages use that projection directly; projects apply it automatically after message selection.
Pure unused maps and literals disappear before emission. Row objects and ordinary indexed
property access remain; eager initializers and callable rows retain their behavior.

For source `ea82945`, controlled complete-consumer builds compare the same native inputs with
only the shared project projection pass disabled/enabled. Vite 7.3.3 application minification on Node 24.13.1 gives:

| Selected messages | Minified before / after | gzip before / after | Brotli before / after |
| --- | ---: | ---: | ---: |
| English gender attribute only | 1,451 / 546 | 817 / 384 | 726 / 318 |
| English gender and nominative form | 1,539 / 1,439 | 845 / 807 | 759 / 720 |
| Russian delivery, counted, and price | 5,858 / 5,598 | 2,309 / 2,226 | 2,075 / 1,999 |

Every consumer executes matching results before/after and has one chunk without imports.
The native regression fails with the pass disabled. Enabled builds also execute cold/warm
manifests, strict message-record selection, and English/French payloads with a missing French
message resolved at build time. Separate ordinary/physical/standalone execution covers own
attributes, prototype keys, missing and Symbol keys, coercion hints, thrown error identity,
named/positional calls, eager String conversions, and exact callable-row `Object.assign` observations. Current form snapshots omit unused fields;
historical comparison oracles are unchanged.

This initial projection establishes byte savings for the selected project shapes, without a
speed or allocation claim. The physical per-page follow-up below addresses independent
attributes shared across the project. Callable/eager forms remain conservative.


## Independent attributes in composed pages

Physical form modules now emit independent attribute tables as separate named exports, imported
through the existing structured ESM namespace binding. Ordinary and standalone form layouts stay
unchanged. Attribute tables retain indexed variant rows and their own property, preserving lookup,
prototype, coercion, and missing-key behavior. Function bodies use the existing form template;
there is no extra runtime dispatcher or CLDR interpreter.

The same native cold/warm fixture compiles gender-only, nominative-only, self-attribute, callable,
eager, and mixed pages for English/French. Each page/locale has exactly one import-free asset with
only its message records. The gender page omits plural parsing and nominative strings despite
other pages using them; the nominative page omits gender strings. Sparse messages resolve inside
the effective locale payload. Actual Vite 7/8 execution, strict checked physical JavaScript, and
an application TypeScript consumer without `allowJs` preserve finite enum and plural arguments.
Opaque forms retain eager conversions and callable initialization observations. Self references
use local attribute bindings; dependencies on another form use its namespace.

Controlled fresh native builds compare the same inputs with physical attribute splitting
disabled/enabled, retaining shared project pruning in both. Node 24.13.1, Vite 7.3.3 and Vite
8.0.13 application minification give these complete-consumer minified / gzip / Brotli bytes:

| Consumer | Vite 7 before | Vite 7 after | Vite 8 before | Vite 8 after |
| --- | ---: | ---: | ---: | ---: |
| Gender page in a project also using nominative forms | 1,351 / 777 / 688 | 546 / 384 / 318 | 1,318 / 754 / 669 | 527 / 367 / 306 |
| Mixed gender/nominative page | 1,439 / 807 / 720 | 1,458 / 813 / 723 | 1,402 / 782 / 697 | 1,421 / 790 / 703 |
| Russian delivery | 2,487 / 1,148 / 1,002 | 2,515 / 1,161 / 1,010 | 2,450 / 1,129 / 978 | 2,478 / 1,141 / 989 |
| Russian counted | 3,266 / 1,511 / 1,361 | 3,220 / 1,491 / 1,342 | 3,191 / 1,460 / 1,311 | 3,145 / 1,439 / 1,294 |
| Russian delivery/count/price | 5,598 / 2,226 / 1,999 | 5,626 / 2,238 / 2,010 | 5,449 / 2,128 / 1,909 | 5,477 / 2,140 / 1,926 |
| Russian price | 2,850 / 1,421 / 1,291 | 2,850 / 1,421 / 1,291 | 2,751 / 1,357 / 1,227 | 2,751 / 1,357 / 1,227 |

All consumers execute matching results and have one chunk without imports. The native page
regression fails when splitting is disabled. A transposed outer object was discarded: Vite 7
removed its unused field, but Vite 8 retained the unrelated plural code and added eight Brotli
bytes to the gender consumer. Named exports remove that dependency on both backends.

Splitting adds 3/6 Brotli bytes to the mixed English page and increases its table/variant-row
object literals from three to six; it does not duplicate form functions. These are module
initialization structures, with no new lookup helper or per-render allocation. Coalescing tables
for composed uses remains open, as do eager/callable projection, repeated normalization, and
broader request overhead. No application speed or measured heap-allocation gain is claimed.
Property-read side-effect assumptions remain at backend defaults; the
[Rollup option documentation](https://rollupjs.org/configuration-options/#treeshake-propertyreadsideeffects)
explains why disabling them can remove getters and invalid-access errors.

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

## Compiled Gregorian week wording

The canonical `yw` request resolves CLDR's count-specific patterns during generation. Its numeric
week domain is 1–53, so fractional/compact operands and out-of-domain rules disappear. Equal
wording merges, and only live patterns survive. Nine pinned locale tags require different live
wording; the others reduce to one pattern. Calendar arithmetic runs directly in the selected
formatter and shares its year rollover work, without an additional calendar helper call.

Literal-only alternatives also share the numeric fields. A Maltese formatter has one conversion
of the week and year around a direct wording choice. Factoring preserves field order and literal
digits, and retains first-use modulo evaluation. The raw-pattern oracle covers 413 years and
17 locale tags; supplied-Date getter counts and caller error identity remain covered. All twelve
locale outputs used by strict consumers, including the nine nonconstant tags, pass checked JS
and application TypeScript without `allowJs` on TypeScript 5.8.3, 5.9.3, and 6.0.3.

Complete executed consumers, Node 24.13.1, minified / gzip / Brotli bytes:

| Selected locale | Vite 7.3.3 | Vite 8.0.13 |
| --- | ---: | ---: |
| `en-GB` | 1,600 / 870 / 765 | 1,529 / 818 / 711 |
| `fo` | 1,632 / 906 / 802 | 1,561 / 852 / 751 |
| `ff-Adlm` | 1,651 / 939 / 809 | 1,585 / 890 / 768 |
| `mt` | 1,617 / 888 / 782 | 1,551 / 842 / 735 |

Every consumer contains one self-contained chunk. Compared with the initial internal calendar-
helper prototype, inlining saves 10–25 Brotli bytes in the first three profiles. Factoring Maltese
wording then saves 58 minified bytes and 3/2 Brotli bytes on Vite 7/8; the three constant-pattern
profiles retain their sizes. These are output-size measurements, with no timing claim. General
message-selector outcome specialization, month weeks, other calendars/zones, and exact composed-
page delivery remain under remediation.

The first-weekday (`fw`) correction changes build-time constants only. Ten locale outputs pass
strict JS/application TS on TypeScript 5.8.3, 5.9.3, and 6.0.3. Eight complete Vite 7/8 consumers
execute as one self-contained chunk each. `en-GB-u-fw-mon` retains `en-GB`'s identical formatter
bytes and the 1,600/870/765 and 1,529/818/711 consumer sizes above; other weekdays use the same
compiled arithmetic with their selected constants. This adds no runtime locale parser or
preference dispatch, and makes no timing claim.

This audit supplements the [fixture review](./generated-code-review.md). It does not close the
remaining optimization, payload-delivery, or calendar/time-zone/skeleton requirements.
