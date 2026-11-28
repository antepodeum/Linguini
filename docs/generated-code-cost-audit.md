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
| GEN-A19 | Fixed for proven carry plans | An executed scientific number-plan probe renders `9.9` as `1.0E01` with zero allowed fraction digits. Owner: `formatters/number.jinja`. | Shared `NumberRoundingPlan` compiles literal carry results. GEN-A82 corrects its quantum proof to require divisibility of the carried mantissa itself; dividing the pre-shift ten boundary was insufficient. The emitter uses a literal one and compiled zero suffix, preserving selected precision without runtime digit slicing/concatenation or shift calculation. Executed fixed/optional/minimum/significant/increment plans cover precisions 0–6, signed carries, positive/negative exponents, zero, and number/bigint/string inputs. The build-time proof is checked against exact powers for 38 scales × 256 increments and at 255 fraction digits. GEN-A82 also replaces unproven normalization with original-mantissa rerounding and adds fixed/engineering width plans. |
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
message-selector outcome specialization, other calendars/zones, and exact composed-
page delivery remain under remediation.

The first-weekday (`fw`) correction changes build-time constants only. Ten locale outputs pass
strict JS/application TS on TypeScript 5.8.3, 5.9.3, and 6.0.3. Eight complete Vite 7/8 consumers
execute as one self-contained chunk each. `en-GB-u-fw-mon` retains `en-GB`'s identical formatter
bytes and the 1,600/870/765 and 1,529/818/711 consumer sizes above; other weekdays use the same
compiled arithmetic with their selected constants. This adds no runtime locale parser or
preference dispatch, and makes no timing claim.

Region (`rg`), subdivision (`sd`), and ISO week preferences likewise resolve before emission
(GEN-A60). The two added validity lookups belong to the pinned Rust compiler data; generated
JavaScript does not carry them. Executed `yw` and `MMMMW` tests compare formatter bytes for
equivalent region, subdivision, ISO, and weekday preferences. All existing application goldens
retain their bytes. Corrected conventions change selected arithmetic constants and reachable
week outcomes without adding runtime preference parsing or dispatch. This is a correctness
and structure check, with no new bundle-size or timing claim.

The obsolete-subdivision follow-up (GEN-A61) adds the complete pinned validity source to the
offline maintainer tool. Regular lookup arms and other existing compiler records remain
identical. Executed regular/obsolete preference pairs emit identical `yw`/`MMMMW` formatter
bytes; raw-pattern checks also cover zero/six wording and constant Sanskrit without cardinal
rules. The registry stays in compiler data, and XML parsing runs only during offline generation.
Generated JavaScript gains no helper, preference parser, or validity table. No new timing or bundle-size claim
is made for this data-coverage correction.

A source enum or alias named `Record` previously broke the helper's utility-type annotation.
Its anonymous index-signature replacement preserves public names and runtime statements.
Actual before/after generated enum consumers execute in one self-contained chunk each and
retain byte-identical bundled code: 522/372/304 minified/gzip/Brotli bytes on Vite 7.3.3 and
503/356/289 on Vite 8.0.13. Both permanent enum/alias fixtures pass strict checked JS and
application TS without `allowJs` on TypeScript 5.8.3, 5.9.3, and 6.0.3. This is a typing
correction with no runtime work or timing claim.

Numeric helper typedefs also use reserved compiler names, preserving source enums named
`GeneratedNumeric`/`GeneratedDecimal` in standalone output. The actual generated before/after
module is identical after removing JSDoc; executed complete consumers retain byte-identical
bundles at 1,957/1,028/929 minified/gzip/Brotli bytes on Vite 7.3.3 and 1,896/995/904 on
Vite 8.0.13. Both consumers contain one self-contained chunk. Source parameter and enum types
pass the permanent checked-JS/application-TS contracts on TypeScript 5.8.3, 5.9.3, and 6.0.3.
No formatter statements, runtime parsing, or runtime type dispatch are added.

A locale variable named `undefined` previously corrupted standalone numeric sentinels. Sharing
private value bindings fixes that behavior without a wrapper, runtime lookup table, or extra
formatter branch. Complete before/after consumers execute finite inputs; the before version
throws on NaN and the after version matches ordinary/physical output. Each remains one
self-contained chunk. Measured minified/gzip/Brotli bytes:

| Bundler | Before | After |
| --- | ---: | ---: |
| Vite 7.3.3 | 1,927 / 1,016 / 918 | 1,925 / 1,016 / 916 |
| Vite 8.0.13 | 1,866 / 984 / 885 | 1,864 / 983 / 890 |

This is a correctness fix, not a uniform compression improvement: Vite 8's Brotli output grows
by five bytes. There is no timing claim. Permanent lexical-scope and positive/negative typed
consumers retain source keys and public signatures on TypeScript 5.8.3, 5.9.3, and 6.0.3.

Public export/helper collisions are resolved with local binding names and static ESM aliases.
The new native namespace and SDK public-value consumers execute on Vite 7.3.3 and 8.0.13 in
one self-contained chunk each, with no imports or dynamic imports left in the bundle. Measured
minified/gzip/Brotli bytes on Node 24.13.1:

| Consumer | Vite 7.3.3 | Vite 8.0.13 |
| --- | ---: | ---: |
| Six native namespaces | 3,017 / 1,335 / 1,212 | 2,932 / 1,327 / 1,207 |
| Five SDK public values | 2,167 / 1,082 / 974 | 2,098 / 1,052 / 956 |

The previous generated modules fail to import because of duplicate bindings, so these are
post-fix consumer sizes, not a before/after compression improvement or timing claim. Existing
non-colliding runtime, declaration, and source-map snapshots remain byte-identical. The fix adds
no runtime registry, wrapper function, or CLDR interpretation.

Canonical month-week (`MMMMW`) formatting reuses the finite wording planner. Its arithmetic
reads the month, day, and weekday once each, without year/leap work or a calendar helper call.
The minimum-days convention restricts the plural domain before emission: one-day first weeks
cannot reach zero, and first weeks requiring at least three days cannot reach six. Equal
wording and month tables share the existing specialization and literal-choice models.

Twelve complete generated locale outputs pass strict checked JS and application TS without
`allowJs` on TypeScript 5.8.3, 5.9.3, and 6.0.3. Executed consumers each contain one
self-contained chunk. Node 24.13.1 minified/gzip/Brotli bytes:

| Locale | Vite 7.3.3 | Vite 8.0.13 |
| --- | ---: | ---: |
| `en-GB` | 1,488 / 844 / 726 | 1,429 / 798 / 684 |
| `fa` | 1,617 / 970 / 814 | 1,558 / 922 / 773 |
| `mt` | 1,540 / 871 / 765 | 1,481 / 826 / 721 |
| `zh` | 1,490 / 857 / 746 | 1,431 / 811 / 689 |

This extends a previously rejected request; these are post-change consumer sizes, not a
before/after improvement or timing claim. Existing formatter/declaration/map goldens retain
their bytes. New curated month-week fixtures join the strict date-consumer gate.

A week-date family with equal compiled count variants no longer requires cardinal-rule data.
The planner proves equality during generation and emits one unconditional pattern. Existing
formatter/declaration/map goldens are byte-identical; the new Sanskrit fixture contains both
families, localized digits, and the selected abbreviated month vector without a classifier.

Four generated locale outputs (`sa`, `ccp`, `aa`, `la`) pass strict checked JS/application TS
on TypeScript 5.8.3, 5.9.3, and 6.0.3. Complete consumers using both year/week and month/week
messages execute as one self-contained chunk each. Node 24.13.1 minified/gzip/Brotli bytes:

| Locale | Vite 7.3.3 | Vite 8.0.13 |
| --- | ---: | ---: |
| `sa` | 2,073 / 1,108 / 933 | 1,999 / 1,066 / 889 |
| `ccp` | 2,302 / 1,127 / 949 | 2,228 / 1,088 / 908 |

These requests previously failed to build. The figures are post-fix consumer sizes, with no
before/after size or timing claim. Differing wording and missing fallback records still fail;
there is no assumed runtime category or new runtime fallback.

Public classifiers with intrinsic/helper names use static export aliases. After the expected
function-name/export substitution, the generated executable statements match the ordinary-name
classifier exactly. Both variants execute through a consumer exporting the same `classify`
API, in one self-contained chunk each:

| Bundler | Ordinary name: minified / gzip / Brotli | Protected custom name |
| --- | ---: | ---: |
| Vite 7.3.3 | 901 / 530 / 478 | 901 / 530 / 478 |
| Vite 8.0.13 | 884 / 520 / 468 | 884 / 520 / 468 |

Vite 7 chooses different minified local names while retaining identical sizes. Vite 8 produces
byte-identical bundles. Numeric/category results and invalid-value errors execute on Node
24.13.1. These are equality measurements against an equivalent valid classifier name, not a
before/after improvement for the previously broken name, and make no timing claim.

Programmatic IR type keywords now receive safe type identifiers without changing value names
or runtime expressions. Permanent enum/alias projects cover 21 names, qualified short aliases,
compiler-prefix separation, and positive/negative type contracts. Checked JS and application TS
without `allowJs` pass on TypeScript 5.8.3, 5.9.3, and 6.0.3.

Controlled before/after SDK projects for an enum named `string` and an alias named `undefined`
execute through complete consumers as one self-contained chunk each. Both cases retain
byte-identical bundles: 489/350/291 minified/gzip/Brotli bytes on Vite 7.3.3 and 470/334/274 on
Vite 8.0.13, measured on Node 24.13.1. The before versions render correctly but fail TypeScript
checks; the correction is to the type contract. There is no runtime registry, wrapper, or
classification added, and no timing claim.

## Native Date annotations beside source types

An unqualified SDK source enum named `Date` captures the generated date input alias, rejecting
native instances in TypeScript although a forwarded explicit date formatter executes them.
Qualified `globalThis.Date` input/result annotations share one host-type template. The source
`Date` enum and programmatic `globalThis` type keep their own exports and finite values.

The permanent fixture checks ordinary, physical, and standalone execution and strict positive/
negative consumers on TS 5.8.3, 5.9.3, and 6.0.3. Eleven changed existing JavaScript goldens have
identical executable syntax after comments are removed; existing source-map goldens and frozen
comparison runtimes are unchanged.

A complete SDK consumer calls its named function's short-date formatter through the generated
locale message. Both versions execute Date instances and ISO text in one self-contained chunk.
App builds on Node 24.13.1 retain byte-identical before/after bundles:

| Bundler | Before minified / gzip / Brotli | After minified / gzip / Brotli |
| --- | ---: | ---: |
| Vite 7.3.3 | 1,461 / 798 / 698 | 1,461 / 798 / 698 |
| Vite 8.0.13 | 1,398 / 750 / 652 | 1,398 / 750 / 652 |

The correction adds no runtime expression or operation and makes no timing claim.

## Shared digit replacement callbacks

Number and currency formatters created a replacement callback per invocation; scientific
formatting created separate callbacks for its mantissa and exponent. A build-time binding pool
now interns digit alphabets. Numeric requests and dates reuse the same compiled lookup function;
different alphabets retain independent functions, and ASCII-only output emits none.

A controlled native project contains number, USD currency, and date messages. Each complete
consumer executes in one self-contained chunk with no static or dynamic imports. Five rounds
of all three messages expose 11 distinct replacement functions before the change and one after
it, on both Vite versions. Text, negative zero, and UTC year rollover remain identical. These
are observed function identities, not heap-size or timing measurements.

Another 32 before/after consumers use physical message entries for number-only or date-only
selection. Unminified Vite 7/8 builds exclude the unrelated date or decimal parser, execute
identical text in one self-contained chunk, and retain the callback reuse after tree shaking.

Node 24.13.1 minified/gzip/Brotli bytes:

| Locale | Bundler | Before | After |
| --- | --- | ---: | ---: |
| `ar-EG` | Vite 7.3.3 | 3,780 / 1,652 / 1,486 | 3,692 / 1,637 / 1,478 |
| `fa` | Vite 7.3.3 | 3,886 / 1,718 / 1,519 | 3,798 / 1,708 / 1,509 |
| `ff-Adlm` | Vite 7.3.3 | 4,008 / 1,717 / 1,519 | 3,916 / 1,703 / 1,513 |
| `ccp` | Vite 7.3.3 | 4,099 / 1,777 / 1,567 | 4,009 / 1,758 / 1,561 |
| `ar-EG` | Vite 8.0.13 | 3,649 / 1,553 / 1,403 | 3,561 / 1,556 / 1,397 |
| `fa` | Vite 8.0.13 | 3,751 / 1,655 / 1,460 | 3,663 / 1,649 / 1,456 |
| `ff-Adlm` | Vite 8.0.13 | 3,877 / 1,653 / 1,464 | 3,785 / 1,645 / 1,455 |
| `ccp` | Vite 8.0.13 | 3,964 / 1,672 / 1,478 | 3,874 / 1,665 / 1,478 |

All minified consumers shrink by 88–92 bytes. Compression is not uniformly smaller: Arabic
gzip on Vite 8 grows by three bytes, and Chakma Brotli is unchanged there. The change removes
per-call callback creation without introducing a runtime lookup registry or CLDR parser.

## Runtime link mutation queues

The DOM observer counted text nodes and duplicate records toward its overflow threshold even
when neither added a traversal root. That promoted a batch to a document scan. The queue now
reports whether it accepted a root, and only that work advances the overflow counter.

A fixture with 600 anchors measures follow-up work after its initial scan. Adding 300 text
nodes changes scheduled tasks from one to zero and visited elements from 601 to zero. A batch
of 300 attribute records for one changed link still schedules one task, but visits one element
instead of 601. Both animation and timer modes execute these checks through complete minified
consumers. Package tests also retain the overflow fallback across one/multiple batches, the
256-element frame budget, download/ignore handling, explicit refresh, and cleanup.

The consumers compile the maintained web package, create its routing metadata, and initialize
runtime link localization. Each executes in one chunk without static or dynamic imports.
Node 24.13.1 minified/gzip/Brotli bytes:

| Bundler | Before | After |
| --- | ---: | ---: |
| Vite 7.3.3 | 9,853 / 3,797 / 3,417 | 9,851 / 3,801 / 3,415 |
| Vite 8.0.13 | 9,646 / 3,682 / 3,323 | 9,644 / 3,684 / 3,332 |

Minified output shrinks by two bytes. Gzip grows by four/two bytes respectively; Brotli falls
by two bytes on Vite 7 and grows by nine on Vite 8. These are operation counts and complete
consumer sizes, without a timing or heap-size claim. The observer still examines mutation
records to find affected roots; the bounded frame budget applies to element traversal.

The subsequent lifecycle fix also makes `refresh()` and repeated `destroy()` inert after
disposal. A throwing document getter exposes two invalid accesses and exceptions before the
fix, and zero accesses/exceptions after it; the observer disconnects once. Both animation and
timer consumers cancel pending traversal. Relative to the queue optimization above, the
guards add 17 minified bytes and four/three Brotli bytes on Vite 7/8:

| Bundler | Before cleanup fix | After cleanup fix |
| --- | ---: | ---: |
| Vite 7.3.3 | 9,851 / 3,801 / 3,415 | 9,868 / 3,806 / 3,419 |
| Vite 8.0.13 | 9,644 / 3,684 / 3,332 | 9,661 / 3,688 / 3,335 |

These complete consumers retain the queue operation counts above and one self-contained chunk.
The figures establish observable cleanup behavior and code size, without a heap-size claim.

## Decimal-separator visibility

GEN-A70 retains the semantic difference between `0` and `0.` in the shared pattern/number
plan. The ECMAScript template folds this metadata into selected concatenation. For an ASCII
integer plan without padding, the return expression appends the separator directly; padding
and digit-localization plans retain the intermediate string when they reuse it. No runtime
flag, pattern parser, or extra helper is emitted.

Complete Vite 7/8 consumers import one integer formatter from the regenerated ten-export
fixture. Each retains only its selected export, executes the five integer corpus inputs plus
NaN/infinity, and emits one self-contained chunk without imports. Node 24.13.1
minified/gzip/Brotli bytes:

| Bundler | Ordinary integer | Always-visible separator |
| --- | ---: | ---: |
| Vite 7.3.3 | 1,257 / 686 / 625 | 1,261 / 690 / 630 |
| Vite 8.0.13 | 1,232 / 670 / 612 | 1,236 / 673 / 615 |

The corrected output requires `+"."`, adding four minified bytes on either bundler. Gzip grows
by four/three bytes and Brotli by five/three. This is a corrected-output comparison, rather
than a claim that incorrect output was equivalent or that heap use or execution time improved.
The initial template retained an unnecessary intermediate binding; folding it into this return
reduced the Vite 7 cost from fourteen extra minified bytes to four.

All 3,438 existing pinned pattern parts use `false`. Eight native number/currency/date
consumers across `ar-EG`, `fa`, `ff-Adlm`, and `ccp` retain byte-identical Vite 7/8 programs,
including identical gzip/Brotli sizes, text, shared digit callbacks, and one chunk per entry.
Existing application goldens retain their code, declaration, and map bytes. Profile v8 extends
compiler metadata; direct Rust struct literals require the new boolean field, as described in
the migration notes. These results do not complete the remaining numeric-pattern model gaps.

## Currency affix provenance

GEN-A71 keeps literal currency marks separate from substitution slots in compiler metadata.
The common number planner expands only slots, producing final prefix/suffix strings before
ECMAScript rendering. Generated formatters gain no runtime affix arrays, slot dispatch,
pattern parser, or substitution helper. Native previews use the same resolved strings.

A shared ten-case contract exercises parsing, planning, native previews, and generated
positive/negative calls. Twenty results match ICU 78.3 with fraction precision fixed to zero,
isolating affix behavior from currency precision defaults. Cases include literal marks,
literal/slot order, repeated slots, suffixes, explicit negative affixes, and doubled apostrophes.

All 6,876 profile-v9 affixes flatten to the exact previous compiler artifact. Pinned literal
fragments contain no currency marks, so existing substitutions retain their text. Eight complete
native Vite 7/8 number/currency/date consumers remain byte-for-byte identical, including
minified/gzip/Brotli sizes, output, callback sharing, and one self-contained chunk per entry.
All existing application code/declaration/map goldens also retain their bytes.

This is a prerelease Rust source change: prefix/suffix fields now require typed affixes.
The API checker passes its 223 lints per affected crate, but its documented type-change
limitation does not establish source compatibility. A compiler witness rejects legacy string
assignments and accepts their typed replacements. All 71 local full-profile gates pass.
Wider currency display markers remain explicitly unsupported; this correction
does not claim complete currency-pattern support or improved execution time or heap use.

## Currency letter-boundary alternatives

GEN-A72 selects full pinned `alphaNextToNumber` patterns in the shared Rust currency planner.
Native previews and target rendering use the same style, symbol, boundary, separator, precision,
and rounding choices. Unicode 16.0 classification stays in the compiler. The ECMAScript numeric
template is unchanged: it receives the selected constants, without runtime classification,
option objects, alternative-pattern tables, or CLDR parsing.

A shared 32-pattern corpus retains the selected raw patterns and executes 64 positive/negative
results against both native previews and generated formatters. Independent ICU 78.3 formatting
uses those explicit pinned patterns and resolved symbols/precision. This verifies the numeric
rendering without treating the host's legacy currency-spacing selection as the oracle for
CLDR's full alternatives. The corpus includes alternatives that move the symbol, change grouping,
or replace an explicit accounting negative pattern with implicit negation.

Complete native consumers render ordinary numbers, USD, CHF, and dates in eight locales. Each
Vite 7/8 entry emits one chunk with no static or dynamic imports. All 16 preserve ordinary
number/date results and digit-callback sharing/counts. Minified/gzip/Brotli bytes:

| Locale | Vite 7 before → after | Vite 8 before → after |
| --- | --- | --- |
| en | 3920/1586/1439 → 3924/1593/1444 | 3761/1528/1377 → 3769/1534/1382 |
| agq | 3932/1628/1470 → 3940/1610/1452 | 3779/1544/1401 → 3795/1550/1398 |
| ccp | 4383/1792/1589 → 4363/1798/1607 | 4224/1696/1502 → 4212/1734/1533 |
| bn | 4205/1789/1574 → 4209/1793/1577 | 4046/1692/1488 → 4054/1696/1492 |
| fa | 4171/1746/1547 → 4175/1751/1554 | 4012/1685/1492 → 4020/1690/1496 |
| ar-EG | 4064/1669/1506 → 4064/1669/1506 | 3911/1582/1425 → 3911/1582/1425 |
| de-CH | 3891/1546/1402 → 3891/1546/1402 | 3738/1486/1340 → 3738/1486/1340 |
| km | 4062/1701/1500 → 4064/1705/1504 | 3903/1617/1422 → 3907/1621/1426 |

The four unaffected ar-EG/de-CH programs retain exact bytes. Other consumers have corrected
currency output; their size changes include required spacing and pattern placement/grouping.
These are complete-consumer costs, without timing or heap-use claims. All previous metadata
retains exact bytes after removing the new alternative fields. The Rust SDK's two added fields
are a documented source break; the common planner's new currency entry point is additive.
All 71 local full-profile gates, Rust 1.87 native/WASM, strict JS 5.8/5.9, and offline
reproducibility checks pass. All 45 rebuilt production locale payloads retain exact paths and bytes.

## Optional integer zeros

GEN-A73 compiles zero integer omission from the shared numeric plan. For selected fixed-fraction
patterns such as `#.0`, the formatter changes the rounded integer `"0"` to an empty string before
formatting the body. Carry already happened, so `0.96` still becomes `1.0`. Prefix/suffix text,
signed zero, grouping, padding, and digit substitution retain their owners. Optional-fraction,
significant, and scientific plans keep their existing integer rules. Generated code receives no
runtime option or pattern parser.

The shared 15-pattern corpus has 180 ICU 78.3 results using the formatter's existing half-up
rounding contract. Native fraction previews execute 156 results and explicitly reject the other
24 percent/padding cases. Generated code executes all results with string/number inputs, plus
bigint equivalence, a thousand-digit bigint, coercion count/error identity, signed zero, and
nonfinite fallback. The curated program also passes strict JS on TS 5.8/5.9/6.

All 5,265 current pinned pattern parts require an integer digit. Sixteen complete native Vite 7/8
number/USD/CHF/date consumers retain byte-identical programs, compressed sizes, results, and
callback sharing. A selected consumer imports just one generated style:

| Style | Vite 7 minified/gzip/Brotli | Vite 8 minified/gzip/Brotli |
| --- | --- | --- |
| Required integer (`0.0`) | 1293/700/634 | 1268/687/624 |
| Optional integer (`#.0`) | 1312/709/639 | 1287/695/634 |

The optional style adds a 19-byte minified comparison in both builds. Each consumer emits one
chunk, one `render` export, and no static/dynamic imports; other styles, affixes, and Unicode digit
tables are pruned. Both execute the selected behavior and observe input coercion once. These
measurements establish payload costs and pruning, without an execution-time or heap-use claim.
Rust 1.87 native/WASM, workspace Clippy, and 223 patch-level common-codegen API checks pass.

## Compiler lookup stack use

GEN-A74 addresses the Windows debug CLI stack overflows in [push run 46063349104](https://github.com/antepodeum/Linguini/actions/runs/46063349104).
A local native `build` reproduces the overflow with a 1 MiB main-stack limit. Unoptimized
lookup match arms constructed each large formatting record by value. Profile v11 assigns each
record a scoped static owner; private lookups return references and public APIs copy just the
selected record after locale fallback. Calendar rows retain their shared static symbol groups.

The same Linux debug binary's disassembled functions have these explicit stack reservations:

| Private lookup | Before, bytes | After, bytes |
| --- | ---: | ---: |
| Number | 1,072,600 | 40 |
| Currency | 1,221,064 | 40 |
| Date | 1,713,656 | 40 |
| Calendar | 1,468,888 | 72 |

These measurements describe the lookup functions, without asserting a total process-memory
bound or a cross-platform assembly layout. Complete native `build` and `generate` calls,
including forwarded Date aliases and two skeletons, pass with both 1 MiB and 256 KiB main stacks.
A Rust regression exercises the lookup families on a 256 KiB thread. The debug CLI grows from
289,089,688 to 289,858,528 bytes as records acquire static storage; this is a compiler artifact,
not an application-payload measurement.

Removing only the new ownership wrappers and private reference return types reproduces the
previous compiler table byte for byte: 766 number, 766 currency, 765 date, and 3,060 calendar
records retain their data. Public lookup signatures and returned values remain unchanged.
All locked workspace tests pass on Node 20. Strict Clippy, Rust 1.87 native/WASM, offline
reproducibility, and all 223 patch-level CLDR API checks pass. Sixteen complete native Vite 7/8
number/USD/CHF/date consumers retain identical programs, compressed sizes, outputs, and callback
sharing. Hosted Windows confirmation remains a separate integration receipt.


## Numeric padding widths

GEN-A76 replaces full-result string concatenation and character-array construction used only
for padding width. Rust counts resolved affixes and scientific symbols. Templates measure the
numeric body before digit substitution: each mapped digit still occupies one Unicode scalar.
Supplementary separators contribute compiled UTF-16 corrections; grouping counts use the
selected primary/secondary widths inside the existing grouping branch. Optional decimals add
their correction only when displayed. No extra traversal of the numeric output is required.
Patterns whose affixes already exhaust both signed widths omit padding entirely.

The new executed corpus covers 2,496 comparisons across fixed/optional/significant fractions,
integer-only/forced-decimal output, scientific signs, equal/unequal/delayed grouping, all four
padding positions, zero/short/wide limits, signed zero, rounding carry, and thousand-digit input.
ASCII, Arabic, Adlam, Chakma, mixed BMP/supplementary digits, and supplementary affixes and
multi-character separators retain Unicode scalar widths. Expected padding uses the unpadded
result's independently counted scalars. Existing literal-output padding tests remain intact.
Execution disables string iteration and verifies zero additional character-code reads.

Complete tree-shaken consumers of the generated template output, compared with `09394e6` on
Node 24.13.1, preserve output and emit one `render` export/chunk without imports. Vite 7.3.3
and 8.0.13 produce these minified/gzip/Brotli sizes:

| Vite | Selected plan | Before bytes | After bytes |
| --- | --- | --- | --- |
| 7 | ASCII digits/separators | 1514/802/734 | 1519/803/737 |
| 7 | Arabic digits, supplementary affixes | 1632/895/820 | 1639/896/823 |
| 7 | Adlam digits, BMP separators | 1599/847/784 | 1606/849/782 |
| 7 | Chakma digits, supplementary separators | 1624/872/810 | 1671/890/829 |
| 7 | ASCII digits, supplementary separators | 1550/833/768 | 1595/856/788 |
| 7 | Equal scalar-width affixes | 1526/817/754 | 1517/814/748 |
| 8 | ASCII digits/separators | 1467/781/712 | 1465/774/707 |
| 8 | Arabic digits, supplementary affixes | 1585/873/794 | 1592/871/795 |
| 8 | Adlam digits, BMP separators | 1552/827/754 | 1552/820/751 |
| 8 | Chakma digits, supplementary separators | 1577/851/782 | 1613/864/799 |
| 8 | ASCII digits, supplementary separators | 1503/813/741 | 1544/833/761 |
| 8 | Equal scalar-width affixes | 1479/797/731 | 1470/792/724 |

All twelve consumers create zero string iterators for padding instead of one. This establishes
removal of that traversal and character array, without an execution-time or retained-heap claim.
Supplementary separators need selected arithmetic and increase some payloads; the largest
observed change is 47 minified and 20 Brotli bytes. None of the current pinned numeric patterns
requests padding, so this arithmetic is absent from their generated implementations. Existing
application code, declarations, and maps remain unchanged; curated padding fixtures exercise
the selected implementations.

## Compiler source-map traversal

GEN-A77 removes growing-prefix rescans from module framing and original-position resolution.
New statements contribute their newline count once. Requested source offsets are sorted and
resolved through disjoint byte intervals, retaining the original generated-map order and UTF-16
column semantics. This includes multiple mappings within a single long source line.

The encoder uses O(M log M) sorting and O(M) temporary storage for M mappings. Text traversal
is proportional to emitted statement text and the used source prefixes. This exchanges compiler
scratch storage for removal of repeated text work; it makes no execution-time or retained-heap
claim. Complete-project JavaScript, declarations, and source maps retain their existing golden
bytes. Focused Unicode/map/provenance tests and strict crate Clippy pass. Application runtime
code and payload costs are unchanged.

## Native fractional plural inputs

GEN-A79 adds a conservative shared-plan proof that every category requires `v = 0`.
For classifiers already using the native integer-suffix path, finite fractional numbers now
return `other` before string conversion or regex parsing. Safe integer operations are unchanged.
Permissive, negated, and modular guards retain their ordinary evaluation; string visible digits,
compact exponents, BigInts, object coercion, and invalid-input errors preserve their contracts.
Executed Russian/Ukrainian cases disable String conversion and regex matching on proven paths,
including subnormals and the native fractional precision boundary.

A complete Vite 7.3.3 consumer compared with `cf65806`, built and executed on Node 24.13.1,
retains one `render` export in one import-free chunk and identical representative results.
Minified/gzip/Brotli bytes change from 925/537/485 to 963/545/492. The added selected branch
trades 38 minified and 7 Brotli bytes for avoiding fractional-input text work. This establishes
operation removal, without an execution-time or retained-heap claim.

## Declaration provenance lookup

GEN-A80 completes a separate source-map rendering follow-up: shared type declarations and
message namespace types no longer recount the complete preceding output or search all schema
origins for each member. A borrowed-name index preserves declaration-kind separation and the
last matching origin. Each scope tracks its current line and maps newly appended fragments.
Nested group documentation/opening lines and descendant/sibling offsets preserve their owners.

For S distinct origins, the ordered index takes O(S) compiler scratch storage and O(S log S)
construction; individual lookups take O(log S). Text counting no longer revisits growing prefixes
within a scope. Nested type rendering still traverses the rendered child text at its ancestors;
this is not a claim that every compiler traversal is eliminated. Generated code, declarations,
and source-map bytes retain their existing snapshots. No runtime bytes or operations change,
and no compiler timing or retained-heap measurement is claimed.

## Scientific normalization records

GEN-A81 replaces nested scientific normalization results with a flat mantissa/exponent record.
Each zero/nonzero branch constructs one record rather than a decimal plus a result wrapper.
The caller retains its original sign source and uses the normalized mantissa for both rounding
and unchanged-record fraction reuse. Frozen canonical input checks retain immutability; the
existing precision/carry, localized digits/symbols, padding, and expansion-bound corpora pass.
All 50 formatter tests and the registered strict checked-JavaScript runtime fixture pass.

Complete consumers compared with `496b424`, built and executed on Node 24.13.1, each retain
one `render` export/chunk, no imports, and identical results for zero, negative zero, carries,
positive/negative and extreme exponents, numbers, strings, and BigInts. Sizes are minified/gzip/
Brotli bytes:

| Vite | Selected digits | Before | After |
| --- | --- | --- | --- |
| 7.3.3 | Latin | 1856/903/835 | 1843/889/824 |
| 7.3.3 | Persian | 1975/996/922 | 1962/982/907 |
| 7.3.3 | Adlam | 1961/963/901 | 1948/948/882 |
| 7.3.3 | Mixed BMP/supplementary | 2126/1015/942 | 2113/1001/929 |
| 8.0.13 | Latin | 1831/883/815 | 1810/869/801 |
| 8.0.13 | Persian | 1950/978/902 | 1929/961/884 |
| 8.0.13 | Adlam | 1936/944/875 | 1915/929/862 |
| 8.0.13 | Mixed BMP/supplementary | 2101/994/912 | 2080/980/905 |

This removes one object construction in the normalization source. It is not a measured
retained-heap or execution-time claim. Non-scientific implementations emit no new code.
GEN-A82 below supplies scientific integer-width metadata and fixed/engineering lowering.

## Scientific width and precision specialization

GEN-A82 retains scientific-only integer widths in compiler metadata and resolves fixed widths,
engineering exponent steps, and fraction/significant/unlimited precision in shared Rust plans.
Normalizers are emitted once per selected layout. The checked fixture shares its engineering
normalizer across finite, unlimited, and quantum requests and emits no default one-digit
normalizer. Unlimited precision emits no rounding call or record alias, and minimum exponent
width one emits no padding call. Unproven quantum carries alone select the numeric shift helper;
its arguments describe actual input shifting rather than CLDR pattern interpretation.

The carry proof now checks whether the carried mantissa is a multiple of the quantum.
When that proof fails, exponent adjustment rerounds the original mantissa. The independent
oracle uses ICU 78.3 `DecimalFormat`, exact `StringPiece` inputs, and half-up rounding. Its 800
results across 32 patterns execute in both generated functions and native numeric previews.
Existing application code/declaration/map goldens retain their bytes. The additional metadata
belongs to the offline compiler input and is absent from application JavaScript.

Complete consumers built and executed on Node 24.13.1 each emit one `render` export/chunk,
no imports, and the selected oracle results. Sizes are minified/gzip/Brotli bytes:

| Vite | Selected pattern | Bytes |
| --- | --- | --- |
| 7.3.3 | `000.00E0` | 1846/887/822 |
| 7.3.3 | `##0.00E0` | 2357/1047/987 |
| 7.3.3 | `###E0` | 1524/787/719 |
| 7.3.3 | `##0.7E0` | 2092/971/904 |
| 8.0.13 | `000.00E0` | 1813/868/800 |
| 8.0.13 | `##0.00E0` | 2308/1030/956 |
| 8.0.13 | `###E0` | 1499/766/700 |
| 8.0.13 | `##0.7E0` | 2057/954/879 |

These are absolute costs for corrected selected implementations. No execution-time or
retained-heap improvement is claimed. GEN-A83 brings native previews onto these shared plans,
including scaling, significant precision, and padding; it adds no application runtime code.

## Normalized significant precision

GEN-A84 collects the input shapes of every caller of each significant rounding/padding helper.
Scientific normalization already establishes the first significant position. A helper used only
by these mantissas therefore omits concatenating and searching the digits again. Its compiled
integer-width bound also removes unreachable integer-rounding branches. Input-derived fraction
limits remain checked when any ordinary caller needs the same dynamic rounding helper.

When the only significant rounding precision has a one-digit normalized mantissa, the canonical
fraction-rounding template emits its literal precision directly, without a dynamic helper.
Padded result records, fresh zero records, and frozen input behavior are preserved. Ordinary
unit-rounding helpers continue reusing unchanged canonical records. The proof is conservative
for mixed callers and independent of their order; no runtime shape flag or CLDR data is added.

The 800 independent ICU expectations execute both with all requests sharing helpers and with
each request emitted alone. Additional mixed-precision checks retain ordinary tiny-fraction
rounding, zero/significant padding, immutable inputs, and the 8,192-digit expansion error.
Operation spies observe no first-digit search during normalized rounding/padding. This does
not remove input parsing or the first significant search needed to normalize tiny input values.

Complete consumers compared with `9f9d204` on Node 24.13.1 retain one `render` export/chunk,
no imports, and identical results. Sizes are minified/gzip/Brotli bytes:

| Vite | Selected pattern | Before | After |
| --- | --- | --- | --- |
| 7.3.3 | `##0.00E0` | 2357/1047/987 | 2111/978/916 |
| 8.0.13 | `##0.00E0` | 2308/1030/956 | 2068/958/886 |

Fixed `000.00E0`, unlimited `###E0`, quantum `##0.7E0`, the ordinary significant and scientific
consumers of `decimal-separator.js`, and the Latin scientific consumer of `scientific.js` retain
their previous bundle bytes on both Vite versions. A new explicit three-significant-digit
scientific consumer costs 1979/924/863 and 1938/904/838 bytes respectively; it has no prior
checked-fixture counterpart, so these are absolute sizes. No execution-time or retained-heap
improvement is claimed. Multiple significant precisions retain their shared dynamic helper
rather than duplicating a literal rounding body for every precision.

## Required significant fraction digits

GEN-A85 removes trim/scan/pad work when a scientific request has equal significant minimum and
maximum precision. One-digit mantissas retain the padded rounding fraction; engineering results
keep only the fraction digits left by their rounded integer width. This accounts for carries
within an exponent group: `10.00` at three significant digits must become `10.0`. Zero uses its
compiled zero suffix. Helpers are selected only for callers still needing significant padding.
When precision exceeds the maximum mantissa integer width, the fraction is proven nonempty
and concatenates directly. Ordinary and optional significant requests retain their existing path.

The same 800 ICU results pass with shared and isolated helper sets, including wide integer-only
rounding and engineering carries. Mixed caller ordering, minimums below maximum precision,
and normalized-padding operation checks remain covered. Executed complete consumers compare
with `f717a83` on Node 24.13.1; each has one `render` export/chunk and no imports. Sizes are
minified/gzip/Brotli bytes:

| Vite | Consumer | Before | After |
| --- | --- | --- | --- |
| 7.3.3 | Engineering three significant digits | 2111/978/916 | 2025/944/880 |
| 8.0.13 | Engineering three significant digits | 2068/958/886 | 1990/926/859 |
| 7.3.3 | Fixed three significant digits | 1979/924/863 | 1861/880/815 |
| 8.0.13 | Fixed three significant digits | 1938/904/838 | 1828/865/796 |

The other six consumers from GEN-A84 retain their previous bytes on both Vite versions.
These changes remove the selected source operations and helper; no execution-time or
retained-heap improvement is claimed.

This audit supplements the [fixture review](./generated-code-review.md). It does not close the
remaining optimization, payload-delivery, or calendar/time-zone/skeleton requirements.
