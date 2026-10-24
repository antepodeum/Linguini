# Migration notes

This file records required application changes when a public Linguini contract changes. The CI
contract gate requires source changes to public configuration, CLI, generated API, Vite, LSP, or
editor boundaries to update this file, public documentation, and executable tests together.

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
Standalone Svelte uses `@antepodeum/linguini-vite/svelte`. These adapters share the Vite core;
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

## Standalone weekdays in full dates

Rebuild generated output to correct Finnish and Inari Sami full-date formatting. Their pinned
CLDR patterns select standalone weekdays (`c`); these now use the appropriate weekday names
instead of the old `UTC` placeholder. Named formatting/standalone contexts and short/narrow
widths remain distinct. The public date request and JavaScript/declaration APIs are unchanged.

## Runtime link and request snapshots

Use `localizeLinkHref(href, locale, attributes, input)` for combined link validation/localization.
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
`@antepodeum/linguini-vite/svelte`; SvelteKit projects use
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
