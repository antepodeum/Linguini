# Migration notes

This file records required application changes when a public Linguini contract changes. The CI
contract gate requires source changes to public configuration, CLI, generated API, Vite, LSP, or
editor boundaries to update this file, public documentation, and executable tests together.

## Native date type annotations

Generated native date annotations now use `globalThis.Date`, preserving native `Date` arguments
when a source type also exports as `Date`. No application changes are required; public source
type exports and accepted date representations retain their meaning. Checked JavaScript and
declaration consumers now agree with executed ordinary, physical, and standalone output.

## Shared project locale matching

The project locale matcher now lives in `linguini_cldr::ProjectLocalePlan`. The existing
`linguini_codegen::ProjectLocalePlan` keeps its Rust type and method signatures through a thin
facade over the same implementation. Locale precedence and generated output are unchanged.
Source tools can use the neutral CLDR service without depending on code generation.

## Inherited source globals in native projects

Native `check`, `build`, and `generate` accept form, enum, function, and variable references
supplied by declared CLDR parents or the project base. Redeclaring these globals in every locale
is unnecessary. Nearer declarations still replace whole definitions, and missing-translation
warnings plus `--deny-warnings` retain their existing meaning.

Rust loaders can provide ordered fallback ASTs through
`SchemaDatabase::analyze_locale_with_fallbacks`. It checks only the current source; analyze each
fallback source independently too. Qualify source IR against composed relative declarations with
`qualify_locale_module_in_context`, then retain the established guarded merge. Original source
identities remain available for diagnostics, dependency manifests, and source maps. Existing
single-source analysis and qualification APIs remain available.

Native and in-memory editor hosts now derive the same declared fallback chain from project
configuration. Validation includes inherited globals within the current namespace, including
unsaved fallback edits, closing a buffer, and configuration changes. Missing-translation
warnings still describe the current locale. The built Node and browser WASM workers share this
behavior. No extension configuration changes are required. Custom Rust hosts can attach an
ordered chain with `LinguiniDocument::with_locale_fallbacks`; standalone documents infer no
implicit base, and ambiguous same-namespace fallback sources are excluded.

Definitions, references, and rename now follow inherited variable/function and locale enum type
ownership through that chain. References from consumers of the same declaration participate
together; nearer overrides and unrelated locales/namespaces retain their own identities. Existing
single-source APIs remain available, and no project or extension configuration change is required.
Rust tools can reuse `inherited_locale_globals` and `locale_navigation` for borrowed effective
globals and current-source occurrences without combining source roots or relying on unique
numeric source IDs. The LSP refuses a rename when an inherited body would acquire a different
binding in another locale. This uses the existing invalid-rename protocol error and leaves all
files untouched; ordinary unambiguous inherited renames remain available. Navigation for typed
form properties remains separate.

## Forwarding classified plural values

Native build and preview validation now accept an existing `Plural` result when a function or
form expects `Plural`, matching the analyzer. This includes explicit `Plural(count)` arguments,
forwarding through named functions, and captured inline values. Classified values remain distinct
from `Number`/`Decimal`; numeric parameters and the numeric-only `Plural(...)` intrinsic still
reject them. No syntax, runtime representation, or public message signature changes are required.

## Function-call and parameter scope

Regenerate output when using the backend API with unqualified named functions whose names also
appear as message or function parameters. Calls now follow validation's named-function precedence
in dependency selection and emission. Private parameter bindings use readable numeric suffixes
when needed to avoid shadowing; named-object keys and public declaration signatures are unchanged.
Function calls no longer acquire implicit currency/date defaults from a same-named parameter.
Ordinary, standalone, and physical outputs share this resolution. No application changes are required.
Native build/preview qualification now also preserves schema message parameters and same-named
function calls in forwarded, inline, and form scopes. Rust loaders that qualify locale files
separately should use the additive `linguini_ir::qualify_locale_module(locale, schema, namespace)`
with both freshly lowered relative modules, then merge the qualified result. No syntax or
application-call changes are needed.

## Editor identity for named calls

Definitions, references, rename, and semantic highlighting now treat `helper(count)` as a named
function call even inside a function parameter or inline binding named `helper`, matching native
validation and generation. Bare `{helper}` references still belong to the local binding. The
shared semantic walk supplies native and WASM editors; no source or extension changes are required.

## Unnamed form receiver validation

Native `check` and `build` now accept existing unnamed form syntax such as `fruit(count)`.
The analyzer previously looked only for a named function and reported `unknown function`.
It now resolves typed enum receivers, including aliases and named/inline function scopes,
requires a callable implementation for every variant, and checks its one plural argument.
Named functions retain their existing precedence over same-named value bindings.

Tooling that constructs expression metadata can use `FormProperty::callable(span)` to describe
the unnamed plural operation. Invalid calls still fail before generated output changes.
Public message signatures and already accepted generated behavior are unchanged.

## Form attribute demand

Regenerate output to omit unused pure form attributes and their dependencies automatically.
The `tree_shaking` option continues to control message selection. Selection runs before inline
lowering, so excluded messages no longer create unused lifted helpers. The common backend plan
is available through `linguini_codegen::prune_unused_form_attributes`; direct message dependency
closures also contain projected forms and source identities from the retained content.

Public message signatures, property lookup/coercion, callable rows, and eager initializers retain
their behavior. Current form snapshots omit unused fields; historical comparison oracles remain
unchanged. Physical form modules now export independent attribute tables separately, so pages
using different attributes retain their own closures. Callable/eager forms remain whole.

Backend integrators should render semantic edges with `EcmaSemanticImport::ecmascript_bindings()`.
For form artifacts, `binding` denotes the consumer-local namespace; value functions/variables
retain named imports and locale enums remain type-only. No artifact struct fields or manifest
versions change. Ordinary and standalone form lookup shapes remain unchanged. This does not
complete mixed-table initialization or optional request/runtime-overhead remediation.

## Native numeric preview plans

Native decimal/currency previews now use `linguini-codegen::NumberFormatPlan` for literal
affixes, grouping widths/thresholds, and fraction precision/quantum. Pattern-defined increments
are no longer ignored, while supplemental currency rounding retains precedence. Unsupported
internal plan families and invalid metadata report preview errors rather than producing an
approximation. No formatter options or application APIs change, and generated artifacts remain
unchanged.

## Shared date-pattern planning

Code generation and native previews now consume the same resolved date-field/symbol and
shared day-period plan from `linguini-codegen`. Missing required symbols are checked alongside
pattern validity. The new `DatePlanError` retains structured symbol errors without changing
existing `DatePatternError` matches. The ECMAScript backend owns target bindings and rendering; native
previews own input evaluation. Source/configuration syntax and generated code, declarations,
and source maps remain unchanged.

## TypeScript watch delivery

`linguini-tsc` now polls known schema, locale, and configuration files through TypeScript's
adaptive priority queues. This removes reliance on delivery from successfully opened filesystem
event watchers; a macOS integration run timed out waiting for a locale text edit. Directory
events still discover source additions/removals. Watch and solution-watch keep declaration-aware
runtime delivery, last valid output on native errors, and explicit disposal. There are no CLI, configuration, generated
artifact, or application API changes, and no application-runtime polling is introduced.

## Native LSP path identity and CI

Native workspace paths now use the same file-URI representation as editor buffers. Windows
canonical verbatim prefixes no longer prevent unsaved buffers from finding their project context.
No client protocol or configuration changes are required. Host parity tests wait for each
versioned diagnostic event with a bounded timeout, preserving invalid-buffer/recovery assertions
under delayed scheduling. Dependency policy allows path-only private/development dependencies,
so unpublished test helpers stay out of registry package manifests. Registry wildcards and
public production path wildcards remain forbidden.

## Native date-time previews

Native `generate` adds noon and offset-bearing date-time cases after the existing date-only Date
sample. Previews normalize the same explicit ISO date-time strings accepted by generated messages
to UTC instead of rejecting them or assuming midnight. Clock fields, display-precision day
periods, year-of-era, and week-year rollover follow the selected compiled request. No generated
JavaScript, declaration, configuration, or application API changes are required.

## Programmatic IR type names

The ECMAScript SDK now emits valid, narrow typing for accepted IR enums and aliases named
`string`, `undefined`, `unknown`, `object`, and other TypeScript type keywords. Previously these
could emit invalid declarations or resolve parameter types to a TypeScript primitive. New safe
type names use a readable reserved prefix, such as `__lgl_type_string`. Imports, references,
short aliases, checked JavaScript, and declarations agree on the name.

Native source case rules, valid existing type names, value exports, and named argument keys
stay intact. This includes contextual names such as `intrinsic`, `type`, and `abstract`; source
names already beginning with the compiler prefix retain their distinct escaped names. Regenerate
SDK output and update imports of newly supported keyword types to their allocated names.
Primitive `TypeModel::String` continues to render `string`; `TypeModel::Named("string")` represents
a named source type and now renders `__lgl_type_string`.

## Custom public plural-function names

Rust callers of `generate_plural_function` can now use names such as `String`, `Number`,
`MAX_PLURAL_DECIMAL_DIGITS`, and `throwInvalidPluralNumber` without shadowing conversions or
redeclaring classifier helpers. Valid public names are preserved through static export aliases.
Invalid identifiers use the same safe-name mapping as other backend exports; `class` maps to
`__lgl_name_636C617373`, and `a.b` maps to `__lgl_name_a$b`.

Normal classifier output, category strings, passthrough categories, numeric parsing, errors,
and project output retain their existing behavior. Regenerate custom-name helpers to receive
the fix; no project configuration changes are needed.

## Public export names and compiler bindings

Regenerate artifacts to fix accepted message or namespace names such as `formatNumber`,
`normalizeMessageArgs`, `String`, and `lgl`. Previously these could redeclare generated helpers
or shadow JavaScript conversion. Public named exports and default-catalog keys retain their
spelling; local bindings now use static export aliases when required. Declarations preserve
positional/named overloads and source enum types, and do not export the private binding names.
Private functions and variables likewise leave standalone helper names available. No source,
configuration, or application import changes are needed.

## Locale variables and runtime globals

Private locale variable bindings now share the runtime-intrinsic protection used by local
functions. For example, `let undefined = Prefix` no longer replaces the numeric helper's
`undefined` sentinel in standalone output; `NaN` and infinities retain ordinary/physical output
behavior. Declaration, reference, aggregate-global export, and semantic-import names agree on
the private binding. Inline inputs retain lexical precedence. Existing source validation,
public message signatures, source properties, and named argument keys remain unchanged.
Regenerate artifacts to receive the correction; no application configuration changes are needed.

## Numeric helper type names

Canonical numeric helpers now use private JSDoc typedefs `__lgl_Numeric` and `__lgl_Decimal`.
The previous `GeneratedNumeric`/`GeneratedDecimal` names could duplicate source enum/alias
imports in a standalone message and make checked JavaScript fail. Source-owned exports and
message parameter types keep their names; executable formatter statements and public
application APIs are unchanged. Regenerate output to update the internal annotations. Compiler
helper typedefs are implementation details, not source types to import from runtime artifacts.

## Source types named Record

Generated checked JavaScript now accepts enums and aliases named `Record`. Previously the
shared argument helper's `Record<string, unknown>` annotation resolved to the source type,
causing TypeScript to report that `Record` was not generic. The helper now uses an anonymous
string-indexed object type. Source type names, public declarations, and runtime behavior remain
unchanged. Regenerate output to replace the annotation; no source or configuration changes
are required. Permanent enum/alias fixtures preserve positive and negative strict-consumer
contracts, including application TypeScript without `allowJs`.

## Private function names and JavaScript intrinsics

Unqualified backend input can contain valid local functions named `String`, `Number`, `Date`,
or other JavaScript intrinsics. These functions now receive collision-safe private bindings.
Previously `fn String(value: Number) { _ => {value} }` could recursively call itself when the
backend emitted JavaScript string conversion. Function declarations, calls, aggregate globals,
and physical semantic imports now agree on the private name. Public message/enum exports,
source properties, and named argument keys retain their spelling. Regenerate artifacts;
application APIs and configuration do not change. This correction covers local function
bindings; other declaration roles remain part of the broader identifier audit.

## Unicode first-weekday preferences

Gregorian `Y` and `yw` requests now honor valid `fw` locale keywords, such as
`en-GB-u-fw-sun`. Previously this locale silently retained Monday and could format the wrong
week/year near boundaries. Resolution uses the shared BCP 47 parser and retains the territory's
minimum-days rule. Invalid keyword types are ignored; text in private use or other extensions
cannot act as an override. Regenerate output to receive the corrected constants. Equivalent
weekday preferences produce identical formatter code, with no runtime locale parsing or extra
branches. Native previews and ordinary/physical generated modules use the same policy.
Week preferences now also honor regular region overrides (`rg`), regular subdivisions (`sd`)
and ISO week settings (`ca-iso8601`) for `Y`, `yw`, and `MMMMW`. For example,
`en-US-u-rg-gbzzzz` and `en-u-sd-gbeng` use GB's Monday/four-day convention, while an explicit
`en-US` region takes precedence over `sd-gbeng`. ISO weeks keep a four-day minimum even if
`fw` or `rg` changes their first weekday. The shared canonicalizer now applies the 147 pinned
subdivision aliases to `rg` and `sd`, including two-letter replacements suffixed with `zzzz`;
`rg` also accepts regular subdivisions. Invalid canonical preference values are ignored.
The pinned subdivision validity registry also recognizes obsolete `sd` IDs without replacement
metadata, such as `sd-gbgbn`; the same obsolete ID remains invalid as an `rg` value. Unmatched
likely regions now use the world week convention rather than the locale matcher's `und`
fallback. Regenerate artifacts to receive the corrected constants; no configuration or runtime
adapter changes are needed. Equivalent preferences retain identical formatter code.

## Constant week wording without cardinal records

Week-date requests now work for locales such as Sanskrit (`sa`), Chakma (`ccp`), Afar (`aa`),
and Latin (`la`) when the pinned count variants compile to the same pattern. Previously both
`yw` and `MMMMW` failed because these locales lack an explicit cardinal-rule record, despite
needing no plural choice. The shared planner proves variant equality before omitting that
requirement and emits one unconditional pattern.

Differing variants and missing fallback records still fail. Explicit `Plural(...)`, plural
selectors, and plural forms retain their existing missing-data errors. Native previews,
ordinary modules, physical modules, and standalone SDK output share this policy. Regenerate
artifacts; no application configuration or imports need changing.

## Gregorian month/week formatting

Use `@date(skeleton = "MMMMW")` for the pinned Gregorian month/week family. The shared
`WeekDatePlan::for_skeleton` compiles reachable count-specific wording and month symbols, and
native previews use `GregorianWeekYearPlan::month_week_number`. An incomplete first week may
be zero; a final week stays in its calendar month. This corrects the earlier missing-skeleton
rejection without a runtime CLDR interpreter, year arithmetic, or new configuration.

Schema defaults, function forwarding, named/positional calls, and ordinary/physical output use
this policy. Existing static date/year-week output, declarations, and maps keep their snapshots.
Regenerate artifacts to use the request. Pinned calendar/week data remains required; cardinal
rules are required when count-dependent wording differs.

## Gregorian year/week formatting

Use `@date(skeleton = "yw")` for the pinned Gregorian year/week format. Generation now resolves
its count-specific CLDR patterns, week number, week-based year, and locale wording during builds.
No application category selection or runtime CLDR parser is needed. The request works in schema
defaults, named functions, native previews, ordinary output, and physical message modules. Existing
static date output, declarations, and source maps retain their snapshots. Month weeks are now
covered separately below; other calendars, closest-match skeletons, and selectable non-UTC zones
remain open.

Rust backends can consume `WeekDatePlan`, `IntegerPluralPlan`, and `DateLiteralChoice` from
`linguini-codegen`; ECMAScript syntax stays in canonical templates. Public existing Rust types
and error variants remain compatible. Regenerate output to use the new request.

## Native numeric Date previews

Native `generate` now appends epoch-millisecond inputs `0` and `-1` to Date samples. Preview
arguments retain their input kind through aliases, named/forwarded functions, callable forms,
and inline captures. Numeric timestamps follow JavaScript's millisecond truncation and finite
Date range; ISO strings keep their existing validation, so the string `"0"` remains invalid.
Formatter results become strings before subsequent calls or formatters. Date errors now describe
an invalid date value rather than restricting the diagnostic to `YYYY-MM-DD`. Generated runtime,
declaration, and source-map output is unchanged. Native Date-object samples remain open.

## Constant plural classifiers

Regeneration specializes classifiers whose compiled numeric result is constant, such as Japanese
and Chinese. Finite native numbers bypass decimal parsing; category-name inputs, canonical
strings, bigints, caller coercion, and invalid-input errors retain their behavior. There are no
source, configuration, or application API changes.

## Shared native project loading and locale fallback

Native `build` and `generate` now share source loading, namespace qualification, guarded fallback
composition, and semantic IR validation. Previews inherit declared CLDR parents before the
project base: `en-AU` can use declared `en-001` messages. Invalid nested formatter options are
rejected during preview preparation too.

Regenerate projects declaring canonical siblings such as `he-IL` and `iw-IL`: each exact declared
spelling follows its own parent chain before the base. A sibling spelling no longer fills missing
messages ahead of that parent. Physical and standalone message compilation also prefer the
exact declared spelling; physical plural imports use that locale runtime's helper bindings.
Canonical metadata tags, application APIs, and configuration are unchanged.

## Native preview enum aliases

Native `generate` previews resolve enum aliases before selecting a form implementation, matching
generated code. Properties, callable plural forms, and named payload forwarding through an alias
no longer report a missing alias-named implementation. Source syntax and generated artifacts are
unchanged.

## Native preview formatting scopes

Native `generate` output now uses the same default-formatting and inline-capture lowering as
code generation. Plain numeric, currency-alias, and Date payloads inside named functions no
longer receive unintended display formatting. Schema defaults still apply at message display
sites and survive inline bindings/captures. Alias cycles remain errors. Generated JavaScript,
declarations, and maps are unchanged.

## Formatted function inputs

Regenerate output to preserve explicit formatters inside call arguments and inline payloads.
Previously those annotations could be silently ignored, rendering `1234.5` instead of English
`1,234.5`. Formatted inputs have type `String`; payload signatures must accept that result.
Native validation now checks nested names/options/input types before output changes, and preview
rendering follows the same explicit formatting. Inputs retain left-to-right, eager, single
execution, including unused inline payloads. Forwarding a value without an explicit formatter
retains its existing behavior. Backend planning reuses `expression_value_type` for scoped results.

## Shared source provenance

Backend authors can use `linguini_codegen::semantic_symbol_span` and
`semantic_symbol_source_ids` for effective locale declarations and their nested source identities.
Dependency slices and ECMAScript semantic artifacts now reuse one traversal. Generated code,
declarations, source maps, and application APIs remain unchanged.

## Shared dispatch planning

Backend authors can reuse `linguini_codegen::form_dispatch_plan` and `function_dispatch_plan`
with build-time selector/text projections. The common crate owns branch traversal, fallback
precedence, and grouping projected results; ECMAScript retains syntax and rendering. Function
selector indices refer to the complete parameter list, excluding named payloads from dispatch
depth. Generated code, declarations, maps, and application APIs are unchanged.

## Shared namespace planning

Backend authors can reuse `select_schema_paths`, `locale_module_for_schema`, `namespace_module`,
`root_module`, `top_level_message_namespaces`, and the nested `MessageTree` from
`linguini-codegen`. ECMAScript generation delegates those semantic operations to the common
crate. Generated code, declarations, maps, and application configuration remain unchanged.
Empty schema groups continue reserving namespaces without creating runtime values or extra modules.

## Shared number-format planning

Backend authors can reuse `linguini_codegen::NumberFormatPlan` and its affix/grouping/scientific
models to resolve selected CLDR number patterns. ECMAScript generation now projects this typed
plan into its existing templates, keeping syntax and helper naming in the backend. Generated
JavaScript, declarations, maps, and application configuration remain unchanged.

## Unchanged fraction rounding

Regenerate number/currency output to reuse canonical decimal digits when unit rounding leaves
them unchanged. Required fraction padding now belongs to the formatter; optional fraction
trimming is skipped for unchanged canonical input. Visible precision, ties, carries, rounding
increments, significant/scientific behavior, negative zero, and input limits remain unchanged.
Message and formatter APIs require no application changes.

## Shared locale fallback composition

Backend authors can use `linguini_codegen::compose_locale_modules` with locale modules ordered
from base to highest priority. It preserves whole-declaration replacement, declaration order,
source identities, and override history; the caller validates the composed locale against its
schema. ECMAScript generation now reuses this implementation. Application configuration,
generated output, declarations, and source maps are unchanged.

## Removed branch-table helper

Regenerate output: generated `shared.js` and its declarations no longer export `selectBranch`.
Static form and function dispatch already compiles to direct branches and returns. Applications
should call generated messages/forms/functions instead of importing this obsolete implementation
helper. Literal translation text containing `selectBranch(` no longer creates a false shared
runtime import in physical message or semantic modules. Named-argument normalization is unchanged.

## Shared project locale planning

Backend authors can reuse `linguini_codegen::ProjectLocalePlan` for project locale fallback
matching and finite resolution-table construction. ECMAScript generation now consumes that
shared model. Declared names, alias precedence, fallback order, generated code, declarations,
and source maps retain their existing behavior; applications need no configuration changes.

## Currency-specific separators

Regenerate currency output to honor the selected numbering system's `currencyDecimal` and
`currencyGroup` overrides. Austrian German currency now groups with `.`, and Swiss French
currency uses `.` for decimals; ordinary number formatting retains its own symbols. The native
preview follows the same selection. Missing overrides resolve to ordinary decimal/group symbols
during CLDR compilation, without runtime option handling.

Rust callers constructing `CurrencyFormatData` must supply its resolved `decimal_symbol` and
`group_symbol`. Compiled currency lookups supply both automatically.

## Compiled scientific symbols

Rust callers constructing `NumberFormatData` must now provide `plus_sign` and
`exponential_symbol` from the selected numbering system. Compiled locale lookups supply these
automatically. Internal scientific lowering uses those symbols and the existing `minus_sign`,
including bidirectional marks. Numeric digit substitution preserves literal separator/sign text.
Scientific and engineering styles remain outside the public formatter configuration.

## Native integer number formatting

Regenerate number and currency output to bypass decimal-text parsing for safe integer numbers
and bigint inputs. Input types, rounding, localized output, negative zero, caller coercion, and
the 8,192-digit bound retain their semantics. Fractional numbers and decimal/exponent strings
keep the validating parser. No application API changes are required.

## Explicit alternate-link helper

`LinguiniWebLocale`, message-enabled web adapters, request contexts, and browser/framework
controls no longer attach `alternateLinks`. Import `alternateLinks` from
`@antepodeum/linguini-web` and call `alternateLinks(web, url, input)` where alternate-language
metadata is needed. For request-local origins, forward the same input previously captured by the
request context, for example `alternateLinks(web, event.url, { url: event.url })`. Browser callers
can use their configured origin or current location. The SvelteKit examples keep an explicit
project-level wrapper. Locale order, `x-default`, prefix policies, external URLs, and error handling
are preserved. Unused SEO code can now be removed from routing and browser-control bundles.

## Native numeric preview digits

The native `generate` preview now applies the locale's pinned decimal digit symbols to number
and currency bodies. Persian, Arabic, Adlam, Chakma, and other selected numbering systems no
longer print ASCII digits in these previews. The shared substitution handles Unicode scalars,
including supplementary and non-contiguous symbols. Literal digits in affixes stay unchanged;
negative zero, grouping, separators, rounding, and accounting affixes retain their semantics.
Generated application code and its formatter inputs are unchanged.

## Native date preview planning

The native `generate` preview now uses the shared date-request selector, pattern compiler,
calendar symbols, day-period plans, and Gregorian week rules. `skeleton`, `time_style`, and
combined date/time requests previously fell back to a date style. They now render the selected
fields, UTC zone labels, contextual names, localized digits, and year-of-era/week-year values.
Invalid requests report the shared structured diagnostic. Date samples use `YYYY-MM-DD` at UTC
midnight, including year zero; generated application formatter inputs retain their existing
Date/epoch/ISO contract.

## Preview currency symbols

The native `generate` preview now resolves currency symbols from the same pinned locale lookup
as ECMAScript generation. Its fixed symbol map previously printed `$` for Polish USD and `¥`
for Japanese JPY, for example; those locales now use `USD` and `￥`. Lowercase codes still resolve
case-insensitively and unknown three-letter codes retain their uppercase fallback. Message source,
formatter options, fraction digits, and rounding are unchanged.

## Locale minimum grouping thresholds

Regenerate number and currency output to honor CLDR `minimumGroupingDigits` after rounding.
Spanish and Polish keep four-digit integers ungrouped; Ewe also keeps five-digit integers
ungrouped. The selected threshold compiles into a numeric constant. Group widths, digit
mapping, public formatter options, and precision are unchanged. The native preview applies the
same threshold. Direct Rust `NumberFormatData` construction must now supply
`minimum_grouping_digits` as well as `minus_sign`; compiled lookups provide both automatically.

## Gregorian week-year fields

Regenerate to use pinned Gregorian patterns containing `Y`, including Scottish Gaelic `yMMM`.
Week-year boundaries follow the requested locale territory's week conventions in UTC. They can
fall in the previous or next calendar year. `Y` keeps a signed extended year (including year zero
and BCE); `y` continues to render the year of era. `YY` keeps two absolute digits with the sign,
while other widths set minimum digit counts. Numeric symbols come from pinned locale data.
Only selected patterns emit boundary helpers; no CLDR week table or pattern parser is shipped.
Other week-number fields, other calendars, and selectable non-UTC zones remain under remediation.

## Localized numeric affixes

Regenerate output to receive pinned CLDR minus signs and bidirectional marks in decimal and
currency negative affixes, including negative zero. Implicit negative patterns previously used
ASCII `-` for every locale; explicit sign placeholders also remained unlocalized. Affix symbols
and percent/per-mille scaling now compile before CLDR quote information is discarded, so quoted
symbols stay literal. Public formatter options and inputs are unchanged; percent styles remain
internal pattern support rather than an exposed formatter option. Rust callers constructing
`NumberFormatData` directly must supply `minus_sign`, including the locale's bidi marks. Callers
using `compiled_number_formatting` receive it from the pinned artifact automatically.

## Application origin validation

Web adapter construction now rejects malformed or relative `environment.origin` values,
non-HTTP(S) URLs, and embedded credentials with `TypeError("Invalid web environment origin")`.
Use an absolute HTTP(S) string or URL object, or omit the field for request/browser defaults.
Valid base URLs retain their existing path resolution and URL object identity. The origin getter
is read once during construction. Per-request origin parsing, explicit URL errors, and link-safety
helpers' false result for invalid links are unchanged; validation belongs to the application runtime.

## Automatic cookie security honors application origins

For `secure: "auto"`, cookie serialization and server writes now use the configured environment
origin when the call does not supply an origin. Previously that configuration was ignored, so a
server-side HTTPS application could serialize a cookie without `Secure`; a browser location could
also override an explicit application origin. Per-call origins still take precedence, browser
location remains the next fallback, and explicit secure flags/fixed policies retain their priority.
No configuration migration is required. The frozen parity oracle receives the effective origin
explicitly for this intentional behavior correction.

## Explicit cookie persistence helpers

`LinguiniWebLocale` and `LinguiniWeb` no longer expose `serializeLocaleCookie` or `setLocaleCookie`
methods. Replace `web.serializeLocaleCookie(locale, input)` with
`serializeLocaleCookie(web, locale, input)`, imported from `@antepodeum/linguini-web`. Replace
`web.setLocaleCookie(target, locale, input)` with `persistLocaleCookie(web, target, locale, input)`
from `@antepodeum/linguini-web/server`. The existing server helper now owns persistence directly.
Both helpers infer the allowed locale from `web`; the locale argument cannot widen that type.

SvelteKit hooks, switch handlers, and browser controls use these helpers internally. Their normal
configuration is unchanged. Serialization, cookie-sink precedence, append-only response headers,
HttpOnly/secure policy, and browser cancellation remain covered. Routing-only web consumers can
now shake the persistence implementation; normalized application policy still belongs to the web
object. Generic framework controls and exact page delivery retain their separate audit criteria.

## Shared module-edge rendering

Structured imports and re-exports now render through one canonical template instead of repeated
Rust string assembly. Public Rust module models and generated JavaScript/declaration/source-map
bytes are unchanged; no application migration is required. The schema alias test now exercises
production declaration generation, and the obsolete test-only re-export emitter is removed.

## Native integer plural inputs

Regenerate plural output to avoid string conversion and regex parsing of safe integer numbers
when the selected CLDR rules can use a bounded integer suffix. Public category-name results,
visible fractional digits, non-safe-integer input validation, caller coercion, and resource limits
are unchanged. Other inputs still use the original parser. The shared Rust
`PluralDecisionPlan::integer_input_suffix_modulus` proves guard and operand eligibility;
each backend must also bound its native input precision. ECMAScript enables this path only for
suffixes of at most 15 digits. This adds a small input branch; complete-consumer size and local
timing measurements are recorded in the generated-code cost audit.

## Plural data follows operation demand

Numeric and literal projects can use known locales such as `ccp` without an unused plural rule
set. Project construction checks the lowered locale and rejects `MissingPluralRules` only when
a plural operation requires absent data. Such failures now occur during validation, before
artifact generation; existing valid output remains intact. Formatter and direction requirements
retain their checks. Public message calls and runtime configuration are unchanged.

## Supplementary Unicode digits

Regenerate number and currency output for locales such as `ff-Adlm`. The old renderer indexed
a supplementary digit alphabet by UTF-16 unit, producing isolated surrogates. Shared scalar
planning now selects direct code-point arithmetic for consecutive alphabets and compiled
branches for other mappings. Date and number lookups share one template; public input types
and default-numbering-system selection remain unchanged.

## Compiled date digit substitution

Regenerate localized date formatters to remove per-field digit-alphabet arguments, symbol-array
construction, and numbering-system validation. Date inputs, output, and getter observations are
unchanged. ASCII-only date output omits localization support; supplementary Unicode symbols
use compiled scalar lookup. A build-time proof coalesces safe date output into one substitution pass;
literal digits and textual labels retain their original form. Rust backends can reuse
`linguini_codegen::decimal_digit_symbols` and `can_localize_date_result` before rendering
target syntax.

## Unpublished Cargo test helpers

The LSP and CLDR generation tool now refer to workspace-only `linguini-test-support` without
a registry version. Cargo omits that development dependency from published manifests, avoiding
a dependency on an unpublished package. Packaged-documentation staging still includes the
complete path-dependency manifest closure and verifies its external versions against the
workspace lock. Application runtime dependencies and APIs are unchanged.

## Shared type expression rendering

Primitive types and enum literal unions now render through the same ECMAScript type template
for declarations and JSDoc. Public Rust renderer signatures, generated bytes, and source maps
are unchanged; no application migration is required.

## Generated locale text references

Regenerating output removes redundant `String(...)` wrappers around generated locale text
variables, including imported variables and aliases. Message APIs and raw caller-input coercion
are unchanged. The shared Rust codegen crate exposes `StringResultFacts` for emitters that need
the same scope-aware proof; create it from the full visible locale before projecting individual
artifacts, and supply local binding names when querying it.

## Form selector bindings

Regenerate forms that reference a locale variable named `value`, or use a custom plural helper
with that name. The unnamed selector previously shadowed that binding and could render the
numeric selector instead of the locale text, or call the selector as a function. It now selects
a free name such as `value2` or `value3`. Non-colliding output and explicit source parameter names
remain unchanged; positional and named message calls keep their API.

## Locale metadata boundaries

`createWebLocaleI18n` now copies only the declared locale metadata fields from its runtime input.
It no longer enumerates the full runtime, reads unrelated catalog getters, or forwards extra
message APIs and symbol properties. Locale normalization/direction callbacks and the declared
web API remain available. Use `createWebI18n` for message-enabled request contexts; import the
generated locale metadata module when constructing catalog-independent hooks.

## Shared LSP host services

The editor includes a WASM worker fallback and a browser entry. Existing desktop native settings
keep their resolution order and LSP handshake, including explicit command/argument launchers.
`linguini.server.runtime` defaults to auto; native and wasm select a runtime explicitly. Auto uses
WASM for browser/virtual workspaces and when an automatic native candidate cannot be used.
The universal VSIX contains the worker; native VSIX packages include it alongside their binary.
No first-activation executable download is needed. Extension builds require the WASM Rust target;
`pnpm bundle` generates the worker and bindings in ignored build output.

Native CLI and editor protocol behavior are unchanged. The Rust LSP crate now also exposes
`MemoryWorkspace`, saved-file/source-root types, and a scheduler interface for embedding the
same server without native filesystem access. Host-provided paths use the config crate's
shared namespace classification. Updating saved files requires the normal watched-files
notification; it does not replace open-buffer synchronization. No editor setting or application
source migration is required. `source_plan` returns valid roots together with invalid-project
errors; `source_roots` retains its strict all-valid result for existing embedding callers.

## Functions without selectors

Native `check` and `build` now accept a single `_` body in functions with only named payload
parameters, including `String`, numeric, Boolean, and Date payloads, and in zero-argument
functions. This agrees with the existing validated-IR and codegen contract. Other keys and
nested dispatch remain errors; failed builds preserve existing generated output.

The existing `fn_without_strings` lint name is retained. It now suggests `form` only when
there are no named payload parameters; a named Date or numeric payload requires `fn` too.
LSP diagnostics use the same correction. Regenerate from the original sources; no application
API or generated runtime implementation changes are required.

## Framework-owned Vite adapters

Replace `@antepodeum/linguini-vite/svelte` imports with
`@antepodeum/linguini-svelte/vite` and rebuild. The former subpath is removed.
SvelteKit's existing `@antepodeum/linguini-sveltekit/vite` entry keeps its public name
and composes the Svelte package's adapter. Install the shared Vite package alongside the
Svelte package when using its build integration; runtime-only use keeps Vite optional.

Vue uses `@antepodeum/linguini-vue/vite` with the official Vue Vite plugin; React uses
`@antepodeum/linguini-vite` directly with its compiler. These representative integrations
link the compiled base-locale `lgl` leaves. Request-selected Vue/React locale hosts remain
under remediation. The framework adapters are TypeScript sources in their owning packages;
only builds emit their JavaScript, declarations, and maps. The Vite core owns shared planning,
watching, and exact linking and has no framework exports or production dependencies.

## Gregorian quarter fields

Regenerate to use exact quarter skeletons such as `@date(skeleton = "yQQQ")` or `yQQQQ`.
Selected `Q`/`q` fields compile in Rust: widths one/two emit UTC quarter arithmetic with
localized digits and optional padding; widths three/four/five select abbreviated/wide/narrow
quarter symbols in the matching format or standalone context. Equal name vectors share data.
Message signatures and the Date input contract are unchanged. Unused quarter formatters and
tables are removed by bundlers. Week-number fields, other calendars, and selectable non-UTC
zones remain under remediation; unsupported selected patterns still fail before output writes.

## Shared date name tables

Regenerate output to share identical month/weekday vectors across the locale runtime's selected
formatter requests. Message APIs and input evaluation are unchanged. Different widths or contexts
retain their own data when their values differ. A formatter can reference a table first declared
beside another formatter; bundlers retain that table independently of the unused formatter body.
Executed supplied-Date getter counts and real Vite consumer graphs verify this separation.

## Exact Gregorian date skeletons

`@date(skeleton = "yMMMd")` and `@date(skeleton = "Hm")` select exact IDs from the locale's
pinned Gregorian `availableFormats`, including annotations on schema aliases. They cannot be
combined with `style` or `time_style`; those existing options retain their output. Skeleton
lookup and pattern compilation run in Rust. Generated functions receive only the date input,
with selected getters, literals, and name tables; no skeleton catalog or pattern parser ships.
Missing IDs and unsupported fields fail before the native build changes owned output.

This adds exact-ID selection. Closest-match/width-adjusted skeletons, combined date/time
synthesis, other calendars, and selectable non-UTC zones remain under remediation. Rust users
of `linguini_codegen::date_request_pattern` must now handle its `Result<String, DatePatternError>`;
the shared planner reports missing IDs and conflicting request forms instead of selecting a default.

## Svelte script insertion boundaries

Update the Vite adapter and rebuild; application syntax and configuration are unchanged. Message
imports, static-link helpers, and loader cleanup now share quote-aware tag boundaries, preserving
`>` inside TypeScript `generics` attributes. HTML/CSS comments containing script-like text cannot
receive injected code. Script module attributes are read outside quoted values, so a generic
constraint containing the word `module` remains an instance script. Both Svelte compiler targets
verify the resulting source. Raw script/style bodies remain the host compiler/preprocessor's input.

## Exact namespace message imports

Rebuild with matching native compiler and Vite adapter versions. ECMAScript namespace imports
now track static message roots, including `catalog.lgl` for the base entry and `catalog.l` or
`catalog.messages` for Svelte. Message values, function references/calls, property/method tails,
and finite computed access use the same exact leaf bindings as named imports. Separately used
locale metadata and `setLocale` exports preserve their original namespace import. Escaped
namespaces, unknown exports/computed roots, and provider/catalog exports fail explicitly;
they cannot fall back to a whole catalog. Default and mixed import declarations remain outside
this receipt.

Custom manifest producers represent a namespace binding with `imported: "*"`; each reference's
provenance still names its actual message export. Namespace bindings require boolean
`retain_import`, set only for separately used metadata/control exports. Named bindings retain
the existing contract and may omit that field. This is build metadata, with no client table
or namespace wrapper added to the generated application.

## Nested deferred message closures

Rebuild with matching Vite and SvelteKit adapters; application imports and configuration stay
unchanged. Nested imports now prepare their additional messages before evaluation. Child payloads
exclude messages guaranteed on every parent path; a direct child import still gets the full
additional set for that path. Shared entries use the intersection of available messages, and
the build handles cyclic parent relationships conservatively. Transit-only modules add no files.
Export variants are compared after those parent facts are applied. Different already prepared
reads can share the same additional closure, including an empty closure with no extra file.
Partial parents, independent direct paths, and self-justifying cycles cannot supply missing
preparation; genuinely different remaining message sets still fail explicitly.

`compilePageBundles` also returns optional build-only `deferredImports`, mapping each page to
its message-bearing lazy module targets, including transit modules. Framework adapters use it
to identify nested callers without following global framework route registries. Hosts must
prepare every message-bearing boundary along the application's import path before evaluating
it; a child's record set can depend on already prepared ancestor records. This metadata is not
shipped as a client catalog or route manifest. Existing inlined-import, distinct call-site, and
unrelated shared-caller guards remain in place.

## TypeScript compiler paths and source discovery

No application change is required. Emit planning normalizes input file names to TypeScript's
`/` spelling, including source-file command lines with an explicit `rootDir`. Directory-watch
events are resolved and compared by path components using the host's separator and case rules.
This restores discovery of added or removed schema/locale namespaces on Windows, where
TypeScript reports `/` paths alongside native `\` paths. Watch deadlines and output-validation
rules are unchanged. The macOS TypeScript job uses `macos-latest` after GitHub rejected the
older runner configuration; mutation testing remains limited to pull requests.

## Conditional deferred loaders

The public `@antepodeum/linguini-sveltekit/load` import and `createClientLoad` calls stay unchanged.
Rebuild with the matching SDK to select the base loader when no additional lazy messages exist.
Deferred navigation preparation and its registration now live in the internal deferred-loader
module, linked only for client builds that need it. SSR retains the base implementation.
This removes unused callback branches without relying on the bundler to prove that a mutable
registration is never assigned. Both implementations share payload loading. The enhanced
loader preserves parallel preparation and request-scoped publication without an extra async
wrapper or a repeated metadata lookup.

## TypeScript output paths and clean CI installations

No application change is required. The TypeScript adapter now serializes derived native output
arguments with `/` separators on Windows as well as Unix hosts. Output ownership, containment,
and traversal validation remain unchanged. Existing build, declaration-only, reference, and
watch tests continue exercising actual native output on the platform matrix.

CI retains Node 20 compatibility tests while running pnpm 11 as a standalone executable there.
Packed-consumer tests accept both script and native package-manager launchers and use
`--prefer-offline`, allowing missing registry metadata to be fetched on clean runners.
The dependency policy runs on the installed stable host toolchain; Tokio and URL dependencies
are updated for RUSTSEC-2026-0386 and RUSTSEC-2026-0592 without advisory suppressions.

## Deferred dependency planning

Application imports and configuration are unchanged. Build analysis now separates initial
messages from each lazy entry's consumed exports, then re-shakes those exports independently.
This preserves exact demand through shared eager/lazy modules, pure re-exports, default exports,
and nested boundaries. Async facades exist only in the in-memory analysis build; they are not
application artifacts and do not impose top-level await on the configured browser target.
The SvelteKit adapter now prepares new deferred messages before evaluating the application
import. Use matching Vite/SvelteKit SDK versions and rebuild; no application configuration or
manual preload call is required. Initial and additional closures remain separate, and identical
additional closures share cached assets across pages. Activated closures follow locale changes
on the same page; another page does not inherit them. Inlined imports, callers shared with
unrelated pages, and distinct additional message sets from the same lazy module on one page
fail explicitly until those shapes can be lowered exactly.

Framework authors can use `compilePageBundles`, `pageMessages.resolve`, and
`pageMessages.analysisModule` to implement preparation. `compilePages` retains its strict
initial-only contract. The SvelteKit build links its internal deferred entry only through
owning page nodes. Applications should continue importing the public `/load` helper;
`/internal/deferred` is SDK implementation plumbing.

The SvelteKit client's private package import now selects separate browser and server locale
readers. Server message calls use `getRequestEvent().locals.linguini`, supporting loads/actions
without component context. Hook setup remains required. Access after `await` depends on the
platform's `AsyncLocalStorage` support; explicit locale providers remain available otherwise.
Browser hydration still uses serialized page metadata or the server's HTML language marker.

## Catalog-independent Svelte setters

Regenerate Svelte bindings with the matching runtime SDK. The locale helper now imports
`createLocaleSetter` and its declarations use `LocaleSetter`; custom `--svelte-host-module`
implementations must export both alongside `createLocaleState`. The generated facade directly
re-exports the helper's setter and passes the same function to `createLinguiniRune` through
its optional `setLocale` option. Existing calls remain compatible, including setter identity.
Latest-choice and request-identity guards share one implementation.

The Svelte Vite adapter declares generated entry initialization discardable using the new
optional boolean `entrySideEffects`. Custom host factories must not depend on construction
effects surviving an unused entry. Required effects belong in explicit application lifecycle
registration. A real native-output/SDK/Vite regression now builds and executes a setter-only
entry in one chunk with no message catalog modules, replacing its previous ten-chunk graph.

## Superseded locale changes

Client controls now let the latest pending locale choice take precedence within its request
context. A change superseded during preparation resolves to the current locale without publishing
state or starting persistence/navigation. Checks after asynchronous persistence and navigation
also prevent stale publication or cleanup from overwriting a newer change. Destroyed controls
invalidate their pending changes.

`createLocaleControl` accepts an optional `getRequestIdentity` callback; replacing that identity
invalidates pending preparation. SvelteKit controls supply their page metadata identity by default.
No SvelteKit application change is required. SSR retains preparation-only behavior and never reads
the client identity callback.

Direct `createLinguiniRune` setters now apply the same latest-choice rule. Its optional third
argument accepts `getRequestIdentity`; SvelteKit supplies the page identity automatically,
including for the generated `setLocale` export. Existing two-argument calls remain valid,
and server providers never read the client identity callback.

## Live SSR page preloads

The SvelteKit hook now recognizes live hydration hints in HTTP `Link` headers as well as
prerendered HTML tags. It prepends only the selected page/locale payload, with
`media="(scripting: enabled)"`, preserving existing hints, streaming bodies, and optional
cookie persistence. Payload URLs follow browser URL resolution, including relative paths,
Unicode, existing percent escapes, and external asset origins. No application API change
is required. Chromium verifies that the payload arrives before framework entry execution
and that scripting-disabled clients request none.

## Site output is generated during builds

The site no longer commits a compressed message archive, its manifest, or native highlight
tokens. Archive restoration and packaging commands are removed. `pnpm --dir site build`
generates current messages and examples; development watching runs the real compiler.
`pnpm --dir site linguini:build` prepares generated declarations before a standalone check.
Outputs remain ignored under `site/src/lib/generated` and `site/.svelte-kit`.

Source-checkout builds use the stable workspace Rust compiler. Run
`cargo build --locked -p linguini-cli` once to populate dependencies; subsequent generation
runs offline. `LINGUINI_COMMAND` can select a supplied compiler executable instead.
CI and Pages build the compiler before generating the site. The repository gate rejects
tracked application build output; golden test expectations remain reviewable fixtures.

## Rust toolchain and minimum

Source development and CI use the moving `stable` toolchain. The tested minimum is declared
once in `[workspace.package].rust-version`; CI reads it and verifies the whole workspace with
`cargo-msrv`. The current minimum is Rust 1.87, found by checking all workspace targets while
ignoring the previous declaration. Native npm/VSIX binaries do not require an installed Rust
toolchain. See [contributing](../CONTRIBUTING.md) for the discovery command.

## SvelteKit shared locale metadata

`createLoad()` now tracks `event.url.pathname` so SvelteKit refreshes shared root-layout locale
metadata when a localized URL changes. Direct `export const load = createLoad()` usage needs
no changes. Custom load composition should pass the original event to the helper, including
its `url`, instead of constructing a locals-only object. Locals-only calls remain accepted for
metadata extraction outside SvelteKit's dependency-tracking lifecycle.

This fixes metadata invalidation; exact composed-page payloads and preparation during ordinary
cross-page/cross-locale client navigation remain open remediation requirements.

## Framework-neutral Vite imports

The default `@antepodeum/linguini-vite` entry now accepts the native non-Svelte manifest,
whose `runtime_helpers` is empty. Imports of `lgl` from generated `index.js` link directly
to their base-locale message implementations, including aliased and bounded computed
access. Rebuild generated output to refresh tracked `lgl` import bindings. No message
call-site change is required.

Custom adapters may omit `localeHelper` for the same static base-locale behavior. Select
that descriptor explicitly when the adapter needs request/client locale state. Only
stateful dynamic adapters require top-level await and the default ES2022 target; plain
`lgl` imports keep the application’s normal target. Svelte and SvelteKit adapters retain
their stateful behavior. This does not complete composed-page locale delivery.

## Deferred page demand

The composed-page guard now checks the page's actual deferred import consumption instead
of retaining every public export of a lazy module. Deferred reads that reuse prepared page
messages work without another message request. Unused shared exports remain excluded even
when another page uses them. The analysis preserves dynamic boundaries to avoid Vite 8's
namespace retention during import inlining; each compiled message payload is still one file.
Deferred boundaries needing new messages continue to fail explicitly until their preparation
protocol is implemented.

## Exact message leaves before JavaScript operations

Static property and method tails now resolve their known message prefix. For example,
`l.main.title.toUpperCase()` imports `main.title` and preserves `.toUpperCase()`;
captured callable leaves also preserve `.call(...)` and other static operations.
Unresolved uses of a named generated message namespace now fail the Vite build instead
of retaining its catalog import. Resolve the message leaf or configure a finite dynamic
allow list. Unrelated imports with the same local name remain unaffected.

## SvelteKit composed-page payloads

With the SvelteKit Vite adapter, import `getPagePayload` from
`@antepodeum/linguini-sveltekit/server` and pass it to both `createHandle` and `createLoad`.
Add `createClientLoad<Locale>()` from `@antepodeum/linguini-sveltekit/load` as the universal
root layout load. Await the server helper when composing its result with another load.
Metadata-only hook consumers can keep their existing synchronous `createLoad()` calls.

The adapter returns a Vite plugin array, still used as `plugins: [linguini(), sveltekit()]`.
It now selects one composed-page/locale payload instead of independent source-file registries.
The root load awaits incoming messages without changing the currently rendered page. Client
controls prepare alternate locales before publishing an override; a new request context
invalidates an old override. The server hook preloads the selected payload for script-enabled
clients. No route/locale loader table or message implementation is serialized into server data.

The shared Vite build API accepts page roots and selected nested export paths, preserving tree
shaking of unused shared functions. Payload source maps follow the application’s normal,
hidden, inline, or disabled setting. Deferred localization boundaries and the remaining
framework/configuration matrix are still open remediation requirements.

## Svelte request identity

`createLocaleState` accepts an optional `getRequestIdentity` callback. A client override
belongs to the request identity present when it was selected; a new identity makes the
authoritative request locale take effect. Selecting the same client locale again remains
reactive. Consumers without this callback retain their existing override behavior. SSR does
not read client request identities or publish client overrides.

## Captured message functions

Parameterized leaves can now be used as function values through native bundler manifests
and Vite transforms. `const format = l.main.items` retains the callable instead of being
reported as an arity mismatch or invoked during assignment. Direct calls and parameterless
values retain their existing forms. Rebuild generated manifests after upgrading; calling a
parameterless string as a function still remains an arity error.

## Generated numeric implementations

Rebuild generated output to receive the formatter/plural specialization changes. Message call
signatures, accepted plural category names, validation errors, and locale payload interfaces
remain compatible. Proven scientific rounding carries now retain the requested precision
(for example, zero-fraction `9.9` produces `1E01`). Shared Rust rounding/plural plans expose
build-time carry and finite integer-modulo exhaustiveness proofs for backend reuse.

Plural output can return a proven final category directly; rejected fractions and malformed
inputs keep their original behavior. Date validation constructs only the returned date and
checks calendar fields before allocation. Successful named-argument calls avoid unknown-key
collection/conversion arrays and sorting, while preserving validation before value reads.
These implementation changes require no application call-site migration.

## npm packages and runtime adapters

Install the CLI as `@antepodeum/linguini`; supporting npm packages use
`@antepodeum/linguini-*`. Replace earlier scope names in dependencies and imports, then refresh
the package-manager lockfile. The executable remains `linguini`.

Web and SvelteKit are reusable TypeScript runtime packages. Move routing, cookie/storage,
locale-source, and hook options into application modules using `@antepodeum/linguini-web`
and `@antepodeum/linguini-sveltekit`. Configure request handling with `createHandle`, URL
rerouting with `createReroute`, and serialized request metadata with `createLoad`. Runtime
resolvers are explicit capabilities: `pathLocale()`, `cookieLocale()`, `requestLanguage()`,
and `localStorageLocale()`. Server hooks use locale metadata without loading message catalogs.

Remove `[web]` and `targets.ts.framework` from neutral TOML. The temporary
`linguini.config.mjs` / `defineModules` policy DSL, native JSON module plans, and `/modules`
SDK entries have been removed. Native generation does not evaluate runtime configuration.
See [Web and SvelteKit](./web-sveltekit.md) for the current application setup.

SvelteKit projects import the build adapter from `@antepodeum/linguini-sveltekit/vite`.
Standalone Svelte uses `@antepodeum/linguini-svelte/vite`. These adapters share the Vite core;
web/server handling remains in the runtime packages. Direct native builds can request thin
Svelte bindings with `--svelte`; `--svelte-host-module MODULE` selects an integration runtime
exporting the state/provider factories and request-locale getter. This selects imports and
types, independently of routing or persistence settings.

Rust integrations must remove the retired `linguini-codegen-ecmascript-web` and
`linguini-codegen-sveltekit` dependencies. `linguini-codegen-svelte` now emits project bindings
backed by `@antepodeum/linguini-svelte`, with an optional `host_module`; it emits no generic
runtime bodies. Rebuild owned output to remove old `web*`, `sveltekit*`, and Svelte control/effect
artifacts. Application controls use the npm adapters. npm runtime source is TypeScript; builds
produce JavaScript, declarations, and source maps.

## Compiled UTC names and date-pattern validation

Rebuild generated output to use pinned localized long UTC names in full time styles. Short
styles retain the pinned short name; missing names fall back to the locale's GMT-zero label.
Chinese patterns retain their zone-before-time order. Zone labels are compiled string literals,
with no runtime CLDR lookup or host `Intl` dependency. Unsupported date fields or unclosed CLDR
quotes now fail generation instead of emitting a fabricated `UTC` value.

Rust consumers constructing `TimeZoneFormatData` must provide `utc_standard_long` and
`utc_standard_short` (`Option<&'static str>`). `None` selects the GMT-zero fallback. The shared
`linguini-codegen` crate exposes `date_request_pattern` and `compile_date_pattern` for other
backends. Selectable non-UTC zones, calendars, and skeletons remain under remediation.

## Standalone weekdays in full dates

Rebuild generated output to correct Finnish and Inari Sami full-date formatting. Their pinned
CLDR patterns select standalone weekdays (`c`); these now use the appropriate weekday names
instead of the old `UTC` placeholder. Named formatting/standalone contexts and short/narrow
widths remain distinct. The public date request and JavaScript/declaration APIs are unchanged.

## Runtime link and request snapshots

Import `shouldLocalizeHref`, `shouldLocalizeLink`, `localizeHrefAttribute`, and `localizeLinkHref`
from `@antepodeum/linguini-web` where needed. These optional helpers are no longer methods on
routing metadata, request contexts, or browser/framework controls. Pass `web` first, for example
`localizeLinkHref(web, href, locale, attributes, input)`. Existing application wrappers can retain
their call shape by reading `linguini.locale` inside the wrapper at call time; see the web guide.
For request-specific origins, forward input such as `{ url: event.url }` explicitly.

Use `localizeLinkHref(web, href, locale, attributes, input)` for combined validation/localization.
`localizeHrefAttribute` delegates to it without additional attributes. These operations resolve
one environment/URL snapshot, and exclusion predicates retain order and receive an isolated URL.
Runtime DOM link handling uses this operation instead of separately checking and reparsing a link.
Request contexts resolve direction once; `direction`, `textDirection`, and `htmlAttrs.dir` share
that value. Direction callbacks are evaluated again for each new context, never cached globally.

## SvelteKit hydration locale and redirects

Keep `lang="%linguini.lang%"` on the root HTML element, as shown in the web guide. The server
hook writes the selected request locale there. During initial hydration, route modules can load
before SvelteKit publishes `page.data`; the client adapter uses that server-selected HTML locale
until serialized page metadata is available. This avoids loading a base-locale payload for a
non-base SSR page. It does not infer the locale from browser preferences or persistence.

The SvelteKit hook ignores slash-only canonical URL differences. SvelteKit retains ownership of
`trailingSlash`; the adapter still redirects missing or incorrect locale prefixes. This prevents
redirect loops when the framework requires a trailing slash.

## Generated string results and form formatting

Rebuild generated output to remove redundant interpolation coercion around validated function,
inline-function, callable-form, and plural results. Raw input coercion and public call signatures
are unchanged. Generated form objects use multiline formatting; runtime behavior and the
canonical JavaScript/declaration boundary are unchanged.

## Compiled number fraction presence

Rebuild output to remove impossible empty-fraction branches in fixed-fraction currency/number
requests. Non-scientific integer-only requests omit unused fraction/formatted bindings; mandatory
fractions concatenate directly, and identical positive/negative prefixes are selected during
generation. Optional/significant fractions, negative zero, grouping, localized digits, padding,
and public formatter signatures are unchanged.

## Bounded plural integer operands

Rebuild output to obtain build-selected decimal integer suffixes for plural rules such as Russian.
These rules now read only the needed final digits, use exact number arithmetic up to 15 digits,
and omit a redundant largest-modulus operation. Wider suffixes retain bigint arithmetic. Direct
integer comparisons and non-decimal moduli retain the full integer path; numeric input validation,
visible fractions, compact exponents, category strings, and public call signatures are unchanged.
The backend-independent `PluralRequirements::integer_tail_digits` exposes the static proof to
other generators. It does not cache plural results across message/function calls.

## Generated argument normalization

Static message parameter names are initialized once per generated module. Positional calls reuse
their argument array; named calls retain own-property checks, unknown-key errors, and schema-order
property reads. Direct users of `normalizeMessageArgs` must not rely on a fresh positional copy.
Mutable inputs retain mutable result typing; readonly inputs have a readonly overload. Public
message call signatures are unchanged. Rebuild generated output to obtain the optimization.

## Plain TypeScript compiler integration

Use `@antepodeum/linguini-tsc` to build a TypeScript application with generated JavaScript and
declarations while keeping `allowJs` disabled. The `linguini-tsc` executable supports project
builds, incremental solution builds (`--build`), and both watch modes through TypeScript's public
APIs. It reads `extends` and project references and uses the native CLI for all generated files
and source maps. Update the native CLI alongside the adapter to obtain declaration-output support.

The adapter delivers generated types beside application declarations, including separate
`declarationDir` output, and runtime beside emitted application JavaScript. `--noEmit` prepares
source types; `--emitDeclarationOnly` emits types without a new runtime. Message-only edits update
runtime even when TypeScript skips compilation. Existing runtime output survives a failed native
rebuild. Programmatic watch callers must call the returned `close()` when finished.

See the [TypeScript build integration guide](./typescript.md) for setup and output rules.

## Explicit Vite framework adapter

Standalone Svelte projects import the Vite plugin from
`@antepodeum/linguini-svelte/vite`; SvelteKit projects use
`@antepodeum/linguini-sveltekit/vite`. The main package entry is framework-neutral. It accepts an
explicit `LinguiniFrameworkAdapter` for generated entry/export names, runtime descriptors,
source extensions, script scopes, static links, and locale-loader cleanup. Other integrations
can reuse Vite's graph, watch, validation, and message transform without importing Svelte.
The Svelte adapter preserves existing generated paths and manifest descriptors; this step
requires only the import-path update in application Vite configuration.

## Integration ownership

Web URL/resolver/persistence behavior lives in `@antepodeum/linguini-web`. Svelte state and
message-provider behavior lives in `@antepodeum/linguini-svelte`; SvelteKit hooks and navigation
live in `@antepodeum/linguini-sveltekit`. These packages have one TypeScript implementation each.
`linguini-codegen-svelte` emits project-specific import/type bindings, while
`linguini-codegen-vite` owns physical message leaves, provenance, and the framework-neutral
manifest. The base ECMAScript backend depends on neither framework nor web runtime code.

## Canonical JavaScript generation

The ECMAScript backend emits one JavaScript implementation and optional adjacent `.d.ts`
declarations. TypeScript consumption remains supported, including strict NodeNext and Bundler
projects with `allowJs = false`. The generated runtime files now end in `.js`, with `.js.map`
companions; Svelte rune helpers use `.svelte.js`. Update explicit generated `.ts` imports to `.js`.
Keep application TypeScript source files and ordinary TypeScript compiler settings as before.

For Rust integrations, replace paired `compile_typescript_*` / `compile_javascript_*` functions
with `compile_*`. Replace `generate_typescript_project_files` with `generate_project_files`, and
`generate_javascript_schema_files` with `generate_schema_files`. Backend model names now use the
`Ecma` prefix: `EcmaOptions`, `EcmaProjectOptions`, `ValidatedEcmaProject`, `EcmaLocaleModule`, and
`CompiledEcma*Module`. Use `EcmaGeneratedFile` directly; the TypeScript alias is removed.
`compile_ecmascript_project_files(project, sources)` no longer accepts a runtime target.
`EcmaModuleKind::{JavaScript, Declaration}` distinguishes JS grammar from `.d.ts` grammar and
cannot request a TypeScript implementation. Custom plural-rule emission also produces JavaScript.

The Vite integration consumes these JS artifacts directly. Module paths recorded in the bundler
manifest and helper descriptors now refer to `.js` files. Rebuild generated output before using
an updated plugin; existing owned `.ts` runtime files are removed by the normal build transaction.
A plain `tsc` application build must deliver generated JS to its runtime output; integrations can
reuse `linguini build --emit-dir DIRECTORY` without enabling `allowJs`.

The site output archive has been regenerated from the canonical JavaScript backend. Its
Node-only restore path now delivers `.js` runtime and `.d.ts` companions, including the new
SvelteKit host bridges. Rebuild an application archive after updating the native generator;
restoring an archive produced by the retired TypeScript emitter does not migrate its contents.

Repeated native builds preserve unchanged owned files instead of replacing them. Message-only
changes retain declaration timestamps, allowing TypeScript incremental and watch builds to reuse
their type graph. Existing ownership, stale-file removal, and rollback rules still apply.

Native build integrations can prepare dependent types with repeatable `--emit-declaration-dir`
arguments. These output roots receive declarations and declaration maps while retaining any
previously owned runtime. Use `--emit-dir` after successful application compilation to update the
runtime as well. This supports project references and separate TypeScript declaration output
directories without copying or rewriting generated source in the JavaScript integration.

The web runtime helpers now live in build-time templates, including URL and request operations,
locale resolvers, route matching, bounded DOM traversal, and server cookie/switch handling.
There is no paired TS implementation body for these helpers. Declarations retain their public
contracts and explicit runtime `.js` imports.

Physical message exports, namespace aliases, value re-exports, and imported JSDoc typedefs now
use shared module templates. Standalone and bundler message compilation reuse the same constant
and function templates as project generation; their generated bytes and source maps are preserved.

Declaration exports, positional/named overloads, callable types, and JSDoc blocks also render
through shared templates. Locale declarations and the schema-owned message contract reuse one
namespace-type renderer, preserving nested message and group source locations. No application
configuration or generated API changes are required for this extraction.

Function and form dispatch now uses direct branches and returns. Inline functions are lifted to
static helpers with explicit captured arguments; generated calls preserve eager inputs and lazy
branch results without building branch objects or callback tables. Unnamed selectors receive
readable names derived from their types, with deterministic collision handling. Internal form
bindings now retain source names (`__lgl_form_Fruit`); safe qualified names retain their components
(`__lgl_name_shop$Money`). Reserved internal prefixes and ambiguous names remain encoded. Rebuild
all generated artifacts together; applications should use the public message/type API rather than
importing private generated bindings. Argument-key tables now use the same scoped naming rules
as forms; a qualified table such as
`__lgl___lgl_name_account$personalized_keys` becomes `__lgl_keys_account$personalized`.
Only private binding names change; public exports, declarations, source maps, normalization,
and call evaluation remain unchanged. Argument normalization retains a JSDoc tuple assertion
from the same call signature, and locale globals import their schema typedefs. This preserves
strict checking of implementation bodies as well as declaration-only consumer checking. Static
form attributes retain literal types so their enum values can be passed directly to generated
functions. These annotations add no runtime validation or allocation.

Formatter calls now pass only the input value. The compiler resolves each used request into
specialized number/currency/date code and selected helpers. Currency symbols come from pinned
CLDR data instead of host `Intl.NumberFormat`; explicit formatter options produce separate,
readable helper bindings. These locale-runtime helpers are generated implementation details;
applications should continue calling the schema's generated message API. Numeric scaling,
scientific notation, significant rounding, and significant padding now have independent helper
selection. A request for one operation neither emits unrelated helpers nor relies on another
operation being enabled. Fraction precision, rounding quantum/halfway threshold, scale shift,
and significant-digit widths are now compiled into selected, deduplicated numeric helpers.
Their runtime APIs take actual values rather than static pattern options. Fixed-width currency
fractions skip optional-digit trimming, and date padding helpers follow the compiled fields.
Input validation and exact arithmetic remain covered by executed corpora, including a rational
rounding oracle, long fractions, increments, carries, negative zero, and bigint inputs.
Input-derived significant precision still checks the decimal expansion limit before padding;
the removal of static option validation does not remove that runtime input bound.
Plural rules now compile to direct numeric comparisons, shared guards,
and one calculation per required remainder. Operand calculations are emitted at the first rule
that needs them; no per-call operand object, range array, or generic relation interpreter remains.
Visible-digit counts and compact exponents use bounded numbers; integer and fractional values
retain exact bigint arithmetic. Shared decision and requirement plans live in `linguini-codegen`.
Input validation, compact exponents, and explicit plural-category inputs retain their behavior.
The formatter regression corpus is kept in its own backend test module, including generated-code
execution and checks for omitted helper families.

Namespace modules now import only the global values directly used by their messages; static
namespaces and re-export-only barrels omit global runtime imports. Rebuild generated files
together. Public message APIs remain unchanged. Direct semantic dependency planning now lives
in `linguini-codegen` alongside the transitive closure planner; the ECMAScript symbol-kind name
remains a compatible re-export. Both planners preserve simultaneous inline-input scope, and
alias-typed form calls use the resolved runtime form binding instead of an undefined alias binding.

Switch-route generation now compiles the configured `{locale}` marker into a fixed prefix/suffix
and extraction bound. Generated requests no longer search or split the route template, and routes
without a suffix omit its check. Host base paths remain dynamic. Redirect status, percent decoding,
same-origin return validation, and selected cookie/path transports keep their existing contracts.

Accept-Language selection now scans preferences without per-locale filtering/sorting or candidate
objects. Most-specific exclusions, quality, input order, base preference, and configured locale
order retain their behavior. Navigator fallback and the public resolver signature are unchanged.

Number/currency grouping now emits a direct string loop specialized for the compiled group
widths. It no longer builds/reverses/joins a temporary group array. Public formatter inputs and
outputs are unchanged, including primary/secondary grouping, padding, exact long values, and
negative zero.

Date generation now resolves locale/context/width-specific symbols from pinned CLDR data:
formatting and standalone month names, narrow/short name widths, AM/PM, and Gregorian era labels.
Rebuild generated code; Arabic and Hindi time results now use `م` and `pm` instead of the incorrect
hard-coded `PM`. Identical selected name tables within one formatter share a binding. The shared
Rust `date_field_symbols` plan and `CalendarFormatData::symbol` lookup contain no target syntax;
JavaScript receives selected literals/tables without runtime CLDR symbol-path lookup.

The intermediate APIs and paired-output parity receipts below describe earlier remediation
stages; this section supersedes their runtime target names and `.ts` output paths.

## 0.1.0-alpha.4 remediation baseline

Unquoted locale text now selects its mode at the start of the value. A later `"`
is literal text, including an unmatched quote; it no longer starts a quoted fragment,
silently disappears, or raises an unterminated-string error. The same rule applies
after a placeholder and inside named/inline function branches. If existing source
used a later quoted fragment for decoding, begin the value with a quoted fragment
instead, for example `"Hello "{name}"!"`. Explicitly quoted strings retain their
escapes; `"""` and `raw"""` retain their dedented and byte-preserving block semantics.
The reference now documents all four modes, brace handling, and invalid escapes.

- CLI commands now resolve project roots consistently across filesystem aliases on
  Windows and macOS. Diagnostic paths, file-scoped fixes, formatting reports, and
  generated source maps use the same root identity. Formatting reports portable
  project-relative paths; symlinks below the root remain rejected for modification.
- Schema dispatch, argument normalization, and numeric/date/digit formatters now
  share runtime bodies across TypeScript and checked JavaScript. Regenerated bytes
  and public helper contracts are unchanged; obsolete helper templates were removed.
- Project locale and index entries now share locale resolution, lazy preparation,
  and provider bodies across both targets. Import paths, declarations, emitted bytes,
  and source-map contracts remain unchanged.
- Plural operands now use that same target-aware runtime renderer; the final paired
  TypeScript/JavaScript runtime templates were removed. Existing decimal bounds,
  compact-exponent behavior, public APIs, and generated bytes are unchanged.
- Stateful Vite adapters using `locale_loading = "dynamic"` now default to an ES2022
  build target when `build.target` is unset, so initial-locale preparation also
  builds on Vite 5 and 6. Explicit targets remain authoritative and must support
  top-level await; eager locale loading keeps Vite's default target.
- Regenerate TypeScript output after upgrading. Generated `normalizeLocale` now uses the complete
  project-specific lookup lowered from Linguini's pinned CLDR graph instead of combining a
  JavaScript truncation heuristic with exceptional overrides.
- Rust integrations using `linguini-syntax` must replace direct `SchemaFile::declarations`,
  `SchemaFile::span`, `LocaleFile::declarations`, and `LocaleFile::span` field access with the
  `declarations()` and `span()` accessors. Parsed file roots are immutable outside the crate;
  transform source and reparse it instead of mutating an AST in place.
- Configuration tooling can consume the committed JSON Schema through
  `linguini_config::CONFIG_SCHEMA_JSON`; keep editor integrations pinned to the matching Linguini
  release.
- Codegen integrations that inspect public parameter types can use `TypeModel` plus its TypeScript
  and JSDoc renderers instead of recreating source-type mappings.
- Codegen integrations can use `generate_javascript_schema_files` for the schema-owned checked
  JavaScript/JSDoc runtime and its declaration companions. The returned `EcmaGeneratedFile` is the
  common record also exposed as the existing `TypeScriptGeneratedFile` alias.
- Rust codegen integrations constructing `TypeScriptOptions` must replace `plural_source` with
  `plural_rules`. Pass canonical `linguini_cldr::PluralRules` for an embedded implementation, or
  leave it unset and provide `plural_import`; codegen now renders target-safe local and exported
  functions itself instead of rewriting caller-provided TypeScript text.
- Bundler integrations can compile each validated locale runtime artifact directly through the
  TypeScript or checked-JavaScript artifact compiler. These APIs preserve exact helper demand,
  ordered source identity, and target-owned source-map paths.
- Rust codegen integrations can enumerate `project_artifacts` and compile each `locale` or `index`
  entry with `compile_typescript_project_artifact_module` or
  `compile_javascript_project_artifact_module`. Existing generated TypeScript module paths and
  declarations are unchanged; the new checked-JavaScript entry artifacts use `.js` imports and
  return adjacent source maps.
- Rust codegen integrations that need the generated web route-exclusion helper can call
  `compile_typescript_web_routes_module` or `compile_javascript_web_routes_module`. Existing
  generated TypeScript route bytes and declarations are unchanged; the JavaScript artifact uses
  checked JSDoc and an adjacent source map.
- Rust codegen integrations that need the generated path-locale resolver can call
  `compile_typescript_web_path_module` or `compile_javascript_web_path_module`. Existing
  generated TypeScript path bytes and declarations are unchanged; the JavaScript artifact uses
  checked JSDoc and an adjacent source map.
- Rust codegen integrations that need the generated cookie-locale resolver can call
  `compile_typescript_web_cookie_module` or `compile_javascript_web_cookie_module`. Existing
  generated TypeScript cookie bytes and declarations are unchanged; the JavaScript artifact uses
  checked JSDoc and an adjacent source map.
- Rust codegen integrations that need the generated local-storage resolver can call
  `compile_typescript_web_local_storage_module` or `compile_javascript_web_local_storage_module`.
  Existing generated TypeScript local-storage bytes and declarations are unchanged; the JavaScript
  artifact uses checked JSDoc and an adjacent source map.
- Rust codegen integrations that need the generated Accept-Language resolver can call
  `compile_typescript_web_accept_language_module` or
  `compile_javascript_web_accept_language_module`. Existing generated TypeScript bytes and
  declarations are unchanged; the JavaScript artifact uses checked JSDoc and an adjacent source
  map.
- Rust codegen integrations that need the selected transformed-link helper can call
  `compile_typescript_web_link_transform_module` or
  `compile_javascript_web_link_transform_module`. Existing generated TypeScript bytes and
  declarations are unchanged; the JavaScript artifact uses checked JSDoc, `.js` imports, and an
  adjacent source map.
- Rust consumers matching `EcmaImportBindings` exhaustively must handle the new `NamedWithTypes`
  variant. `EcmaImport::named_with_types` preserves mixed value/type imports in TypeScript and
  emits only value bindings in JavaScript; type-only dependencies create no runtime imports.
- Rust integrations generating Svelte locale state can use
  `compile_typescript_svelte_locale_module` or `compile_javascript_svelte_locale_module` with
  `SvelteLocaleMode::{Context, Standalone, SvelteKit}`. TypeScript bytes and declarations remain
  unchanged. JavaScript retains `.svelte.js` runes for framework compilation, checked JSDoc,
  and an adjacent source map; it is not a plain-JavaScript replacement for Svelte compilation.
- Svelte browser-effect codegen callers can use `compile_typescript_svelte_effects_module` or
  `compile_javascript_svelte_effects_module`. The shared emitter groups imports before browser
  initialization; runtime bodies and declaration bytes are unchanged. Unselected capabilities
  are omitted structurally, rather than removed from rendered TypeScript text.
- Lightweight Svelte controls can be compiled with `compile_typescript_svelte_control_module` or
  `compile_javascript_svelte_control_module`. Imports are grouped before standalone browser
  initialization; declaration bytes and transport behavior are unchanged. Pathless policies no
  longer use a post-render removal pass for navigation imports and URL reads.
- Svelte message-facade callers can use `compile_typescript_svelte_module` or
  `compile_javascript_svelte_module`. Pass `false` for context-only state or `true` for web
  controls. Runtime bodies and declarations remain unchanged; context imports are normalized
  to the shared emitter's layout, and JavaScript uses explicit `.js` module edges.
- Server cookie and locale-switch route integrations can use the corresponding
  `compile_{typescript,javascript}_web_{server_cookie,switch_route}_module` functions. Cookie and
  declaration bytes are unchanged; empty template-placeholder lines in switch routes are
  normalized. JavaScript retains checked locale generics and source maps. Absent switch routes
  produce no artifact, and disabled cookie capabilities produce no persistence import.
- Rust integrations using `linguini-schema` must replace direct `SchemaSymbols` and symbol-field
  access with the matching read-only accessors such as `messages()`, `parameters()`, and `span()`.
  Symbol tables can no longer be mutated after validated construction.
- Rust integrations using `linguini-config` must replace direct model-field access with read-only
  accessors such as `project()`, `paths()`, `typescript()`, and `features()`. `parse_config` is now
  the only public constructor for a complete validated configuration.
- Generated documentation now preserves consecutive lines and blank paragraphs in one safe JSDoc
  block across runtime and declaration output. Typed backend tags are generated separately from
  canonical source prose.
- Generated output is ESM-only. Remove `targets.ts.module` and CommonJS output assumptions.
- `@date` now accepts the additive `time_style` option. Existing date-only output is unchanged;
  regenerate output to use CLDR time widths or combined date-time patterns.
- Regenerate locale runtimes that use 12-hour CLDR time patterns. Day-period expressions are now
  parenthesized before concatenation, so the generated value retains its hour and minute text
  instead of collapsing to only `AM` or `PM`.
- SvelteKit web policy belongs in application TypeScript modules and hooks; migrate old
  `[web.*]` tables using [Web and SvelteKit](./web-sveltekit.md).
- Parameterless generated messages are string values. Parameterized messages accept positional
  arguments or one named object with every declared property.
- Bundler-mode `l.*` access must be statically resolvable unless each finite dynamic path is listed
  under `[targets.ts.bundler.dynamic]` with `mode = "bundle"`.
- Dynamic locale loading prepares a locale before publishing it. Direct framework-agnostic callers
  must await `prepareLinguini(locale)` before synchronous non-base access.

The closed project web facade now has `compile_typescript_web_module` and
`compile_javascript_web_module` entry points. Both use the same selected physical locale-source
modules, URL/request policy, public type model, and target-owned source maps. The superseded
TypeScript web template is retained only as a frozen test oracle. Checked target types, closed
source matrices, URL/link/cookie safety, and concurrent request preparation run in quick/full gates
through `tests/fixtures/golden/snapshots/js-web`.

SvelteKit server adapters now use one target-aware builder through
`compile_typescript_sveltekit_module`, `compile_javascript_sveltekit_module`,
`compile_typescript_sveltekit_control_module`, and `compile_javascript_sveltekit_control_module`.
The generated message and metadata adapters preserve request-local contexts, streaming page
transforms, reroutes, canonical redirects, safe locale-switch routes, and selected cookie writes.
Their superseded implementation templates are frozen test oracles only. Installed-SvelteKit type
checks and executable TS/JS/frozen-runtime parity are registered quick/full gates under
`tests/fixtures/golden/snapshots/js-sveltekit`.

`compile_ecmascript_project_files(project, target, sources)` now enumerates a complete native
TypeScript or JavaScript project through the shared module builders. It emits an owned map beside
every implementation; declarations follow the validated declaration setting. Source paths are
relative to the output root and are rebased to each map directory. Missing/duplicate source
identities and output collisions fail before filesystem writes. The established
`generate_typescript_project_files` API renders source-only output from that same artifact plan.
The CLI now consumes mapped project artifacts through its existing atomic ownership transaction.
JavaScript namespace typedefs retain the full message API even when declarations are disabled.
The integrated fixture is `tests/fixtures/golden/snapshots/js-complete-project`.

Locale declaration barrels now re-export their imported namespace bindings directly. This fixes
strict declaration checking that previously reported an import/local declaration collision and a
self-referential namespace type; the public export names and value types remain unchanged.

Generated schema types now include `shared.d.ts.map` and `messages.d.ts.map` companions.
TypeScript and JavaScript schema maps identify the original enum, alias, message, and group
declarations, so editor navigation can return to the Linguini source.

The compiled CLDR data API now exposes Gregorian, Buddhist, ROC, and Japanese calendar
patterns and contextual symbols through `compiled_calendar_formatting`, along with shared
day-period, week, time-zone display, and Japanese era data. Calendar data follows the same
canonical locale fallback policy as the existing formatting APIs.

Internal schema dispatch code now renders from one JS/TS template with shared type macros.
This changes its source ownership while preserving generated TypeScript, JavaScript,
declaration, and source-map bytes. Template rendering runs inside the Rust compiler during
project generation; consumers need no additional build tools.

The Rust code-generation crate is now `linguini-codegen-ecmascript`, covering both JavaScript and
TypeScript. Update Cargo dependencies and use the `linguini_codegen_ecmascript` import path. The
existing target-specific API names and generated module contracts are unchanged.

Backend-independent type models, formatter/plural requirement analysis, and transitive message
dependency selection now live in `linguini-codegen`. The ECMAScript backend depends on this
shared crate and re-exports `TypeModel`; its renderers, target bindings, and templates stay in
`linguini-codegen-ecmascript`.

Numeric, date, digit, plural-operand, and locale-provider helper bodies now render from shared
JS/TS templates during builds. This ownership change preserves their generated source and map
bytes; no generated runtime implementation files are checked into production sources.

Generated declaration imports now use explicit `.js` module paths. Strict TypeScript consumers
can resolve JavaScript output through its adjacent `.d.ts` files with `allowJs = false`,
including NodeNext projects. Bundler resolution remains supported.

`linguini build --emit-dir DIRECTORY` can additionally generate into build output directories
without editing `linguini.toml`. The option is repeatable; duplicate directories are coalesced,
and overlapping or unsafe locations fail validation before generated trees are written.

Compiled date fields `b` and `B` now use the pinned fixed/flexible day-period rules instead of
substituting AM/PM for every value. For example, English `B` at 09:30 emits “in the morning”, and
`b` can emit “noon” or “midnight”. Exact points follow the displayed time precision, with midnight
at the beginning of the day. Localized labels, missing-label fallbacks, intervals, and widths
resolve during generation; no CLDR rule tables or interpreter are added to runtime output.

Gregorian date field `y` now formats the year within its era: astronomical year zero displays
as 1 BCE, and negative years count backward within BCE. Widths of three or more digits now
zero-pad to the requested minimum; `yy` keeps its two-digit truncation. For example, `yyyy G`
formats year 1 as `0001 AD`, and year zero as `0001 BC`. Year and era formatting share one UTC
year read, and date requests containing only a padded year select the padding helper correctly.

Vite and TypeScript adapters now depend on `@antepodeum/linguini-project` for shared configuration
loading and ECMAScript layout validation. Existing valid TOML paths and adapter entry points are
preserved; malformed layout fields now fail consistently before native generation in both hosts.
No application change is required. Source checkouts resolve the shared package through local pnpm
overrides; published adapters depend on its pinned release version. Publish the shared project
package before the adapters for a new release. The module-configuration migration above removes `[web]`.

VS Code highlighting now comes entirely from the shared Rust language pipeline. The Linguini
TextMate grammars and their regex dependencies have been removed. Keep semantic highlighting
and the matching native server enabled; existing theme scope mappings still apply. Markdown
Linguini fences use the same native token classification. Quotes no longer auto-close in locale
text because they can be literal characters; explicit quote surrounding is retained.

The CLI adds `highlight <file.lgs|file.lgl>` for versioned JSON token output, also available through
the `linguini/highlight` LSP request. Site examples use this output at refresh time and check its
source hash during ordinary builds. Generated highlight HTML lives only in `.svelte-kit`.

Highlight and LSP source positions now recognize CR-only line endings as well as LF and CRLF.
Multiline token spans exclude each line terminator, and UTF-16 offsets round-trip in all three forms.

Generated messages now use number, currency, and date formatter results directly, omitting
redundant outer `String` calls. Input coercion, formatter order, exceptions, aliases, and the
public string result contract are preserved. No application changes are required.

Vite now folds missing translations into the selected locale payload at message granularity.
Sparse locales no longer trigger an additional base-locale payload request. Generated message
dependencies, including shared functions and formatter helpers, stay within their owning
application/locale payload. Loader cleanup, HMR, and preload-before-switch remain supported.

All npm package source directories now live under `packages/`. Move source-checkout paths from
`plugins/vite` and `plugins/tsc` to `packages/vite` and `packages/tsc`; update local `file:`
dependencies and scripts accordingly. Published package names and adapter entry points are
unchanged by this directory move. Runtime adapter packages are maintained in TypeScript and
publish built JavaScript and declarations.

The new `@antepodeum/linguini-web` and `@antepodeum/linguini-sveltekit` packages contain reusable
TypeScript runtime adapters. Their options are application runtime values; they are not native
code-generation inputs. Runtime locale sources are explicit capabilities (`pathLocale()`,
`cookieLocale()`, `requestLanguage()`, `localStorageLocale()`). The SvelteKit server entry
creates hooks and serialized request metadata without importing a generated message catalog.
The site and native build integration use these runtime boundaries.
