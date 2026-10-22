# Migration notes

This file records required application changes when a public Linguini contract changes. The CI
contract gate requires source changes to public configuration, CLI, generated API, Vite, LSP, or
editor boundaries to update this file, public documentation, and executable tests together.

## Plain TypeScript compiler integration

Use `@antepod/linguini-tsc` to build a TypeScript application with generated JavaScript and
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

Svelte and SvelteKit projects now import the Vite plugin from
`@antepod/linguini-vite/svelte`. The main package entry is framework-neutral. It accepts an
explicit `LinguiniFrameworkAdapter` for generated entry/export names, runtime descriptors,
source extensions, script scopes, static links, and locale-loader cleanup. Other integrations
can reuse Vite's graph, watch, validation, and message transform without importing Svelte.
The Svelte adapter preserves existing generated paths and manifest descriptors; this step
requires only the import-path update in application Vite configuration.

## Independent integration crates

Rust callers now import Svelte generation from `linguini-codegen-svelte` and SvelteKit generation
from `linguini-codegen-sveltekit`. Their implementations and templates moved out of the ECMAScript
crate. `EcmaProjectOptions::framework` and `EcmaFramework` are removed: compose base project output
with the selected integration's `generate_project_files` function. Its source-map argument selects
source-only output or code with owned map companions. Generic web artifacts remain owned by the
ECMAScript backend and follow its selected `web` policy.

SvelteKit reuses Svelte generation. The native Vite artifact/manifest plan lives in
`linguini-codegen-vite`; it accepts explicit application and runtime-helper plans without depending
on either framework crate. The CLI composes these owners, and packaged CLI workspaces must include
all three new crates. Output paths and the existing generated consumer contracts are preserved.

Svelte generation now takes explicit `SvelteHost` bindings instead of a SvelteKit boolean.
Replace `SvelteProjectOptions::request_routing` with `host` (use `SvelteHost::default()` for a
standalone app). The low-level effects/control compilers also take `&SvelteHost`.
Replace `SvelteLocaleMode` with `SvelteLocaleOptions { initialize, request_locale }` when compiling
locale state directly. Host bindings identify named ESM imports, including optional navigation
state types; they do not contain executable source strings.

SvelteKit composition supplies its own request-locale and navigation modules. Rebuild owned
output to include `sveltekit-locale.js` and, for path-based navigation, `sveltekit-navigation.js`
with their map/declaration companions. Request locale is read from SvelteKit page context when a
message consumer asks for it. Pathless policies emit no navigation runtime or `$app/navigation`
dependency. Application-facing locale-switch options and `App.PageState` typing are preserved.

Svelte effects now omit browser readers when no locale sources are selected and initialize from
the base locale directly. Runtime-link imports, effect objects, and HMR disposal are emitted only
for runtime-link mode. Refresh/destroy exports remain callable no-ops in manual/transform modes.
Controls without path/cookie capabilities no longer allocate an unused options object; defaults
follow the selected transport and host navigation capabilities. Optional switch arguments,
callable arity, denied-persistence handling, and navigation-state forwarding are preserved.

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
importing private generated bindings. Argument normalization retains a JSDoc tuple assertion
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
- Vite applications using `locale_loading = "dynamic"` now default to an ES2022
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
- SvelteKit web policy uses nested `[web.*]` tables. Replace legacy flat routing, source, cookie,
  local-storage, link, exclusion, and switch-route fields with the mappings in
  [Web and SvelteKit](./web-sveltekit.md).
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

Web generation now belongs to `linguini-codegen-ecmascript-web`. Rust integrations must move
`EcmaWebOptions`, its policy types, `render_web_options`, and `compile_web_*` imports to
`linguini_codegen_ecmascript_web`. `WebCodegenError` owns web-policy validation failures.
Remove `EcmaProjectOptions.web`; explicitly append the web crate's
`generate_project_files(&WebProjectOptions { web, declaration }, source_maps)` output alongside
language artifacts, then append the selected framework artifacts. SvelteKit composes Svelte and
web capabilities; the base ECMAScript crate depends on neither. Generated module paths, runtime
contracts, declarations, and source-map contents are preserved. The CLI handles this composition;
this ownership change does not yet remove legacy `[web]` project configuration.

Web declaration and JSDoc type definitions now share the static model in
`linguini-codegen-ecmascript-web/templates/web/types.jinja`. Rust no longer contains web API type
headers, method signatures, or member type expressions. Generated source, declaration, and map
bytes remain unchanged; no consumer migration is required for this template extraction.

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
