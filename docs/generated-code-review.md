# Generated-code fixture review

This review starts from the fixtures at `d42c133`. The inventory contains 910 files:
195 JavaScript, 515 TypeScript (including declarations and frozen comparison implementations),
155 source maps, 21 JSON, two Rust files, and 22 other input/expectation files. Executable
JavaScript/TypeScript/Rust contents form 244 distinct groups after identical copies are grouped.
The generated JavaScript compiler scan covers 171 roots, excluding consumers and the oldest
`snapshots/js` comparison implementation. Findings below are traced to their current generators;
frozen comparison code is evidence of earlier behavior, not a production implementation to edit.

## Acceptance

Resolve CLDR patterns, symbols, conditions, precision, and other static choices in Rust during
generation. Runtime parsing of actual user numeric/date values remains necessary. Runtime
parsing or revalidation of already compiled CLDR syntax and choices does not satisfy this
contract. Preserve exact arithmetic, negative zero, visible decimal digits, callable forms,
argument validation, source maps, SSR, and typed consumers while specializing code.

For each fix, inspect freshly regenerated output and execute representative cases. A smaller
snapshot alone does not establish correctness or faster execution. Page delivery still needs
the composed-page graph and request checks specified in ESM-A3/MOD-A4.

## Findings

| ID | Status | Evidence and generator responsibility | Required change and verification |
| --- | --- | --- | --- |
| GEN-A1 | Fixed in `d42c133` | `codegen-js-plural-ru.js`; `src/plural.rs`, plural templates, shared plural plans | Direct predicates, common guards, scalar operands, shared remainders; no generic relation calls or range/operand objects. Executed 24-locale/operator corpus and a Russian fraction test with bigint construction disabled. Decimal-modulus-only integer relations now select a bounded suffix through shared requirement analysis: exact number arithmetic through 15 digits, bounded bigint beyond that, and no redundant largest-modulus operation. Widths 1–19, mixed moduli around the safe-integer boundary, and 8,192-digit/exponent inputs execute with bounded-construction assertions; full-value and non-decimal-modulus relations retain their full integer path. |
| GEN-A2 | Fixed | `codegen-js-formatters-ru.js`: `roundGeneratedDecimal(decimal, 3, "1")`; `number_helpers.jinja`, `formatters/number.jinja` | Fraction precision and quantum are known at build time, but each call validates/parses them, computes a remainder modulo one, and may construct a large power of ten for discarded digits. Emit specialized rounding from the validated request. Verify ties, carries, currency increments, scientific/significant rounding, huge exact values, and negative zero. |
| GEN-A3 | Fixed | Same currency fixture: `trimOptionalFractionDigits(rounded.fraction, 2)` with exactly two generated digits | Omit trimming when minimum and maximum fraction lengths coincide. Select the helper only when another actual request needs optional digits. Preserve significant-digit padding. |
| GEN-A4 | Fixed | Same date fixture: unused `padNumber`; `date_helpers.jinja`, `formatter_codegen.rs` | Derive padding/digit helper demand from the compiled date fields, including mixed requests. The medium Russian date needs no padding helper; short numeric dates still do. |
| GEN-A5 | Fixed | Numeric plans with scale; `number_helpers.jinja`: `scale === 1 ? ... : scale === 100 ? ...` | Fold the validated scale into its decimal shift during generation; omit runtime selection among unused scales. Preserve expansion limits and exact input semantics. |
| GEN-A6 | Fixed | `js-baseline/locales/ru/email_input.js` imported every global despite containing only static messages; root `ru.js` imported unused `DeliveryNote` | Namespace/barrel imports now use shared direct semantic edges. Static modules omit global imports; transitive globals stay in their owning modules. Executed project and physical bundler graphs cover function calls, captured variables, alias-typed callable forms, simultaneous inline-input scope, named/positional calls, and sparse regional fallback. Exact composed-page delivery remains GEN-A10/MOD-A4. |
| GEN-A7 | Replaced by runtime packages | Frozen `js-svelte-control` and `js-svelte-effects` fixtures record earlier generated scaffolding | Generic state, controls, and effects are no longer generated. Maintained TypeScript npm runtimes own this behavior; real Svelte server/client compilation verifies request isolation, loader snapshots/disposal, and prepare-before-publish. Runtime control tests and the real site cover persistence/navigation. Initial bundle demand remains GEN-A9/MOD-A4. |
| GEN-A8 | Replaced by runtime package | Frozen `js-web-server` switch handlers searched a static marker per request | The reusable TypeScript server adapter compiles the route prefix/suffix once when the application creates its handler. Web parity and hook tests cover encoded locales, safe/unsafe return targets, statuses, cookies, and streaming. No native route emitter remains. |
| GEN-A9 | Open | The retired generated web facade exposed unselected capabilities; npm adapters now own those implementations | Keep resolver implementations as explicit imports and metadata-only hooks independent of message catalogs. Native policy generation/transport has been removed. Verify actual tree-shaken client/server graphs and remove unused runtime operations from initial page delivery; package separation alone does not establish exact delivery. |
| GEN-A10 | Partial | `js-complete-project/kit/javascript/index.js` eagerly imports base-locale messages; `locale.js` contains a locale-resolution table; shared runtime exports general dispatch/argument helpers | Vite folds sparse fallback per message and compiles the surviving composed-page closure into one self-contained locale payload. Real Kit output verifies three pages across nine locales, with a layout, child component, and shared function whose other export belongs only to another page. Chromium verifies the selected preload before framework execution, matching SSR/hydration, cross-page/cross-locale preparation, cached revisits, and no message requests without JavaScript. Lazy actions now prepare only their additional selected-locale closure before module evaluation, with cached cross-page reuse. Unsupported import shapes, remaining configuration coverage, and selected resolution/helper operations stay open. Lookup tables of resolved aliases are not a CLDR-pattern parser, but their initial delivery must still be justified. |
| GEN-A11 | Partial | `js-baseline/locales/ru.js`: delivery repeats plural normalization through `Delivered`, `SizeAdj`, and the form, and wraps known string results in `String`; globals have dense nested form formatting | Generated number/currency/date results now bypass redundant outer `String` calls. Alias/default/explicit formatter chains retain their string result fact; skipped unknown formatters cannot establish it. Executed tests preserve named/positional calls, raw input coercion order, date reads, special numeric values, and thrown errors. Validated named/inline function calls, callable form branches, and plural intrinsic results also bypass outer coercion through a shared semantic result fact. Generated execution forbids conversions of those results while retaining raw-input coercion, eager captures, and exceptions. Form objects and nested dispatch bodies now use consistent multiline indentation. Shared scope-aware facts now remove wrappers around simple generated locale text variable references, including nonliteral aliases and imported globals. Executed project and physical graphs cover variable initializers, functions, forms, namespaces, raw-input coercion hints, and errors. Local bindings mask global facts. Repeated pure plural calculations and further proven reference shapes remain open; preserve evaluation order and source maps. |
| GEN-A12 | Fixed | `web/accept-language.js` filtered/sorted preferences separately for each locale; runtime-link helpers check then reparse the same href; request context computes direction repeatedly | Accept-Language now selects preferences directly, normalizes ranges/locales once, and keeps scalar winner state without per-locale candidate objects. Explicit expectations and 512 frozen-implementation comparisons preserve exclusions, quality, specificity, input/base/locale order, navigator fallback, and getter behavior. The TypeScript web runtime now combines link attribute checks and localization around one parsed URL/environment snapshot. Ordered exclusion predicates share an isolated URL, preserving mutation order and thrown errors. Request contexts resolve direction once and reuse it across their metadata fields, with no cross-request cache. Executed constructor/getter counts, 600-anchor scheduling/cleanup parity, URL-policy comparisons, and packed strict consumers verify the boundary. These parse runtime inputs, not CLDR patterns. |
| GEN-A13 | Partial | `formatter_codegen.rs::date_parts` substituted English AM/PM/era labels, conflated format/standalone context and name widths, and uses a default UTC expression for unhandled fields | Shared `date_field_symbols` now compiles selected Gregorian context/width-specific name tables, AM/PM values, and era labels; identical tables within a formatter share a binding. Executed Russian context/era, English width/deduplication, Arabic/Chinese period, and Arabic/Hindi real-request cases correct the old expectations. Fixed/flexible period rules now compile into scalar conditions and selected literals; an independent raw-rule oracle executes 14 locales × two period fields × three widths × every minute × four subminute offsets, with display-precision and missing-label/merging cases. Gregorian year-of-era conversion and minimum-width padding now preserve BCE/CE boundaries, including year zero; 18 year samples × eight widths execute against explicit expected values, and the checked date fixture covers the boundary. Named formatting `E`/`e` and standalone `c` weekdays now use their correct context/width tables; executed Finnish/Inari Sami full requests correct a real `UTC` placeholder bug, with explicit formatting/standalone and narrow/short expectations. Shared date-pattern compilation now parses quoting, rejects unsupported fields/widths with byte offsets, and folds UTC names/ISO-zero forms into merged literals. Executed full/long requests in eight locales cover localized names, missing-name GMT fallback, and Chinese zone-before-time order with host formatting disabled. A broad style/composition scan covers the 508 distinct Gregorian data records reachable through the pinned supplemental resolution graph. Extended calendars, selectable non-UTC zones, non-exact skeleton matching, and additional selected fields remain open. No runtime CLDR interpreter is allowed. |
| GEN-A14 | Fixed | `normalizeMessageArgs` copies rest arguments on the positional path; callers allocate a static keys array each call; numeric grouping allocated/reversed a group array | Grouping now compiles equal/unequal primary/secondary widths into direct string loops, omitting arrays/reversal/join. Executed 32-plan width/padding coverage includes Unicode separators, negative zero, and 8,192-digit values alongside the formatter corpus. Local Node 24 timing checks covered 5–8,192 digits. Static argument-key arrays are initialized once per message module; positional normalization returns its input without copying. Executed generated calls verify own-property and unknown-key checks, symbols/non-enumerable keys, null-prototype objects, inherited properties, frozen inputs, getter/Proxy observation order, and validation before property reads. |
| GEN-A21 | Fixed metadata invalidation | Maintained `packages/sveltekit/src/server.ts::createLoad` only read `locals`; real SvelteKit data responses declared no URL dependency, permitting stale shared-layout locale metadata on navigation. | The helper now reads the tracked original URL pathname. A built-server regression verifies French and Russian data responses declare `uses.url`; packed strict TypeScript consumers cover optional URL forwarding and reject pathname strings. Hooks stay metadata-only, locals-only extraction remains compatible, and the eight site gates plus executable documentation pass. Exact page-payload preparation remains GEN-A10/MOD-A4. |
| GEN-A22 | Fixed native base-locale linking | The Vite default required a nonexistent locale helper, and native `lgl` imports lacked tracked import identities. | The analyzer and CLI share the built-in namespace vocabulary. The default Vite adapter links base-locale leaves directly; virtual re-exports contribute zero rendered bytes. Real eager/dynamic builds execute values, functions, and bounded computed access, excluding inactive French messages and locale state/loaders. Cold/warm native manifests retain exact direct/aliased bindings; a native compiler-to-Vite build executes in one chunk. Stateful adapters retain their separate locale behavior; composed-page delivery remains open. |
| GEN-A23 | Fixed setter catalog retention | A setter-only import through the generated Svelte facade retained the base catalog, eight lazy locale catalogs, and a shared helper in the nine-locale site probe. | The locale helper owns one catalog-independent setter; the facade re-exports it directly and injects the same function into its provider. The Svelte Vite adapter declares unused entry initialization discardable. The production graph gate builds actual native output for server and client with the SDK and adapter, asserts one chunk containing only generated locale metadata/state modules, excludes dynamic catalog imports, and executes normalized/default locale changes. Runtime tests preserve function identity and stale-change guards. These isolated builds are not general application size benchmarks. |
| GEN-A24 | Fixed | `js-date/skeletons.js` emitted identical abbreviated month arrays separately for `GyMMMd` and `yMMMd`. | Locale-runtime date requests now intern complete equal name vectors. Different widths and Russian formatting/standalone names remain distinct. Executed month/weekday getter counts preserve observations. Real Vite consumers retain the shared table without the unused era formatter, and time-only output drops all month tables. The measured two-formatter bundle shrinks from 1,598 to 1,502 minified bytes and from 651 to 634 Brotli bytes. |
| GEN-A25 | Fixed native validation mismatch | Existing codegen executes `fn Echo(value: String) { _ => {value} }`, but source analysis rejected its body as an extra selector; Date/numeric payloads also received a misleading style warning. | The source analyzer accepts the sole root `_` body when there are no dispatch parameters, including zero-argument functions. The existing style lint now checks named payload presence, regardless of type. Native check/cold/warm builds and executed positional/named, nested/captured, Boolean/number/date calls pass; invalid keys, nested dispatch, and duplicate bodies retain errors and prior output. Workspace LSP diagnostics agree. Native Vue/React parent-child SSR builds use the String helper, while all nine site gates cover a Date helper through sparse-locale fallback and exact payload delivery. |
| GEN-A26 | Fixed | `packages/web/src/locale.ts` spread the full runtime into the metadata-only web object, enumerating arbitrary properties and forwarding message APIs. | Copy only declared locale fields. Throwing catalog/symbol getters and an enumeration-trapping Proxy verify construction and request metadata. Kit hooks and an installed strict TypeScript consumer retain metadata without catalog access; an executed SSR bundle shakes catalog barrel exports and unselected resolver modules. Broader platform-operation and page-demand acceptance remains GEN-A9/MOD-A4. |
| GEN-A27 | Fixed | Unnamed callable form maps always used `value`, shadowing a referenced locale variable or a custom plural helper with that name. An executed project rendered `1 / Second label` instead of the declared global text. | Share the function selector reference walker and deterministic name allocator. Legacy and physical semantic/message modules execute the correct branch text through positional/named calls, including suffix collisions with `value2`; a custom classifier named `value` also executes correctly. Non-colliding bindings and explicit source names stay unchanged. |
| GEN-A28 | Fixed | The date digit helper used `Array.from(digits)`, checked its length, and allocated a capturing replacement callback for every numeric field, although the alphabet is fixed CLDR data. | Shared Rust planning validates Unicode scalars before emission. BMP symbols use literal string indexing; consecutive supplementary symbols use code-point arithmetic; other mappings use direct branches. Date fields reuse one compiled callback and omit per-field alphabets and runtime system validation. Shared analysis selects one whole-result substitution when literals and all possible text labels contain no ASCII digits; literal digits and `Q1`-style names retain field-only substitution. Executed tests forbid symbol parsing, retain Date/string/timestamp output and getter counts, and cover supplementary digits. A three-message Arabic consumer creates zero symbol arrays and uses one digit callback, replacing eleven arrays and eleven callbacks, and performs three substitutions instead of eleven. ASCII output omits this support. |
| GEN-A29 | Fixed | A native `ff-Adlm` project rendered `123` with an isolated surrogate because numeric digit replacement indexed the fixed supplementary alphabet by UTF-16 unit. | Number and date generation share a scalar-aware lookup template. Shared Rust planning proves consecutive Unicode ranges and emits direct code-point arithmetic; mixed-width/nonconsecutive mappings compile to branches. All ten digits execute for number, bigint, and string inputs with number/currency formatting; native cold/warm builds retain the result. Mixed/BMP/supplementary alphabets and fractional/negative output execute, and a curated Adlam runtime passes strict JSDoc consumer checks. Existing BMP snapshots remain byte-identical. |
| GEN-A30 | Fixed | A native numeric-only `ccp` project failed because project emission unconditionally requested missing plural rules despite having no plural operations. | Validate plural data from lowered IR demand during project construction; locale emission options can then retain absent unused rule data. Ordinary, standalone, and physical semantic/message modules execute Chakma numeric output without a plural helper. Intrinsic, inline, and named plural calls retain early `MissingPluralRules` errors. Native check/sample generation/cold/warm builds pass, and required-data errors preserve valid output. |
| GEN-A31 | Fixed for proven safe integer inputs | Russian integer counts still converted to strings and matched regexes before computing a two-digit suffix. | Shared planning proves zero fractional/compact guards and integer-only decimal-suffix demand. Eligible safe native integers calculate the absolute-value remainder directly; category decisions remain shared with parsed inputs. Executed Russian/Ukrainian cases cover negative counts, negative zero, safe-integer boundaries, coercion hints/errors, category passthrough, huge bigint/string values, fractions, exponents, and the 8,192-digit limit. A conversion/regex spy verifies zero parsing for safe integers. The complete page consumer adds 22 Brotli bytes; the cost audit records the local runtime tradeoff. Selector-specific category outcomes and repeated normalization remain open. |
| GEN-A32 | Fixed for routing-only web consumers and browser server-sink retention | `createWebLocaleI18n` attached cookie serialization and server writes to every routing object, retaining both implementations in a path-only bundle and server sinks in browser controls. | The metadata object owns normalized policy; explicit serialization/server persistence helpers own their operations. Client and Kit adapters reuse them. Executed Vite 8 graphs exclude persistence/transports from routing-only output and server cookie sinks from browser controls, preserving browser preparation/publication/write/navigation order. Complete Vite 7 consumers save 250 and 107 Brotli bytes respectively. Runtime and installed strict TypeScript tests preserve cookie flags, sink precedence, appended headers, finite locale types, and framework hooks. Generic framework-control pruning remains WEB-A13. |

Named generated imports with unresolved uses now fail explicitly rather than retaining
their catalog barrel. Native token analysis resolves a static property/method tail to its
known message prefix, including quoted members and Unicode comments. Cold/warm manifests
and executed stateful/stateless Vite builds cover string operations and callable `.call(...)`.
Namespace imports now also carry exact import identities and spans through static message roots.
Executed stateful/stateless Vite builds cover named and namespace values, callable leaves, method
tails, and finite computed access. Separate metadata/control exports survive without retaining the
catalog; escaped namespaces and provider/catalog exports fail closed. Cold/warm native manifests
retain the same selected leaves. Default/mixed declarations and unsupported deferred shapes remain open.
The real site's nested lazy module uses a namespace import; exact child records, browser request
counts, locale changes, concurrent server loads, and hydration pass through the same production gates.
The Svelte adapter also shares tag boundaries across message, link, and cleanup insertion. A
regression fixture preserves quoted generic constraints and ignores fake script tags in HTML/CSS
comments; the resulting source compiles with both client and server Svelte targets. Module words
inside quoted attributes cannot redirect instance cleanup into a module or extra script.

Deferred demand now has a separate planning boundary in `packages/vite/src/page-demand.js`.
It reads consumed exports from the emitted caller and re-shakes each lazy entry independently,
so a shared eager/lazy helper does not merge unrelated message demands. Shared contract cases
cover nested imports, required module effects, pure re-exports, default exports, an escaping
namespace, dead lazy exports, and an ES2018 target. These analysis-only async facades never
enter the application output. The Kit adapter compiles and prepares additional closures before
module evaluation. Executed late-linker tests preserve live namespaces; Chromium covers lazy
requests, activated locale changes, cancellation, and reuse across two pages. Concurrent server
loads execute the same lazy values and functions through request-local locale lookup. Inlined
imports, callers shared with unrelated pages, and differing additional demands from the same
lazy module on one page fail explicitly and remain open. The Kit adapter now selects its root
loader after demand planning: a controlled Vite 8 build without additional closures emits the
same bytes as the direct base loader, with no deferred callback branches. Both implementations
share payload loading; enhanced navigation retains metadata-read counts and request isolation.
Broader platform/configuration
coverage remains open.

Nested deferred closures now subtract only messages guaranteed by every parent path. Shared
entries intersect availability, direct paths retain their required records, cycles remain
anchored to initial preparation, and transit-only modules emit no empty files. The generic
compiler tests execute exact records for nested and direct paths. The Kit linker follows only
planned lazy targets when identifying owning pages; real Vite 7/8 output tests cover both kinds
of parent and exclude traversal through a shared framework route registry. The site verifies
two additional closures across nine locales: the child contains only its new message, while
parent values and named formatter calls remain usable. Chromium checks first-action requests,
cached repetition, locale changes, nested concurrent SSR, and hydration without deferred files.

Canonical number/date fixtures already contain build-selected symbols and direct date getter
expressions rather than raw CLDR patterns. Rounding/scale helper specialization and date-helper
selection have since been implemented and tested. Date-semantic substitutions and the other
open findings still need work. Historical TypeScript and oldest JavaScript comparison fixtures retain `Intl`, generic formatter forwarding, branch maps, and the retired
plural matcher. Their existence must not be mistaken for current generator acceptance.

Exact Gregorian `availableFormats` IDs can now be selected through `@date(skeleton = "yMMMd")`
and schema defaults. Rust resolves and validates the selected pattern before emission. Shared
tests check every pinned skeleton entry across 508 resolved locale records; generated requests
execute in English/Russian/French/Chinese with host date formatting disabled, including BCE/CE.
The real nine-locale site keeps the skeleton message in its owning page's one payload and omits
time-field operations; concurrent SSR, hydration, no-JS rendering, and locale navigation pass.
Quarter skeletons `yQQQ`/`yQQQQ` now compile `Q`/`q` arithmetic or selected four-name vectors.
Both contexts and all widths are checked against all 508 pinned Gregorian locale records;
execution covers every month, UTC offset boundaries, Arabic digits, BCE, and supplied-Date
getter observations. Actual bundles omit unused quarter work. Unknown IDs, style conflicts,
and malformed skeleton IDs preserve existing owned output. Every source-selectable exact ID
across 508 resolved locale records now compiles; unsupported fields such as week-number `w`
still produce structured shared-compiler errors. Plural-suffixed internal IDs such as
`yw-count-one` remain outside the source request grammar.
Week-year `Y` compiles the requested territory's first weekday/minimum-days rules into bounded
January/December arithmetic. The shared plan is checked for all 49 rule combinations across a
Gregorian cycle. Eight-locale execution checks eight signed widths, BCE/CE boundaries, localized
signs/digits, and independent calendar-year/era fields. Only selected requests emit the helper;
impossible boundary branches and unused sign/padding bindings are absent. Native cold/warm
builds execute Scottish Gaelic `yMMM` and preserve both locales' output after an invalid request.
Closest-match/width-adjusted skeletons, combined date/time synthesis, other calendars, and
selectable non-UTC zones remain open under GEN-A13/#158.

## Representation and allocation follow-up

The [cost audit](./generated-code-cost-audit.md) records the measured category-name/numeric
prototype, remaining selector-outcome demand, named-validation arrays, fixed fraction presence,
the discarded ISO date-time validation allocation, scientific carry specialization, and
integer-modulo exhaustiveness (GEN-A15–A20). Numeric category literals
alone showed no useful memory reduction in that experiment; branch-specific compilation and
removing proven unnecessary work remain the acceptance criteria.

## Syntax follow-up

The unquoted-quote failure was reproduced and corrected by selecting the lexer mode at the
start of a value. Parser, formatter-preservation, generated-runtime, and editor-grammar cases
cover literal/unmatched quotes, quoted escapes, interpolation, and multiline whitespace.
The reference documents unquoted, quoted, dedented, and raw modes and their boundaries;
migration notes identify the corrected handling of later quotes.
