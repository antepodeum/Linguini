# Linguini audit remediation checklist

Source of truth: [`Linguini_Audit_EN_FINAL_WITH_DOCS.md`](./Linguini_Audit_EN_FINAL_WITH_DOCS.md).

Status legend:

- `[x]` implemented and verified locally;
- `[-]` partially implemented, but an integration, contract, or verification gap remains;
- `[ ]` not yet completed.

This file is intentionally stricter than a change log. An item stays partial until the
production path uses the fix and its relevant tests pass. Earlier verification receipts describe
only their named source revisions; they do not certify the in-progress architecture changes below.
The module/configuration requirements in MOD-A1–A8 supersede the earlier TOML web-policy and
paired-runtime design. Preserve the tested behavior while replacing those ownership boundaries.

## Evidence already committed

- `7fda0b5` — safe project output paths.
- `bcfdf88` — form branches, multi-key branch preservation, alias-cycle guard.
- `36b45b2` — formatter identity and shared primitive-type registry.
- `e2bcf62` — hermetic, offline CLDR data generation.
- `522bb8a`, `2399d15` — semantic-preserving formatting and protected raw blocks.
- `5791678`, `a76fdc2` — serialized Vite rebuilds and real Vite builds.
- `90a0f1a` — typed nested web-policy configuration and validation.
- `fe9f978` — reference/call identity, declaration kinds, multiline modes, source IDs,
  recursive groups, and syntax invariants.
- `27e78b7` — atomic and contained CLI project operations.
- `b362a79` — native npm packages, target VSIX packaging, and protocol handshake.
- `3ce2c7e` — typed semantic analysis and a validated-IR capability.
- `9c64ec4` — namespace-aware indexed LSP operations.
- `9ab0c88` — stable project source IDs, canonical namespaces, merge provenance, cross-file
  schema checks, duplicate locale rejection, and a sealed validated codegen boundary.
- `2ea35a4` — typed form-map dispatch, zero-argument calls, injective TypeScript identifiers,
  complete string escaping, and strict project-input validation.
- `bead89b` — exact CLDR plural operand parsing/evaluation with compact exponents,
  overflow checks, structured offsets, and deterministic fallback categories.
- `451aade` — pinned MSRV/stable CI, platform and release matrices, dependency policy,
  publishable-crate packaging, native artifacts, and synchronized release versions.
- `08a1ee1` — public parameterless message values across grouped and top-level APIs.
- `8de3e20` — validated exhaustive dispatch and visible missing-branch failures.
- `d547858` — same-origin web localization, exact locale segments, safe exclusion matching,
  and a consistent URL error contract.
- `def8e52` — warning promotion for both CLI checks and pre-write builds.
- `b3b93b6` — closed generated locale-source contracts, configuration-aligned defaults,
  weighted `Accept-Language` negotiation, strict SvelteKit headers, and safe cookie decoding.
- `220b8a3` — lexer-derived TextMate and language-configuration golden tests.
- `d82d239` — append-only locale cookie persistence through supported response sinks.
- `615b24d` — portable, injective namespace filenames plus locale-component and
  case-insensitive output-collision validation.
- `8f62f82`, `8ea9901` — absolute-path preservation before validation plus executable
  project-path normalization and filesystem-namespace collision contracts.
- `6604bac` — explicit JSON and SARIF 2.1.0 CLI diagnostics with source ranges,
  related locations, stable rule IDs, severities, and quick fixes.
- `cbd4cb3` — scoped public analyzer/type guarantees, exact lint severities,
  explicit unused-message limits, and an executable CLI documentation contract.
- `bd80239` — removed the generated site tree from Git tracking and made CI regenerate it
  before runtime, type, and production-build verification.
- `7e8fa50`, `a6e5f7b`, `7cbb209`, `27aa665` — namespace-aware dependency validation,
  enum-valued form properties, locale-symbol bindings, and collision-safe public type aliases.
- `a449ce0` — removed the unreachable generated redirect-status override and fixed canonical
  redirects to the runtime contract's temporary 307 response.
- `b58b66c` — confined generated request state to the typed `locals.linguini` namespace and
  removed collision-prone generic SvelteKit locals.
- `110fce9` — added collision-free generated hook/load aliases plus explicit SvelteKit
  `sequence`, reroute-priority, and load-merge composition contracts.
- `858d5cf` — host-time-zone-independent generated date formatting with strict ISO
  coercion and visible invalid-date failures.
- `16d854e`, `f93e516` — pinned CLDR aliases, parent locales, likely subtags, BCP 47
  extension aliases, and one canonical locale-resolution service consumed by CLI and codegen.
- `581fb85` — removed regex HTML rewriting and response buffering, centralized anchor
  skip policy, and added bounded per-instance browser observers with HMR disposal.
- `c98a2ac`, `dd275c6` — unambiguous filesystem/group namespace documentation,
  migration rules, canonical-path collision semantics, and a project-build contract test.
- `6089d95` — delegated web locale fallback to the generated CLDR resolver and
  delegated trailing-slash canonicalization to SvelteKit page options.
- `b5b2a69` — added a SHA-pinned workspace public-API compatibility job against
  each pull request base or pre-push revision.
- `b5f27af`, `18f9618` — modeled significant digits, rounding increments, scientific
  exponents, percent/per-mille scaling, and all padding positions from CLDR number patterns,
  then lowered those semantics into the exact generated TypeScript decimal runtime.
- `f378a5f` — formatter parsing now reuses the validated parser's lossless token stream,
  with a regression test proving one lexer invocation per source.
- `1dd4223` — removed unsupported inline `fn` syntax from the normative reference and
  locked the parser rejection contract with an executable test.
- `10fc9d7` — implemented value-based inline `fn` plus selector-first/payload-last named
  functions across syntax, IR, analyzer, formatter, CLI preview, TypeScript codegen, LSP,
  TextMate grammar, docs, and generated-site runtime; also added bounded exact numeric/plural
  evaluation, CLDR currency minor units and rounding, and visible invalid/missing-data failures.
- `3ae3a38` — added opt-in, bounded application-source discovery plus conservative static and
  dynamic-prefix message usage analysis to CLI check/build, JSON, SARIF, and warning policy.
- `4e648f4` — added explicit group metadata with docs/spans through IR lowering, qualification,
  validation, project merge/fallback, and bounded codegen projections.
- `f2faa80` — added a validated, deterministic per-message semantic dependency closure with
  transitive symbol selection and source-ID metadata for later ESM/HMR consumers.
- `6caec55` — added the public source-mapped single-message TypeScript ESM compiler with exact
  dependency/helper emission, callable leaf exports, and deterministic source validation.
- `cae571a` — added the schema-owned recursive `LinguiniMessages` namespace with value/callable
  leaves, group/leaf JSDoc, safe keys, effective filtering, and strict TypeScript verification.
- `b7758fd` — added deterministic source-aware application references with exact leaf spans,
  value/call identity, decoded paths, and transform-safe binding provenance.
- `591c532` — added deterministic physical per-message modules and source maps plus the v1
  bundler manifest through the transactional CLI output path.
- `bed72e0` — split canonical locale metadata and reactive Svelte locale state from the eager
  project runtime so per-message wrappers can resolve locale without importing every message.
- `18792cf` — added opt-in typed bundler source discovery and a deterministic application
  manifest with exact hashes, byte spans, binding provenance, arity, and unresolved references.
- `1e5ae83` — added strict-module import parsing, stable binding identities, exact removal spans,
  and binding-wide transform-safety metadata to analyzer and the bundler manifest.
- `1ce930b` — split Svelte browser effects from the eager message runtime, added metadata-only web
  policy, and generated distinct plain-Svelte and SvelteKit reactive locale implementations.
- `1804d1f` — added lightweight Svelte client controls and stable SvelteKit control hooks so
  transformed applications can avoid eager message barrels without breaking public APIs.
- `aa75a3f`, `45a78c1` — added Vite bundler transforms and exact virtual message modules, then
  hardened raw-source/resolver identity, scoped alias allocation, effects retention, and graph tests.
- `a6d13fc` — added manifest-delta HMR with exact source-to-message, per-locale physical-module,
  and application invalidation plus add/remove browser propagation.
- `a4f9f57` — fixed exact imported-message spans across Svelte runes, nested same-name markup,
  TypeScript `as const`, destructuring property keys, and function-scoped `var` bindings.
- `de3cf0e` — enabled bundler-native transforms on the real SvelteKit site and added a production
  graph gate proving the eager message provider and locale barrel are absent.
- `df8a13d` — added an offline, dependency-free root runner for ordered Rust, plugin, package,
  editor, and site verification profiles.
- `a20d057` — added a strict-by-default dynamic bundler policy with an explicit finite exact-message
  allowlist for the bounded bundle escape hatch.
- `9e578cd` — added source-exact dynamic message references with binding provenance, computed-key
  spans, value/call identity, and conservative classification of ambiguous or mutating uses.
- `49deef3` — separated complete tracked import usage from exact-static import safety so bounded
  dynamic transforms cannot remove imports when any source use is unrepresented.
- `19c3616` — added bounded dynamic-reference records, strict pre-write diagnostics,
  exact finite allowlist validation, and tracked-binding transform eligibility.
- `0c12d74` — added validated Vite rewrites for finite dynamic lookups with frozen
  null-prototype dispatches, exact virtual imports, resolver identity checks, and selective HMR.
- `eaf8962` — made the root site profile assert the built production graph after generation,
  runtime tests, Svelte checks, and the production build.
- `4e80ed0` — added an eager-by-default typed locale-loading policy and manifest metadata for
  opt-in bundler-visible dynamic locale boundaries.
- `71cb928` — added generated locale-loader registration, deterministic disposal, concurrent
  preparation, and preload-before-switch semantics for lightweight Svelte controls.
- `8405d65` — added Vite client locale boundaries with literal dynamic imports,
  current-locale preloading, retry-safe caches, SSR-static modules, and HMR-safe disposal.
- `fa7bccd` — enabled dynamic locale loading on the real site and proved 522 physical locale
  modules stay outside the initial static client graph while SSR remains synchronous.
- `e8be06d` — reconciled public configuration and SvelteKit documentation with the nested
  value/callable API, exact message imports, strict bounded dynamic access, dynamic locale
  loading, and repository-wide local verification profiles.
- `203faa8` — excluded transactional and physical generated output from the Vite development
  watcher while preserving manifest-driven rebuilds, eliminating real-site startup and HMR storms.
- `d68e432` — moved formatter, date, number, currency, and plural implementations into one shared
  runtime per locale, with source-aware HMR and real-site bundle-graph verification.
- `0f8cfc3` — moved reusable variables, forms, functions, and locale enums into exact per-symbol
  ESM modules, collapsed the unreleased bundler manifest to v1, and verified the real site graph.
- `76059bc` — added positional-compatible named-object message overloads from one signature model,
  shared boundary normalization, exact generated types, and real-site type/runtime coverage.
- `5e81c9a`, `193c83f` — lowered the validated locale-switch plan into generated TypeScript,
  shared it across browser and both SvelteKit controls, and locked source/declaration typing parity.
- `53950ef` — sealed parsed schema and locale file roots behind immutable declaration/span
  accessors, with compile-fail coverage and a documented Rust API migration.
- `8b82f7c` — lowered the complete project-relevant pinned CLDR locale-resolution graph into
  generated output and removed the parallel JavaScript truncation heuristic.
- `013c3cd` — sealed `IrModule` declaration vectors behind slice accessors, a duplicate-rejecting
  builder, and atomic validated composition, making cross-file symbol collisions fail at the
  merge boundary instead of the late reference checker.
- `5fa36cd`, `bf5aee6`, `10f75f8` — made locale coverage/type/exhaustiveness analysis consume one
  complete schema source set, fixed duplicate LSP diagnostics for split schema documents, and moved CLI/LSP
  locale analysis, message completion/hover, and CLI fix discovery behind the same owned
  `SchemaDatabase` snapshot as schema symbols/diagnostics.
- `5b41b55`, `adfecb2` — made LSP navigation resolve and collision-check one immutable workspace
  occurrence snapshot and removed the unused parallel `linguini-locale` scope database.
- `86456c6` — moved schema/locale declaration and reference indexing into `linguini-schema`, so
  LSP reference, definition, rename, completion, hover, and semantic tokens consume the shared
  semantic walk instead of maintaining an independent AST traversal.
- `2d78c9f`, `45030b1`, `d77d2f6`, `af9e05d` — moved plural and formatter helpers plus locale
  runtime artifacts onto target-aware TypeScript/checked-JavaScript emission, removed source-text
  rewriting and the direct runtime generator, added strict/source-map/helper-demand gates and a
  pinned-locale behavior corpus, and fixed 12-hour day-period concatenation.

## Numbered findings

### Cross-cutting architecture and product claims

- [x] #1 — Align specification with shipped syntax, linting, typing, and ESM-only scope. The
  frozen v0.1 contract retains implemented inline functions, names all seven emitted lints,
  defines the validated typing/exhaustiveness boundary, and excludes CommonJS/module switching.
- [x] #2 — Enforce validated IR as the only production codegen boundary.
- [x] #3 — Consolidate duplicated semantic walks into one project semantic database. Schema
      symbols, diagnostics, and locale coverage/type/exhaustiveness analysis now share one owned
      source snapshot; the unused locale-scope database is gone, schema/locale navigation semantics
      live in `linguini-schema`, and LSP resolution uses one immutable per-operation index.
- [x] #4 — Implement argument, selector, and reference type checks retained by the spec.
- [x] #5 — Implement retained exhaustiveness and unreachable-pattern checks.
- [x] #6 — Carry stable source identity through syntax, diagnostics, and semantic IR.
- [x] #7 — Use one namespace model for every declaration kind and every pipeline. Filesystem
      namespaces qualify enums, aliases, variables, messages, groups, forms, functions, types,
      references, and provenance through one IR pass; schema and locale tooling resolve the same
      canonical paths while LSP workspace operations enforce document namespace identity.
- [x] #8 — Use one CLDR-aware locale canonicalization and fallback algorithm everywhere.
- [-] #9 — Expand CLDR formatting to the documented production contract. Exact pinned Gregorian
      skeleton IDs now compile to selected fields/literals; closest matching, other calendars,
      and selectable zones remain open.
- [x] #10 — Keep builds hermetic. Rust compilation consumes checked-in, hash-verified CLDR
      artifacts. The site regenerates localization output through the workspace compiler during
      builds; CI uses the same source build. Application-generated JavaScript, declarations,
      source maps, compressed bundles, and editor runtime output remain ignored, and the tracked
      build-artifact gate rejects configured output trees. Curated golden expectations and pinned
      CLDR build inputs remain versioned.
- [x] #11 — Make accepted web strategies exactly match generated runtime capabilities.
- [x] #12 — Hide invalid mutable public model states behind validated constructors. `IrModule`
      declarations use the duplicate-rejecting builder and atomic composition boundary;
      `SchemaSymbols` and every nested symbol expose read-only views over builder-owned data; and
      parsed configuration plus its nested policy models expose immutable accessors instead of
      public fields. Compile-fail contracts reject external mutation.
- [x] #13 — Provide stable diagnostic codes, categories, severities, and source IDs.
- [x] #14 — Execute normative documentation as conformance fixtures. All 103 code blocks are
  registered; Linguini syntax and TOML config fences are enforced; extracted JavaScript,
  TypeScript, HTML, and Svelte examples build, typecheck, and execute where runtime-capable.
- [x] #15 — Add property, platform, generated-code, and real integration coverage. Seeded parser,
      formatter, plural, and path properties; Linux/macOS/Windows jobs; generated runtime/type
      corpora; Vite 5–8, SvelteKit production, and native VSIX integration gates are all defined.
- [x] #16 — Synchronize and automatically verify all component versions.
- [x] #17 — Rewrite public guarantees so documentation states only shipped behavior.

### `linguini-core`

- [x] #18 — Remove placeholder `CRATE_PURPOSE` values from the public API.
- [x] #19 — Preserve unknown formatter names instead of silently erasing identity.
- [x] #20 — Centralize primitive types in one registry.
- [x] #21 — Make sample generation use the canonical primitive registry.
- [x] #22 — Add cross-layer contract tests for core types and formatter identity.

### `linguini-config`

- [x] #23 — Parse real TOML rather than a partial custom dialect.
- [x] #24 — Remove the independent Vite and CLDR pseudo-TOML parsers.
- [x] #25 — Reject unsafe, overlapping, absolute, traversing, and escaping output paths.
- [x] #26 — Prevent discovery symlink cycles with canonical visited-directory tracking.
- [x] #27 — Validate and canonicalize real BCP 47 locale tags, including numeric regions.
- [x] #28 — Diagnose duplicate TOML sections and keys consistently.
- [x] #29 — Reject obsolete duplicate `[paths] cache` configuration.
- [x] #30 — Validate empty values, case-folded locale duplicates, and duplicate strategies.
- [x] #31 — Remove unsupported `globalVariable` configuration.
- [x] #32 — Require `Secure` for `SameSite=None`.
- [x] #33 — Validate web paths, patterns, cookie settings, origins, and durations.
      Web runtime constructors now reject malformed/relative/non-HTTP(S) origins and credentials
      at startup, capturing the origin getter once. Executed cases preserve valid URL objects,
      base-URL path resolution, absent defaults, and per-request URL/link error behavior.
      Six runtime/package/parity/documentation gates, 26 Node 20 web/hook/graph tests, and all
      nine site-profile gates pass the runtime-origin follow-up.
- [x] #34 — Reject non-UTF-8 namespace components rather than silently dropping them.
- [x] #35 — Reject files outside the configured namespace root.
- [x] #36 — Reject locale files outside the configured locale root.
- [x] #37 — Preserve I/O error kind, path, and operation context.
- [x] #38 — Define reproducible normalized path ordering.
- [x] #39 — Generate only executable nested web-policy combinations.

### `linguini-syntax`

- [x] #40 — Implement and test the normative inline `fn` grammar end to end.
- [x] #41 — Distinguish a reference from a zero-argument call.
- [x] #42 — Support zero-argument form/function declarations consistently.
- [x] #43 — Preserve `form` versus `fn` declaration identity.
- [x] #44 — Make text mode explicit and recovery-safe instead of switching after every `=`.
- [x] #45 — Parse dedented and raw multiline text blocks without losing newlines.
- [x] #46 — Enforce the reserved-name policy.
- [x] #47 — Enforce declaration naming conventions.
- [x] #48 — Diagnose empty enums, duplicate variants, and duplicate parameters.
- [x] #49 — Decode string escapes into semantic values.
- [x] #50 — Provide an unambiguous literal-brace escape.
- [x] #51 — Share locale-tag validation with configuration.
- [x] #52 — Return parser diagnostics instead of panicking on recovery invariants.
- [x] #53 — Restrict public AST mutation behind parser-owned immutable file roots. External
  consumers can inspect declarations and spans through accessors but cannot replace or mutate a
  parsed root into a grammar-invalid state.
- [x] #54 — Attach source IDs to spans.
- [x] #55 — Preserve complete override metadata and span.
- [x] #56 — Quarantine recovered/error nodes so invalid bytes cannot stitch declarations.
- [x] #57 — Make documentation attachment deterministic.
- [x] #58 — Validate deep property-path shapes.
- [x] #59 — Complete Unicode, recovery, nesting, and size fuzz/property coverage. Seeded
  proptest suites generate arbitrary Unicode recovery inputs, recursively nested valid groups,
  and large valid locale files; the parser properties enforce no-panic and complete-parse
  invariants.

### `linguini-schema`

- [x] #60 — Diagnose duplicate enum variants before map construction.
- [x] #61 — Detect self-aliases and alias cycles.
- [x] #62 — Preserve formatter annotations on aliases.
- [x] #63 — Diagnose duplicate message parameters.
- [x] #64 — Keep recursive group membership and the message index consistent.
- [x] #65 — Enforce schema naming constraints.
- [x] #66 — Attach unknown-type diagnostics to exact type spans.
- [x] #67 — Preserve related-file source identity.
- [x] #68 — Make the schema builder a view over the shared semantic database. Cross-file
      `SchemaDatabase` now owns the schema sources, immutable symbols, schema diagnostics, public
      messages, and locale coverage/type/exhaustiveness entry points; CLI and LSP consume that
      boundary. The unused locale-scope database is removed and schema/locale navigation indexing
      is owned by the same semantic crate.
- [x] #69 — Require a cross-file schema merge pass before every codegen entry.
- [x] #70 — Cover cycles, duplicate variants/parameters, and cross-file declarations.

### `linguini-locale`

- [x] #71 — Store variables in a dedicated variable index.
- [x] #72 — Replace cross-kind overrides atomically across all indexes.
- [x] #73 — Reject same-file overrides before replacement.
- [x] #74 — Prevent recovered declarations from shadowing valid parent declarations.
- [x] #75 — Preserve source identity on related diagnostic spans.
- [x] #76 — Stop registering groups as phantom messages.
- [x] #77 — Validate source order against the locale/path hierarchy.
- [x] #78 — Keep the declaration store and kind indexes synchronized.
- [x] #79 — Diagnose duplicate members and cross-source collisions.
- [x] #80 — Cover variable classification, override cleanup, recovery, and source order.

### `linguini-ir`

- [x] #81 — Preserve spans and source IDs through lowering.
- [x] #82 — Preserve locale enum declarations in lowered IR.
- [x] #83 — Represent override provenance and replacement explicitly.
- [x] #84 — Traverse expressions in forms, attributes, maps, and nested objects.
- [x] #85 — Resolve complete member and property paths.
- [x] #86 — Check call arity and argument types.
- [x] #87 — Preserve reference versus call identity.
- [x] #88 — Detect variable/helper dependency cycles.
- [x] #89 — Validate formatter kinds and options before codegen.
- [x] #90 — Normalize documentation consistently.
- [x] #91 — Define plural through the shared builtin registry.
- [x] #92 — Attach sources and related graph edges to IR diagnostics.
- [x] #93 — Replace mutable public vectors with validated collections. `IrModule` fields are
      sealed; `IrModuleBuilder::build` rejects repeated `(kind, name)` pairs, and
      `IrModule::try_append` moves declarations atomically so a conflicted merge leaves the
      target untouched. CLI schema/locale merges now fail visibly at the composition boundary,
      including the cross-file namespace-collision case previously caught only by the late
      reference checker (`013c3cd`).
- [x] #94 — Diagnose duplicates before set/map insertion.
- [x] #95 — Finish splitting schema, locale, and validated project IR types. `SchemaIr` and
      `LocaleIr` reject cross-domain declarations, `ValidatedProjectIr` preserves those domain
      types after semantic validation, CLI composition returns typed modules, and the public
      TypeScript project-codegen boundary accepts only the typed schema/locale pair (`750ff11`,
      `99d3c8b`).
- [x] #96 — Make the validated capability mandatory in every public production emitter.
- [x] #97 — Cover nested references, cycles, forms, duplicates, and invalid IR.

### `linguini-analyzer`

- [x] #98 — Analyze `foo()` as a call.
- [x] #99 — Resolve and type-check complete property paths.
- [x] #100 — Check form and method call arity and types.
- [x] #101 — Require numeric plural selectors.
- [x] #102 — Type-check user-function arguments.
- [x] #103 — Type-check builtin plural arguments.
- [x] #104 — Validate local enum variants in implementations and branches.
- [x] #105 — Restrict dispatch dimensions to dispatchable types.
- [x] #106 — Represent zero-dimension functions correctly.
- [x] #107 — Diagnose duplicate map keys and branches.
- [x] #108 — Diagnose unsafe local/global shadowing.
- [x] #109 — Use a complete type environment for retained language constructs.
- [x] #110 — Prevent an early wildcard from hiding missing coverage.
- [x] #111 — Require wildcard-last and diagnose unreachable arms.
- [x] #112 — Diagnose duplicate branches.
- [x] #113 — Check multidimensional and nested coverage.
- [x] #114 — Accept `other` only where the dispatch type defines it.
- [x] #115 — Generate structurally valid missing-arm quick fixes.
- [x] #116 — Generate meaningful implementation-variant fixes.
- [x] #117 — Prevent locale enums from silently overwriting schema enums.
- [x] #118 — Diagnose unknown form targets and types.
- [x] #119 — Implement the retained lint set with stable codes.
- [x] #120 — Add project/application usage data for unused-message analysis.
- [x] #121 — Diagnose cross-file and cross-kind duplicate declarations.
- [x] #122 — Preserve both declarations when reference-graph nodes collide.
- [x] #123 — Report one complete strongly connected component per cycle.
- [x] #124 — Include source paths and graph edges in cycle diagnostics.
- [x] #125 — Memoize graph traversal.
- [x] #126 — Cover all locale-level project reference kinds.
- [x] #127 — Emit stable diagnostic codes, categories, and source IDs.
- [x] #128 — Render cross-file related spans correctly in every CLI path.
- [x] #129 — Validate span bounds before rendering.
- [x] #130 — Give quick fixes explicit edit/command semantics.
- [x] #131 — Represent source-less diagnostics without fake spans.
- [x] #132 — Measure terminal diagnostic columns by display width.
- [x] #133 — Attach lint identity and severity policy.

### `linguini-format`

- [x] #134 — Reject unknown file extensions.
- [x] #135 — Apply and test `sort_enum_variants`.
- [x] #136 — Complete migration from heuristic tokens to the canonical syntax model. Formatting
      first requires a validated parser-owned AST; that tree classifies declaration and override
      headers, placeholder braces, and dedented/raw text modes, while the paired lossless token
      stream is retained only for concrete spelling and trivia. Literal keyword matching no longer
      controls structural continuation layout (`2399d15`, `928f432`, `d48f536`).
- [x] #137 — Remove private-use sentinel corruption.
- [x] #138 — Measure grapheme/display width correctly.
- [x] #139 — Preserve source newline style.
- [x] #140 — Report invalid spans instead of emitting empty text.
- [x] #141 — Enforce the documented line-width contract.
- [x] #142 — Bound indentation/width allocation and overflow.
- [x] #143 — Preserve comments under recovery.
- [x] #144 — Eliminate redundant parse/lex passes.
- [x] #145 — Document and test zero-width “unlimited” behavior.
- [x] #146 — Prove formatting preserves message semantics.
- [x] #147 — Add idempotence, comment, Unicode, CRLF, and malformed-input properties.

### `linguini-cldr`

- [x] #148 — Reject repeated or mixed numeric signs.
- [x] #149 — Support exponent notation and compact operands `c`/`e`.
- [x] #150 — Avoid integer overflow in plural operands.
- [x] #151 — Avoid lossy `f64` conversion.
- [x] #152 — Define exact integer conversion semantics.
- [x] #153 — Parse plural operands once.
- [x] #154 — Remove category-order dependence for empty rules.
- [x] #155 — Return structured plural parse errors with offsets.
- [x] #156 — Implement canonical aliases, parent locales, and likely subtags.
- [x] #157 — Support CLDR numbering systems beyond `latn`.
- [-] #158 — Expand date/calendar/context/time-zone/skeleton support. Exact Gregorian
      `availableFormats` IDs are resolved during Rust compilation, including schema defaults.
      Source validation rejects style conflicts; unknown IDs and unsupported fields fail before
      output mutation. Every pinned entry across 508 resolved locale records is checked, and
      generated English/Russian/French/Chinese requests execute with host formatting disabled.
      The nine-locale site also verifies the exact owning-page payload, absence of time-field
      operations, concurrent SSR, hydration, locale navigation, and no-JS requests.
      Gregorian quarter fields `Q`/`q` now compile numeric arithmetic or four-name vectors for
      all widths and contexts. Pinned quarter skeletons/symbols are checked across 508 locale
      records; generated requests execute in four locales, with UTC boundaries, Arabic digits,
      supplied-Date getter observations, strict JS typing, and unused quarter removal covered.
      Date/time composition preserves inserted literal markers and apostrophe quoting
      (GEN-A62). Shared tests check all 32,512 pinned style combinations for unchanged patterns;
      executed English/Arabic requests retain literal digits, UTC rollover, and getter behavior.
      Week-based year `Y` now compiles territory week conventions into bounded January/December
      arithmetic, including Scottish Gaelic `yMMM`, signed extended years, localized digits/signs,
      and independent calendar-year/era fields. A full Gregorian cycle checks all 49 week-rule
      combinations; executed eight-locale coverage checks eight widths and BCE/CE boundaries.
      Impossible boundary branches and unused helpers/padding/sign bindings are omitted.
      Canonical `yw` now compiles its week number and count-specific wording through shared
      `WeekDatePlan`/`IntegerPluralPlan`, with no runtime CLDR interpreter. A full Gregorian
      cycle proves every day under all 49 week conventions; raw-pattern execution covers 413
      years and 17 locale tags, including all nine tags with different live wording. Native
      preview comparisons cover 1,044 cases in three host zones, alongside public cold/warm,
      ordinary/physical, schema-default/function, and preserved-output-on-error checks (GEN-A47).
      Strict JS/TS 5.8/5.9/6, Node 20, Clippy, Rust 1.87, patch API, and complete Vite 7/8 consumers
      pass; existing generated code/declaration/map snapshots remain unchanged. Closest-match skeletons, combined date/time synthesis, other calendars, and selectable
      non-UTC zones remain open.
      Valid Unicode `fw` preferences now override the compiled first weekday through the shared
      BCP 47 parser while preserving territory minimum days (GEN-A48). Seven weekdays across
      pinned locale candidates, 24-tag raw week execution, 1,276 native/generated cases in three
      host zones, and public ordinary/physical builds pass on Node 20. Equivalent preferences
      emit identical formatter bytes. Workspace Clippy, Rust 1.87 native/WASM, 223 patch API
      checks, strict JS/TS 5.8/5.9/6, documentation gates, and complete Vite 7/8 consumers pass.
      Regular `rg` region overrides, `sd` subdivisions, and ISO week preferences now resolve
      at compilation too (GEN-A60). Pinned containment/alias data supplies 257 regular regions
      and 5,027 regular subdivisions; `rg` accepts both region and subdivision overrides.
      Shared canonicalization applies 147 pinned subdivision aliases before validity checks.
      Unrecognized, grouping, and malformed preferences are ignored. First-day precedence
      and the ISO four-day minimum are independent. Missing explicit territory rows and unmatched likely regions
      use world rules. Both week families pass boundary execution and 3,456 native/generated
      cases per family, executed in three host zones,
      and cold/warm ordinary/physical builds on Node 20. Equivalent preferences retain identical
      formatter bytes; existing application goldens stay unchanged. Clippy and Rust 1.87
      native/WASM, 223 patch API checks per affected data crate, reproducible generation,
      and all 71 full-profile gates pass. Closest matching, other calendars, and selectable
      zones remain open.
      The complete pinned validity registry now records 626 deprecated subdivision IDs,
      including those without replacement aliases (GEN-A61). Its regular and unknown sets are
      checked against the JSON data; all 11,820 `sd`/`rg` cases execute after canonicalization.
      Native/generated week cases also cover Northern Sotho zero wording, Maltese week six,
      and Sanskrit without cardinal data. Cold/warm ordinary/physical builds and prepared
      project entries pass. The new parser is confined to the offline maintainer tool;
      existing compiler formatting records, regular lookup arms, and application goldens
      remain unchanged. Equivalent regular/obsolete preferences emit identical formatter bytes.
      All 198 emitter tests and affected shared/CLI/data suites pass on Node 20; the expanded
      week corpus passes on Node 24. Each family checks 4,096 native/generated cases in three
      host zones. Clippy, Rust 1.87 native/WASM, reproducible generation, 223 patch API checks,
      seven strict-date/documentation gates, and dependency policy/advisory checks pass.
      Broader date/calendar criteria remain open.
      Canonical month-week `MMMMW` now uses the same finite family planner and direct arithmetic
      for the selected convention's reachable subset of 0–6 (GEN-A54). Week zero, week six,
      formatting/standalone month names, localized digits, and final-month ownership execute
      correctly without year/leap calculations or runtime CLDR parsing. All 49 conventions and
      a full Gregorian cycle are checked; raw-pattern execution covers every live wording variant,
      extended years, TimeClip endpoints, getter counts, and caller errors. Native/generated
      parity covers 1,664 cases in three zones, plus cold/warm public builds, previews, function
      forwarding, and preserved failed-build output. All 57 common/194 emitter tests and CLI
      tests pass on Node 20, with changed cases on Node 24. Clippy, Rust 1.87 native/WASM,
      223 patch API checks per changed codegen crate, strict JS/application TS 5.8/5.9/6, nine
      date/project/documentation gates, and eight complete Vite 7/8 consumers pass. Existing code/declaration/map snapshots stay unchanged. Broader skeleton
      matching and other calendars/zones remain separate.
      Week families with no cardinal record now compile one unconditional pattern only after
      proving all available count variants equal to the fallback plan (GEN-A55). Eligible pinned
      locales, equivalent quoting, differing-pattern/missing-fallback guards, and Sanskrit raw
      pattern execution are checked. Both families execute ordinary/physical/standalone output
      for `sa`, `ccp`, `aa`, and `la`; native parity expands to 1,792 cases per family in three zones.
      All 59 common/195 emitter tests and CLI tests pass on Node 20, with changed cases on Node 24.
      Clippy, Rust 1.87 native/WASM, 223 patch API checks per changed codegen crate, strict
      JS/application TS 5.8/5.9/6, nine date/project/documentation gates, and four complete Vite 7/8
      consumers pass. Existing snapshots stay unchanged; explicit plural operations still require
      rules. This removes an unused compile-time requirement without a runtime fallback.
- [x] #159 — Apply currency symbols, digits, increments, and spacing.
- [x] #160 — Share one plural grammar implementation across all targets. `linguini-cldr` owns the
  sole rule parser; the offline generator imports it and TypeScript codegen lowers its typed AST.
- [x] #161 — Correct documented plural-category examples.
- [x] #162 — Compare pinned CLDR cardinal evaluation with host ICU `Intl.PluralRules` across 152
  English, French, Russian, Arabic, Polish, Czech, Slovenian, and Welsh integer/decimal cases.

### `linguini-cldr-macros`

- [x] #163 — Remove compiler-time network and `git` operations.
- [x] #164 — Remove deletion of environment-selected checkout paths.
- [x] #165 — Make compiler CLDR inputs read-only.
- [x] #166 — Make CLDR builds hermetic and offline.
- [x] #167 — Pin the full immutable CLDR identity and content manifest.
- [x] #168 — Remove remote-ref resolution from compilation.
- [x] #169 — Validate every required locale subtree.
- [x] #170 — Remove compiler-process checkout locks.
- [x] #171 — Remove stale lock ownership ambiguity.
- [x] #172 — Remove racy lock cleanup.
- [x] #173 — Use tracked packaged inputs for cache invalidation.
- [x] #174 — Move heavy data generation out of the compiler process.
- [x] #175 — Fail on malformed or missing locale payloads.
- [x] #176 — Generate non-Latin numbering and broader calendar data. Pinned artifacts include
      78 numbering systems and 3,060 locale/calendar pairs across Gregorian, Buddhist, ROC, and
      Japanese calendars, with contextual symbols, eras, skeletons, day periods, and week rules.
- [x] #177 — Parse text-direction JSON structurally.
- [-] #178 — Complete CLDR number-pattern semantics. The typed model covers quoted affixes,
      group widths, fixed/significant digits, rounding increments, exponent widths/sign flags,
      scaling, and padding. Decimal/currency generation now honors localized numeric affixes
      and minimum grouping thresholds after rounding; native previews use the same thresholds.
      Currency separators also resolve pinned `currencyDecimal`/`currencyGroup` overrides during
      compilation, independently of ordinary number symbols. Generated/native comparisons cover
      Austrian German and Swiss French alongside the existing digit-system corpus.
      Internal scientific lowering also compiles localized exponent separators and signs,
      preserving bidirectional marks and literal digits within those symbols. Broader scientific
      and engineering patterns remain part of OPT-A1/A2 rather than a completed surface.
      Offline numeric-pattern parsing now honors padding escape characters, restricts padding
      to the four component boundaries, and treats rounding digits 1–9 as mandatory digits
      (GEN-A65). It reuses the shared padding model. All 24 maintainer, 60 common planner, and
      202 emitter tests pass; 96 generated padding cases execute on Node 20/24. Pinned CLDR
      artifacts reproduce byte for byte. Clippy and Rust 1.87 pass; broader pattern semantics
      remain open.
- [x] #179 — Validate model limits before narrowing integer casts.
- [x] #180 — Share the runtime plural grammar. Generated Rust predicates and TypeScript predicates
  both lower the canonical `linguini-cldr::PluralRule`; neither target reparses CLDR rule text.
- [x] #181 — Parse the CLDR manifest as real TOML.
- [x] #182 — Verify the vendored archive/tree cryptographically.
- [x] #183 — Validate the packaged full-corpus manifest and golden hash.

### `linguini-codegen-ecmascript` core emitter

- [x] #184 — Emit form branches as callable dispatch rather than discarding them.
- [x] #185 — Preserve every selector in multi-key branches.
- [x] #186 — Dispatch maps by their semantic enum/string/plural type.
- [x] #187 — Emit explicit zero-argument calls.
- [x] #188 — Reject unknown formatters at the mandatory validated boundary.
- [x] #189 — Allocate safe, collision-free JavaScript identifiers.
- [x] #190 — Allocate safe, collision-free type and parameter identifiers.
- [x] #191 — Escape carriage returns and control characters completely.
- [x] #192 — Guard recursive alias resolution.
- [x] #193 — Validate currency codes before runtime.
- [x] #194 — Apply currency minor units and CLDR currency rules.
- [x] #195 — Preserve large-number precision.
- [x] #196 — Make date output independent of host local time zone.
- [x] #197 — Reject invalid dates.
- [x] #198 — Fail visibly on missing required CLDR data.
- [x] #199 — Emit source maps back to Linguini sources. The common backend maps individual
      schema enums, aliases, messages, and groups in TS/JS output and schema declaration
      companions. Decoded v3 mappings verify exact multi-file source positions and stable source
      identity; CLI maps retain project-relative paths and exact sourcesContent.
- [x] #200 — Require every declared message in direct codegen input locales.
- [x] #201 — Replace eager locale imports with real bundler-visible splitting. The bundler-native
      Vite path exposes one virtual dynamic entry per effective locale, with no per-message client
      entries; the generated runtime index statically imports only the base locale and uses real
      dynamic imports for every non-base locale.
- [x] #202 — Tree-shake transitive forms, functions, variables, and helpers.
- [x] #203 — Diagnose unknown `included_messages`.
- [x] #204 — Stop copying all global declarations into every namespace.
- [x] #205 — Deduplicate formatter helpers/data per locale.
- [x] #206 — Sanitize namespace paths and generated filenames.
- [x] #207 — Reject case-folded locale filename collisions.
- [x] #208 — Use the shared CLDR fallback graph. Rust codegen, CLI fallback composition, and
      generated locale normalization consume the canonical pinned graph; generated runtime uses
      one project-specific lookup with no independent truncation or script heuristic.
- [x] #209 — Reject an invalid base locale.
- [x] #210 — Reject unknown text direction.
- [x] #211 — Remove `targets.ts.module` and all CJS documentation/templates. The parser retains
      only a value-level migration probe so the removed key still receives a targeted diagnostic.
- [x] #212 — Make all parameterless messages public values.
- [x] #213 — Reject empty locale sets.
- [x] #214 — Treat missing dispatch branches as validated-IR failures.

### Generated web/SvelteKit runtime

- [x] #215 — Remove unsupported strategy dispatch and generate selected source modules.
- [x] #216 — Make default source behavior match validated configuration.
- [x] #217 — Accept only the supported SvelteKit header interface.
- [x] #218 — Parse `Accept-Language` weights and wildcards correctly.
- [x] #219 — Handle malformed cookie percent encoding safely.
- [x] #220 — Carry validated cookie constraints into the generated runtime.
- [x] #221 — Append rather than overwrite existing `Set-Cookie` headers.
- [x] #222 — Prevent public URL localization from rewriting external URLs.
- [x] #223 — Match only exact locale path segments.
- [x] #224 — Derive trailing-slash behavior from SvelteKit routes.
- [x] #225 — Compile exclusion globs without prefix overmatching.
- [x] #226 — Make regular-expression matching stateless.
- [x] #227 — Define a consistent URL parse/error contract.
- [x] #228 — Replace unsafe regex HTML rewriting.
- [x] #229 — Share server/client link skip rules.
- [x] #230 — Preserve SvelteKit streaming.
- [x] #231 — Namespace generated `locals` fields.
- [x] #232 — Provide explicit hook composition contracts.
- [x] #233 — Wire configured redirect status or remove the dead option.
- [x] #234 — Give runtime link observers per-app/HMR ownership.
- [x] #235 — Batch and bound DOM observation.
- [x] #236 — Replace eager locale imports from the generated web/runtime index. It imports only
      the base locale eagerly, deduplicates asynchronous non-base preparation, and gives direct
      synchronous callers an explicit preparation contract.

### `linguini-cli`

- [x] #237 — Contain output beneath the project.
- [x] #238 — Reject output overlap with project and source roots.
- [x] #239 — Reject canonical and symlink escapes before writes.
- [x] #240 — Use owned randomized staging/backup directories.
- [x] #241 — Report post-commit cleanup separately from build success.
- [x] #242 — Surface and recover rollback failures.
- [x] #243 — Write and enforce a generated-tree ownership manifest.
- [x] #244 — Make cross-file schema validation mandatory before generation.
- [x] #245 — Namespace forms, functions, variables, and origins consistently.
- [x] #246 — Diagnose duplicate `(namespace, locale)` inputs.
- [x] #247 — Use one fallback service shared with codegen/runtime.
- [x] #248 — Route analyzer errors as blocking errors.
- [x] #249 — Continue safe independent semantic checks after syntax errors.
- [x] #250 — Add `--deny-warnings`.
- [x] #251 — Replace the remaining incomplete raw-IR reference gate.
- [x] #252 — Contain format writes and reject symlink escape.
- [x] #253 — Make formatting writes atomic.
- [x] #254 — Use content hashes for optimistic fix concurrency.
- [x] #255 — Report `CreateFile` no-ops accurately.
- [x] #256 — Select fixes by stable IDs rather than title/suffix matching.
- [x] #257 — Generate safe nested stub names and detect collisions.
- [x] #258 — Detect obsolete, duplicate, and overlapping fixes.
- [x] #259 — Report `init` changes versus existing/no-op paths accurately.
- [x] #260 — Respect TTY and `NO_COLOR`.
- [x] #261 — Load schema once for all sample locales.
- [x] #262 — Qualify sample namespaces consistently.
- [x] #263 — Guard sample alias recursion.
- [x] #264 — Bound Cartesian sample generation.
- [x] #265 — Use canonical primitive types in samples.
- [x] #266 — Return explicit evaluator errors.
- [x] #267 — Share production/sample formatting semantics.
- [x] #268 — Cover decimals and large numeric samples.
- [x] #269 — Add machine-readable JSON and SARIF output.
- [x] #270 — Return LSP startup errors instead of panicking.

### `linguini-lsp`

- [x] #271 — Analyze locale documents only against their namespace schema.
- [x] #272 — Resolve references through the workspace index.
- [x] #273 — Rename resolved symbols only.
- [x] #274 — Validate rename targets, grammar, scope, and collisions.
- [x] #275 — Navigate to resolved declarations.
- [x] #276 — Resolve namespace ambiguity for hover and definitions.
- [x] #277 — Complete context/type-aware completion beyond indexed names. Schema type positions,
  locale placeholders, and typed dispatch keys now return filtered candidates with accurate LSP
  kinds/details; matching schema parameters and enclosing plural/enum variants are resolved.
- [x] #278 — Integrate schema semantic diagnostics.
- [x] #279 — Preserve independent semantic diagnostics under recoverable syntax errors.
- [x] #280 — Move filesystem discovery out of async request hot paths.
- [x] #281 — Cache parsed/indexed project state.
- [x] #282 — Bound discovery and prevent symlink cycles.
- [x] #283 — Debounce, cancel, and version document analysis.
- [x] #284 — Publish diagnostics with document versions.
- [x] #285 — Recompute dependent diagnostics on close.
- [x] #286 — Track disk, configuration, and source changes.
- [x] #287 — Surface poisoned internal state.
- [x] #288 — Use robust file URI/path conversion.
- [x] #289 — Reject unsupported language IDs.
- [x] #290 — Use half-open token ranges.
- [x] #291 — Emit multiline raw-string semantic tokens.
- [x] #292 — Classify tokens from semantic identity.
- [x] #293 — Remove inert code actions.
- [x] #294 — Offer rename only on a resolved symbol.
- [x] #295 — Resolve overlapping apply-all edits deterministically.
- [x] #296 — Preserve notes, codes, related sources, and quick-fix metadata.
- [x] #297 — Declare and implement executable quick-fix commands.
- [x] #298 — Return formatting failures to the client.
- [x] #299 — Respect formatting options and minimize edits.
- [x] #300 — Index complete workspace symbols.
- [x] #301 — Honor `include_declaration` for references.
- [x] #302 — Surface incomplete discovery/read results.
- [x] #303 — Reject unrelated nearest-config associations.
- [x] #304 — Enforce document, project, file, recursion, and nesting limits.
- [-] #305 — Share preview semantics with the production evaluator. Native numeric previews
      now honor localized negative affixes, minimum grouping thresholds, and pinned currency
      symbols. Date-only previews now reuse production request/pattern/symbol/day-period/week
      plans for exact skeletons, time-only output, and composed styles. All source-selectable
      skeletons across 508 resolved locales render; 2,880 comparisons across 12 locales and 30
      requests execute real generated modules, covering year zero, week boundaries, contextual
      labels, and BMP/supplementary digits. Numeric previews share the Unicode-scalar digit
      writer with dates; 462 generated-module comparisons across fourteen locales check decimal,
      standard currency, and accounting output, including exact large/exponent strings, rounding,
      grouping, and negative zero. Affix-literal digits and mixed supplementary symbols have
      focused coverage. Explicitly formatted call arguments and inline payloads now retain their
      string results in previews; twelve native/generated comparisons across English/Russian
      number, currency, and date requests cover the correction. The broader generated Date
      input domain remains open. Native previews now prepare each locale through shared default/
      inline lowering, removing their separate inline evaluator and default planner. Fourteen
      additional English/Russian comparisons preserve unformatted Number/currency-alias/Date
      function payloads, nested calls, schema defaults, inline bindings, and captured defaults;
      message and callable alias-cycle errors remain covered. Generated artifacts stay unchanged.
      CLI unit/integration tests pass on Node 20, with the final preview suite, strict Clippy,
      Rust 1.87, and executable documentation checks also passing.
      Form lookup also resolves enum aliases through the common bounded resolver. Twelve
      English/Russian native/generated comparisons cover properties, callable plural forms,
      alias-typed forwarded payloads, and direct-enum controls, without changing source syntax
      or generated artifacts. Qualified namespace coverage passes on Node 20 and Node 24;
      CLI unit/integration tests, Clippy, Rust 1.87, and documentation registration pass.
      Native build and preview now share validated project-IR loading, namespace qualification,
      guarded composition, and common locale planning. Preview inheritance follows declared
      CLDR parents before the base, and exact alias spellings do not inherit a canonical sibling.
      Native build/execution and preview cases verify both corrections and exact parent source IDs;
      invalid nested formatter options now fail preview preparation too.
      ISO date-time previews now normalize signed offsets, valid `24:00`, fractions, and calendar/
      year rollover to UTC; clock fields and fixed/flexible periods follow displayed precision.
      Twelve locales × thirty requests × twenty-four valid inputs execute against generated
      modules in three host zones, alongside invalid calendar/time/offset rejection. Native
      Date samples add noon and offset cases after the preserved date-only baseline. Typed
      epoch-number parity is now covered by GEN-A46: values retain their kind through nested
      functions, aliases, inline bindings, callable forms, and explicit formatter results. Native
      `generate` adds `0`/`-1` after the ISO samples. All 144 generated-module cases execute in
      three host zones; 1,042 independent JavaScript Date comparisons verify TimeClip bounds,
      fractional truncation, negative epochs, BCE/year zero, and UTC fields. Native Date-object
      parity remains open. CLI unit/integration suites pass on
      Node 20 and Node 24, with workspace Clippy, Rust 1.87, executable documentation, and
      packaged-source checks passing. Generated code/declaration/map expectations are unchanged.
      Plural input previews now follow ECMAScript trimming, accepting BOM while rejecting NEL
      (GEN-A63). All 25 trim characters, internal whitespace, invalid controls, exact fractions,
      compact exponents, and category passthrough execute through five call paths in five locales:
      7,375 native/generated comparisons pass on Node 20 and 24. CLI suites, workspace Clippy,
      and Rust 1.87 pass; application artifacts stay unchanged.
      Decimal/currency previews now consume `NumberFormatPlan` for affixes, grouping thresholds,
      fraction precision, and rounding quantum, removing their separate static-choice assembly.
      Pattern-defined increments retain their quantum unless supplemental currency data overrides
      it. Invalid or unsupported internal plan shapes fail explicitly. Grouping uses the compiled
      widths/threshold rather than a per-call array/reversal/join; the existing fourteen-locale
      generated-module corpus retains its exact numeric, separator, sign, and digit behavior.
      CLI tests pass on Node 20/24, alongside workspace Clippy, Rust 1.87, executable syntax,
      documentation registration, and packaged-source checks. Generated artifacts stay unchanged.
- [x] #306 — Resolve preview locale from project layout.
- [x] #307 — Return runtime-construction errors rather than panic.
- [x] #308 — Reuse indexed diagnostics for code actions.
- [x] #309 — Handle workspace-folder and file create/rename/delete changes.

### `linguini-test-support`

- [x] #310 — Use atomic random temporary directory creation.
- [x] #311 — Never reuse an existing fixture directory.
- [x] #312 — Reject path separators and traversal in fixture names.
- [x] #313 — Return creation/time errors instead of panicking.
- [x] #314 — Surface cleanup failures.
- [x] #315 — Use `tempfile::TempDir`.
- [x] #316 — Locate or package fixtures without assuming a monorepo checkout.
- [x] #317 — Cover isolation, collisions, traversal, and cleanup.

### Vite plugin

- [x] #318 — Watch only configured Linguini source roots.
- [x] #319 — Remove obsolete watches after config/path/unlink changes.
- [x] #320 — Queue a dirty follow-up build for changes during a build.
- [x] #321 — Catch watcher callback rejections.
- [x] #322 — Rebuild on file/directory deletion.
- [x] #323 — Report build failures through Vite with source context.
- [x] #324 — Invalidate configured output modules without hard-coded paths.
- [x] #325 — Handle an initially absent config intentionally.
- [x] #326 — Parse TOML through the shared real-TOML path.
- [x] #327 — Synchronize plugin version with the release train.
- [x] #328 — Run real Vite 5, 6, 7, and 8 builds.
- [x] #329 — Cover races, failures, config changes, unlink, output, and Windows paths.
- [x] #330 — Debounce and serialize CLI rebuild processes.
- [x] #331 — Document the source-JavaScript Node ESM compatibility contract.

### VS Code extension

- [x] #332 — Bundle a compatible native server in target VSIX packages.
- [x] #333 — Synchronize extension/compiler/package versions.
- [x] #334 — Serialize and debounce client restarts.
- [x] #335 — Dispose stopped clients instead of accumulating subscriptions.
- [x] #336 — Prevent client reassignment/start-stop races.
- [x] #337 — Remove the unused client-construction context parameter.
- [x] #338 — Support multi-root working directories.
- [x] #339 — Resolve document substitutions only with an actual document context.
- [x] #340 — Debounce config-watcher restart storms.
- [x] #341 — Add unit, packaging, and real-server smoke coverage.
- [x] #342 — Pin packaging tools and avoid `npx --yes`.
- [x] #343 — Use one lockfile convention inside the extension.
- [x] #344 — Use the shared recoverable Rust lexer and semantic walk for highlighting, replacing
  the parallel TextMate grammar and grammar goldens. Cover complete syntax fixtures, UTF-16,
  quotes/escapes, multiline strings, interpolation, incomplete edits, language configuration,
  Markdown fence mapping, packaged native tokens, and real VS Code restart/edit behavior.
- [x] #345 — Perform a compiler/LSP protocol version handshake.
- [x] #346 — Verify clean extension install, compile, package, and VSIX contents.

### Site and documentation

- [x] #347 — Build/check the site without Cargo, network, `git`, or a Rust toolchain. Normal
  generation restores the bounded, traversal-safe packaged output; runtime tests, strict Svelte
  checking, and the production build pass, and Rust is reserved for the explicit refresh command.
- [x] #348 — Remove unshipped syntax/CJS claims and align the reference with conformance.
- [x] #349 — Reframe incomplete type, exhaustiveness, and unused-message guarantees.
- [x] #350 — Replace the unsupported site `preferredLanguage` strategy.
- [x] #351 — Define project and path namespace behavior unambiguously.
- [x] #352 — Execute examples through compile, typecheck, and runtime golden tests. The actual
  Getting Started and locale-provider projects pass, and every web-guide TypeScript, HTML, and
  Svelte variant is extracted into a generated SvelteKit project and passes strict checks. The
  harness rejects newly added normative web-guide blocks without executable fixture ownership.
- [x] #353 — Document actual CLDR formatter limits and fallback behavior.
- [x] #354 — Resolve the unfinished contracts tracked by `REPOSITORY_STATE`.
- [x] #355 — Synchronize version, copy, and release status across every surface.

### CI, release, and supply chain

- [x] #356 — Check the declared Rust minimum in a dedicated CI job. `cargo-msrv` discovery across
      all workspace targets measures 1.87; CI reads the workspace declaration and verifies it
      with `cargo-msrv`. Development and normal CI follow `stable`. Packaged documentation
      inherits workspace package metadata instead of maintaining a separate compiler floor.
- [x] #357 — Compile from packaged, offline, read-only CLDR data.
- [x] #358 — Land and execute Linux, Windows, and macOS CI. Complete workspace suites pass on
      all three hosts for the shared-backend source verified in ESM-M11.
- [x] #359 — Land dependency, license, and advisory checks. Tokio 1.54.1 and URL 2.5.9
      resolve the advisories reported by push run 45669523927 without suppressions; host-toolchain
      cargo-deny and cargo-audit pass, and the locked workspace still checks on Rust 1.87.
- [x] #360 — Add enforceable coverage plus parser/formatter property jobs. CI rejects workspace
  line coverage below 80%, runs seeded parser/formatter/plural/path property suites separately,
  and rejects surviving mutants in pull-request diffs.
- [x] #361 — Real-site generated TypeScript/JSDoc/`.d.ts` overloads have positive/negative type
  checks and runtime parity tests. The standalone full-profile corpus now typechecks and executes
  documented JavaScript/TypeScript calls and separately verifies JSDoc wrappers, positional/named
  success, missing parameters/properties, excess properties, and invalid JSDoc argument types.
- [-] #362 — Complete real Vite, native VSIX, npm CLI, and WASM LSP integration CI.
      Real Vite 5/6/7/8 tests pass with ANSI colors enabled; diagnostic assertions strip control
      codes while retaining exact message checks. Project and web/Svelte/Kit package tests pass
      on Node 20 with standalone pnpm 11 and a fresh metadata cache. Packed consumers fetch only
      missing cache data through prefer-offline installs. Windows TypeScript output arguments
      now use portable separators; the existing platform matrix remains enabled. Confirmation
      of the workflow repair on GitHub-hosted runners remains required after pushing.
      Push run 45689095669 confirms 22 successful jobs. Its remaining Windows failures exposed
      emit-input spelling and namespace-watch filtering errors, reproduced by local regression
      cases covering both separator styles, path components, drive/case rules, and cleanup.
      The macOS job never obtained a runner: GitHub rejected its older configuration. It now
      uses `macos-latest`; confirmation of these remaining repairs on hosted runners is pending.
      Current Linux Vite/installed native and universal VSIX receipts are recorded under ESM-M11;
      they do not establish the pending hosted Windows/macOS results or release signing.
      Later [push run 45954831211](https://github.com/antepodeum/Linguini/actions/runs/45954831211)
      on `aec1d335` failed during local dependency refresh: frozen installs had package contents
      but no registry metadata for the subsequent offline update. The earlier installer fixture
      contained only directory dependencies and missed this constraint. Setup now rebuilds a
      local-package consumer's modules from its frozen graph, retaining the prior installation
      until success. A fully offline registry-dependency fixture reproduces `NO_OFFLINE_META`
      before the fix and passes cold/warm afterward with an empty metadata cache and unchanged
      lock/declarations. Failure and graph-drift checks restore the prior modules too. All 23
      installer/runner tests pass on Node 20/24; actual Svelte/tsc setup paths also accept empty
      metadata caches. Node 20 Svelte/Kit runtime gates and all fourteen native tsc integration
      tests pass, including packed installation, build/watch, and project references. The complete
      updated working tree also passes all 71 full-profile gates, including rebuilt editor/WASM,
      native tsc, offline packages, documentation, and production SSR/browser/graph checks. Hosted confirmation remains
      required after pushing; this receipt does not claim the failed remote run is green.
- [x] #363 — Land and verify packaging for every publishable crate.
- [x] #364 — Land commit-SHA-pinned GitHub Actions.
- [x] #365 — Use `rust-toolchain.toml` for the moving `stable` channel and required components;
      normal CI follows the same channel. The independently measured minimum stays in Cargo metadata.
- [x] #366 — Standardize and reproducibly verify JavaScript package-manager boundaries. Every
  dependency-bearing project pins pnpm 11.1.0 and lockfile v9; VS Code migrated from npm with an
  explicit dependency-build allowlist, and root/CI policy rejects mixed lockfiles or npm installs.
- [x] #367 — Land automatic release-version synchronization checks.
- [x] #368 — Add public API/semantic-version compatibility checks.
- [x] #369 — Make documentation and examples an executable conformance suite. The root full
  profile registers all 103 code blocks, enforces Linguini syntax plus TOML config fences, and
  builds, typechecks, and executes the Getting Started JavaScript/TypeScript example from both the
  workspace and Cargo package file sets. The complete SvelteKit provider and every alternative
  web-guide hook/component/client/server variant compile and pass strict Svelte checking.
      Package staging follows Cargo path dependencies, including development manifests, while
      pruning unrelated packages only in an isolated lock and rejecting external version changes.
      Executed packaged examples pass, and workspace dry-packaging omits the unpublished test
      helper from the LSP manifest.

## Required documentation migration

### `README.md`

- [x] DOC-R1 — State SvelteKit-first, ESM-only scope.
- [x] DOC-R2 — Remove CJS, multi-framework, and overstated type-safety claims.
- [x] DOC-R3 — Use value leaves, positional calls, and named-object overloads correctly.
- [x] DOC-R4 — Explain that Vite owns tree-shaking, chunks, preload, and loading.
- [x] DOC-R5 — Make the npm CLI the default JS installation; retain Cargo as an option.

### `docs/getting-started.md`

- [x] DOC-G1 — Document runtime adapters configured in application TypeScript and hooks.
  Retire the policy DSL and nested TOML web examples; executable application examples verify
  explicit resolver capabilities and the SvelteKit build/runtime integration.
- [x] DOC-G2 — Remove `targets.ts.module` and every CJS choice.
- [x] DOC-G3 — Explain one JavaScript runtime plus JSDoc and `.d.ts`.
- [x] DOC-G4 — Remove manual message-loading setup from the normal SvelteKit path.
- [x] DOC-G5 — Build the documented project, typecheck its JavaScript and TypeScript direct-use
  variants, execute both generated ESM paths, and assert exact English runtime output.

### `docs/reference.md`

- [x] DOC-F1 — Specify recursive groups, qualified paths, and collisions.
- [x] DOC-F2 — Specify value leaves, positional calls, and named-object overloads.
- [x] DOC-F3 — Specify dedented/raw multiline semantics and brace escaping.
- [x] DOC-F4 — Specify `///` attachment and generated/editor propagation.
- [x] DOC-F5 — Specify the implemented inline `fn` grammar and semantics.
- [x] DOC-F6 — Remove CJS output and configuration completely; `.cjs` remains only as one
      application source extension understood by unused-message analysis.
- [x] DOC-F7 — Match formatter, plural, lint, typing, and exhaustiveness claims to tests.

### `docs/web-sveltekit.md`

- [x] DOC-W1 — Document `locale_prefix` values and default.
- [x] DOC-W2 — Document exact ordered locale sources and implicit default fallback.
- [x] DOC-W3 — Document cookie/local-storage defaults, validation, and enablement.
- [x] DOC-W4 — Document link modes and `localizeHref`.
- [x] DOC-W5 — Document route-exclusion matching.
- [x] DOC-W6 — Document switch route as transport over `LocaleSwitchPlan`.
- [x] DOC-W7 — Add path, cookie, hybrid, and pathless examples.
- [x] DOC-W8 — State the no-JavaScript local-storage-only limitation.
- [x] DOC-W9 — Explain SvelteKit-derived origin/base/trailing-slash values. The web guide now
      identifies `$app/paths.base`, `event.url.origin`/`window.location`, cookie-path auto
      derivation, and matched-route `trailingSlash` ownership.

### `docs/examples/sveltekit-locale-provider.md`

- [x] DOC-E1 — Use the final value/function API.
- [x] DOC-E2 — Reuse `localizeHref`.
- [x] DOC-E3 — Show browser and static switching over one transition plan.
- [x] DOC-E4 — Show `locale_prefix = "never"` with a valid persistence source.
- [x] DOC-E5 — Avoid eager/manual imports of every locale or message.

### `docs/why.md`

- [x] DOC-Y1 — Remove claims for incomplete typing, lints, exhaustiveness, and usage.
- [x] DOC-Y2 — Label intended advantages as goals until conformance proves them.
- [x] DOC-Y3 — Explain the exact per-message bundler-native graph.
- [x] DOC-Y4 — Require reproducible evidence for payload-reduction claims.

### Site, templates, generated examples, and enforcement

- [x] DOC-S1 — Migrate site config, hooks, examples, and snippets to the nested API.
- [x] DOC-S2 — Remove generated-output CJS choices from templates, help, schemas, comments, and
      screenshots. The VS Code extension's internal Node bundle format is not a codegen target.
- [x] DOC-S3 — Update editor examples for groups, docs, multiline, values, and overloads.
- [x] DOC-S4 — Publish the complete flat-to-nested migration map.
- [x] DOC-C1 — Register every normative code block as a fixture. The full profile rejects unknown
  languages, unsupported metadata, empty fences, and unclosed fences across all 91 blocks.
- [x] DOC-C2 — Build docs against packaged CLI/codegen artifacts. The full profile stages an
  isolated workspace from Cargo's package file sets, then builds both documented applications with
  that CLI/codegen graph before running the same TypeScript, JavaScript, and SvelteKit checks.
- [x] DOC-C3 — Snapshot config schema, CLI help, declarations, and examples in CI. The committed
  JSON Schema and exact CLI help snapshot have focused Rust gates; generated TypeScript
  declarations retain golden snapshot coverage, and the full profile builds, typechecks, and runs
  documented JavaScript, TypeScript, and SvelteKit examples.
- [x] DOC-C4 — Require docs, migration notes, and fixtures for every public contract change. The
  quick/full runner and CI diff gate classify public CLI/config/syntax/codegen/Vite/LSP/editor
  boundaries and reject changes missing any companion evidence class.

## Additional architecture acceptance criteria

### Application-owned web runtime policy

These requirements follow the corrected npm runtime boundary in MOD-A1–A3. Routing and
persistence settings do not select native codegen output. Runtime packages are maintained in
TypeScript, and application/bundler imports select their capabilities.

- [x] WEB-A1 — Configure routing, locale sources, cookies, storage, links, exclusions, and optional
      switch routes in application TypeScript modules/hooks. Remove `[web]` from neutral TOML,
      diagnose the retired table, and remove the policy DSL/native transport. The site and all
      documented SvelteKit variants build and typecheck against the npm adapters.
- [x] WEB-A2 — Replace `prefix_default_locale` with `always | except-default | never`.
- [x] WEB-A3 — Default routing to `except-default`.
- [x] WEB-A4 — Replace `redirect` with `canonical = redirect | preserve`.
- [x] WEB-A5 — Obtain origin/base from application environment and leave trailing slash to the framework.
- [x] WEB-A6 — Provide explicit path, cookie, local-storage, and accept-language capabilities.
- [x] WEB-A7 — Preserve explicit source order, reject duplicates, and fall back to the base locale
      when no capability resolves. The runtime does not infer hidden resolvers from routing settings.
- [x] WEB-A8 — Validate cookie options and `SameSite=None` security.
- [x] WEB-A9 — Validate provided local-storage keys and derive normalized runtime defaults.
- [x] WEB-A10 — Replace `localize_links` with transform/runtime/manual.
- [x] WEB-A11 — Validate route patterns and reset stateful regexes between evaluations.
- [x] WEB-A12 — Normalize and validate runtime options when creating application adapters.
      Preserve source order and a common browser/server transition policy; keep this policy out
      of native configuration, compiler semantics, and generated message artifacts.
      Automatic cookie security also honors the normalized application origin after per-call
      origins and before browser location. Executed header/cookie-sink cases cover both protocols,
      URL objects, absent browser state, invalid origins, and explicit policy/secure overrides.
      Four web/Kit/package/parity gates, 25 Node 20 web/hook/graph tests, and all nine site-profile
      gates pass this follow-up; the frozen comparison implementations are unchanged.
- [-] WEB-A13 — Import resolver implementations as explicit capabilities and keep metadata-only
      request hooks independent of message catalogs. Verify tree-shaken page graphs for unused
      platform operations and libraries; exact delivery remains GEN-A9/MOD-A4.
      Metadata construction now copies only declared locale fields, without enumerating or reading
      unrelated string/symbol getters or forwarding message factories. Hook and packed strict-TS
      consumers use throwing catalog getters. An executed production SSR graph shakes catalog
      barrel exports and unselected cookie/storage/request-language resolver modules. Broader
      platform-operation pruning and exact page delivery remain open.
      Cookie serialization and server writes now have explicit shared helper owners, composed by
      browser controls and Kit hooks instead of attached to every metadata/routing object.
      An executed Vite 8 routing-only graph excludes both transports and unselected resolver
      modules; the complete Vite 7 consumer saves 250 Brotli bytes. Runtime and installed strict
      TypeScript consumers preserve flags, appended headers, cookie-sink precedence, and finite
      locale arguments. Browser controls also omit server cookie sinks while retaining browser
      serialization, saving 107 Brotli bytes in the complete control consumer; executed Vite 8
      controls preserve preparation/publication/persistence/navigation order.
      Alternate-language link generation now has an explicit shared helper rather than methods
      on metadata, request contexts, or controls. Executed routing/control/SSR graphs omit the
      unselected SEO module; selected helpers preserve URL snapshots, request-origin reads,
      locale order, and `x-default`. Complete Vite 7 routing/control consumers save another
      83/94 Brotli bytes respectively. Generic framework controls still retain optional operations;
      keep this criterion open until their platform and page-demand acceptance passes.
      Link safety now has an explicit shared TypeScript helper owner too. Routing metadata,
      request contexts, and controls omit its methods, while runtime DOM traversal imports the
      same combined helper. Executed production routing/control/SSR graphs omit unselected
      link operations; selected-helper execution retains attribute/exclusion behavior. Complete
      Vite 7 routing/control consumers save 299/336 Brotli bytes. Single URL/environment reads,
      ordered isolated predicates, short-circuiting, and thrown error identity remain covered.
      Installed strict TypeScript consumers require explicit helper imports and retain finite
      locale and attribute types. Fourteen runtime/parity, documentation/package, and production
      site/browser gates pass; twenty-nine Node 20 runtime/hook/graph tests also pass. Broader
      platform and page-demand acceptance remains open.
      Runtime link queues now count newly accepted roots, skipping text nodes and duplicate
      records (GEN-A66). A fixture with 600 anchors visits zero elements instead of 601 after
      text additions, and one instead of 601 for repeated records on one changed link.
      Animation/timer scheduling, both overflow paths, the 256-element frame budget, ignored
      attributes, refresh, and cleanup remain covered. Vite 7/8 consumers retain one chunk;
      the cost audit records byte/compression tradeoffs. Generic framework capability and page
      delivery requirements remain open.
      All 25 web tests pass on Node 20/24, alongside thirteen runtime/parity, packaging,
      documentation, and production site/SSR/browser/graph gates.
- [x] WEB-A14 — Normalize one `LocaleSwitchPlan` shared by browser and server transports.
- [x] WEB-A15 — Use `localizeHref` for all path-based transitions.
- [x] WEB-A16 — Create an optional runtime switch handler with validated same-origin return targets.
- [x] WEB-A17 — Reject switch handlers with no server-writable transition.
- [x] WEB-A18 — Share one route matcher across resolution, redirects, links, and switching.
- [x] WEB-A19 — Diagnose local-storage-only SSR expectations.
- [x] WEB-A20 — Remove obsolete flat policy fields. Application adapters receive `$app/paths.base`;
      request/browser context owns origin and SvelteKit owns trailing slashes. Builds do not
      mutate generated runtime source to apply web policy. Runtime TypeScript and declarations
      are built into npm `dist/`, while native output contains only thin framework bindings.

### One SvelteKit/ESM ECMAScript backend

- [x] ESM-A1 — Remove `targets.ts.module` from the model and all public surfaces. A value-level
      error probe preserves its migration diagnostic without accepting it into the raw model.
- [x] ESM-A2 — Treat SvelteKit as the primary supported adapter for now; it is not the only one
      planned for the future.
- [-] ESM-A3 — Use one structured ECMAScript emitter with explicit module interfaces and
      artifact ownership for exact messages, semantic variables/forms/functions, schema types,
      locale helpers/globals, namespace modules/barrels, project entries, and thin Svelte bindings
      in their owning codegen crates; reusable web/Svelte/SvelteKit behavior belongs in npm runtime adapters. Common semantic/type/dependency planning belongs in
      `linguini-codegen`; ECMAScript rendering belongs in `linguini-codegen-ecmascript`. Executable
      syntax belongs in canonical JS templates or ergonomic, compile-checked typed AST quotation
      macros in Rust, with source identity and mappings preserved. Ad hoc JavaScript string
      assembly is not an alternative to either structured authoring path.
      At compile/build time, determine the exact messages used by each page, including its
      components, layouts, imported functions, bounded dynamic references, and all transitive
      message/function/form/formatter/plural dependencies. Emit only that dependency closure:
      no unrelated messages, symbols, CLDR data, generic dispatch machinery, or unused helpers.
      Never ship a whole message catalog as a fallback for imprecise analysis. Resolve finite
      dynamic use sets; diagnose unsupported unbounded access instead of silently broadening output.
      If a page's locale is known from its build/prerender route or SSR request, resolve it there
      and deliver only that effective locale's required messages and dependencies. Resolve missing
      translations at message granularity while building that payload; do not ship an additional
      whole base-locale payload for fallback. Do not require client JS to rediscover a known locale
      or eagerly deliver inactive locales. Retain additional locale/message payloads as lazy assets
      only where client locale switching or other actual dynamic behavior needs them.
      Where client localization code is required, coalesce its page/locale closure into one
      self-contained payload; avoid per-message requests and extra payloads for composed components
      or layouts. Emit no localization runtime/payload when server-rendered static content suffices.
      Apply OPT-A1–A8: fold known conditions, specialize used formatter/CLDR patterns, and remove every unneeded
      branch, binding, data entry, or loader from the delivered localization graph.
      Prove these constraints on real multi-page builds and served prerendered/SSR pages: inspect
      emitted module ownership, sentinel content, initial HTML/preload/response payloads, and
      browser requests before and after explicit client transitions. Preserve sparse fallback,
      synchronous SSR, typed calls, source maps, HMR, and preload-before-switch through the change.
      Every supported message must render during SSR: parameterless values, positional and
      named calls, recursive namespaces, local/inline functions, forms/plurals, formatters, finite
      dynamic references, and sparse-locale fallback. Server evaluation must use the request's
      locale, require no browser globals or client preparation, and isolate concurrent requests.
      The server-rendered text and initial client hydration must agree for all these forms.
      Existing application-module scoping is a baseline; it does not verify the full page-level
      and initial-locale delivery contract. Complete ESM-M12/M13 and MOD-A1–A8 before closing.
- [x] ESM-A4 — Introduce one shared language-neutral `TypeModel`. Public message parameters lower
  once into backend-neutral string, Boolean, numeric-input, date-input, or named types; TypeScript
  and JSDoc renderers consume that model, and signature generation no longer maps source types.
- [-] ESM-A5 — Render canonical JavaScript plus JSDoc from the shared semantic models and
      templates. Exact bundler message and semantic leaves must preserve expression evaluation,
      form dispatch, local/inline functions, call signatures, imports, demand-selected runtime
      helpers, and source mappings. Locale namespaces and barrels must preserve recursive message
      objects, callable form attributes, parameterless values, exports, defaults, and fallback.
      Project entries must retain checked-JavaScript locale metadata, finite locale types,
      directions, lazy loaders, preparation, provider, and configure contracts. Positional and
      named-object overloads, local function parameters, captured values, plural operands,
      formatter inputs/options, schema enums/aliases, and nested namespace values must retain their
      checked JSDoc types. Thin framework bindings and complete native project output must use
      the same canonical JS generation path without paired TS runtime implementations or
      per-template TS implementation branches. Reusable web/Svelte/SvelteKit runtime adapters
      use one maintained TypeScript implementation per npm package, with build-produced JS/types/maps. Emit `.d.ts` from the same type
      and signature models; preserve public documentation and source provenance on both surfaces.
      Verify exact JS/declaration/source-map snapshots, executed runtime semantics, strict JSDoc,
      and positive/negative TypeScript consumers without `allowJs`, including Node ESM, bundler,
      plain `tsc`, and real SvelteKit integration after the module/configuration migration.
- [x] ESM-A6 — Render `.d.ts` from the same `TypeModel`. Message overloads, object leaves, enum and
  alias declarations, and runtime annotations now share the sole source-type lowering function.
- [x] ESM-A7 — Maintain one JavaScript implementation per runtime family. Remove public TS
      runtime emission APIs, target switches, typed implementation fragments, and paired runtime
      templates. Keep `.d.ts` emission from the common semantic model and verify strict TS consumers.
      The source audit on `1de150f` confirms JS-only runtime compiler APIs and only JavaScript/
      declaration module kinds across the ECMAScript, Vite, and thin Svelte generators. The
      executable ownership regression rejects retired paired APIs and per-template TS branches;
      historical comparison fixtures are not production emitters. Maintained npm adapters use
      one TypeScript source implementation with build-produced JS/types. Executed generated
      output, strict JSDoc, declaration-only TS consumers without `allowJs`, native tsc build/watch,
      and installed-package consumers verify the canonical boundary. Remaining syntax extraction,
      exact page delivery, and final architecture/platform gates stay in ESM-M10–M13 and MOD-A4–A8.
- [x] ESM-A8 — Emit source maps and shared runtime helpers from the common backend; exact message
      modules plus semantic, locale-runtime, locale-global, namespace, and locale-barrel artifacts
      now have target-owned source maps, ordered source identity, and demand-selected helpers.
      Project-wide output and every web/Svelte/SvelteKit artifact compiler now return target-owned
      maps too. CLI atomic ownership includes those maps; the full ESM-M11 matrix passes.
- [x] ESM-A9 — Make parameterless public leaves values in both surfaces.
- [x] ESM-A10 — Keep JavaScript generation first-class and near-zero-config. Public JavaScript
      bundler-leaf, locale-runtime, locale-global, namespace, and locale-barrel compilers now emit
      native `.js` import and source-map paths without a TypeScript transpilation pass. Project
      entry, web, Svelte, SvelteKit, and complete project compilers do the same. Target corpora and
      consumer integrations pass the final ESM-M11 gates.

#### Emitter migration and canonical JavaScript template extraction

This is a replacement plan, not a compatibility or dual-runtime plan. Complete the stages in
order. During each stage, keep the old path only as a temporary parity oracle inside tests; delete
that slice immediately after the structured output passes its gates. Do not leave runtime feature
flags, fallback dispatch, duplicated templates, or two production emitters behind.

- [x] ESM-M1 — Freeze the migration baseline. Inventory every generated implementation,
      declaration, import edge, output path, manifest entry, source-map edge, and public Rust API;
      record byte snapshots plus semantic/runtime fixtures for TypeScript, JavaScript/JSDoc, and
      `.d.ts` before changing ownership. The canonical inventory and evidence map is recorded in
      `docs/ecmascript-backend-migration-baseline.md`.
- [x] ESM-M2 — Finish the common module model. Give one backend model explicit target language,
      imports/re-exports, type-only declarations, value statements, documentation, source spans,
      output paths, declaration companions, and deterministic rendering. Make invalid mixed-target
      states unrepresentable. `EcmaModule` now owns validated target/path metadata and accepts
      imports, re-exports, documentation, type declarations, and values through target-safe
      insertion methods; its renderer owns deterministic module framing and source-map naming.
- [x] ESM-M3 — Migrate schema-owned modules first: `shared`, recursive message types, enums,
      aliases, namespace declarations, JSDoc, and declaration output. Require exact TypeScript API
      parity plus checked-JavaScript and `.d.ts` positive/negative type tests before deleting their
      direct assembly functions. Schema roots and locale namespace declarations now use the common
      artifact model; checked `shared.js`/`messages.js`, companion declarations, byte snapshots,
      and positive plus `@ts-expect-error` negative type cases run in the quick/full test registry.
- [x] ESM-M4 — Migrate locale shared runtime helpers: argument normalization, branch selection,
      plural evaluation, number/currency/date formatting, and formatter data. Schema-owned shared
      helpers and locale-owned plural/formatter helpers now render checked JavaScript or exact
      TypeScript from common models without type erasure or export rewriting. The pinned `en`,
      `ru`, `ar`, and `hi` behavior corpus, demand selection, strict JSDoc checking, target-owned
      paths, ordered source identity, source-map snapshots, and unchanged TypeScript runtime
      snapshots pass; the direct locale runtime generator is deleted.
- [x] ESM-M5 — Migrate semantic/global leaves: locale enums, variables, forms, functions, their
      direct dependency graph, runtime-helper imports, JSDoc, and source maps. Per-symbol semantic
      artifacts and aggregate locale-global artifacts now share target-aware emitters; the old
      global assembler is deleted. Closure, cycle, collision, tree-shaking, exact helper edges,
      strict checked-JavaScript, source-map snapshots, and executable JS/TS form-function parity
      are registered quick/full gates.
- [x] ESM-M6 — Migrate physical message leaves, locale namespace modules, locale barrels, fallback
      composition, and default exports onto the same module model. Public locale artifacts now
      enumerate deterministic namespace/barrel paths and ordered fallback provenance; TypeScript
      and checked-JavaScript compilers share imports, values/callables, positional/named overloads,
      recursive namespaces, defaults, and source maps. Exact TypeScript byte parity, strict JSDoc,
      fallback execution, and JS/TS runtime parity run in quick/full gates; superseded locale
      assemblers are deleted.
- [x] ESM-M7 — Migrate project `locale` and `index` modules, eager/dynamic locale loaders, and
      public configure/prepare APIs. The old six entry templates and direct project assembly are
      deleted. TypeScript output and declarations retain byte snapshots; checked JavaScript has
      strict positive/negative JSDoc, source-map snapshots, and executable target parity. Project
      and Vite gates cover sparse locales, concurrent-load deduplication, retry, loader disposal,
      HMR, SSR boundaries, and complete pinned-CLDR locale resolution.
- [x] ESM-M8 — Migrate web, Svelte, and SvelteKit modules in dependency order: closed web feature
      leaves, link/runtime modules, locale state, effects, controls, root facade, then SvelteKit
      server adapters. Require browser/server capability gating, route/link safety, request
      isolation, streaming, Svelte checking, and production-build parity after each cutover. The
      closed web facade and every source/link/server leaf, Svelte locale/effect/control/facade
      module, and both SvelteKit server adapters now use the common backend. Superseded web and
      SvelteKit implementation templates are frozen test oracles only. TS/declaration snapshots,
      checked JS, source maps, installed-SvelteKit types, source/capability matrices, request-local
      preparation/isolation, streaming, redirect/switch safety, cookie policy, and real Svelte rune
      compilation pass. All 55 full-profile gates are verified, including packaged documentation
      after the source commit; the freshly regenerated real site passes all seven site gates.
- [x] ESM-M9 — Switch CLI output ownership, bundler manifests, Vite virtual-module resolution,
      HMR invalidation, npm examples, and VSIX consumers to structured artifacts. Require atomic
      write/cleanup parity, unchanged portable paths, truthful source maps, Vite 5–8 integration,
      real-site production graph checks, native npm CLI tests, and native VSIX tests. Project-wide
      TS/JS enumeration now shares one artifact plan, and the CLI uses its source-mapped TypeScript
      path. Source-only JS preserves namespace typedefs without declaration companions; source
      identities, map paths/content, and atomic ownership are covered. Vite 5.4.21, 6.4.4, 7.3.7,
      and 8.3.4 each pass all 46 plugin/integration tests; dynamic registries select an ES2022
      default while preserving explicit targets. Quick checks pass 44/44 and the real site's
      seven production gates pass after refreshing its local plugin dependency. Packed Linux
      x64 npm launcher/native archives pass version, init, and source-mapped build checks. The
      native Linux VSIX packages successfully; both development and extracted-archive extension
      hosts pass activation, restart, and real LSP hover checks against VS Code 1.120.0.
      Cross-platform final coverage is recorded in ESM-M11.
- [-] ESM-M10 — Finish removing direct implementation assembly: obsolete functions,
      templates, adapters, compatibility branches, tests that only exercise the removed path, and
      misleading compatibility-path terminology. Repository-wide absence checks must prove one
      production ECMAScript runtime implementation and one target renderer family. Shared schema
      dispatch/argument helpers now use one runtime body with explicit type annotations; obsolete
      schema and single-index templates are removed, and template substitution is test-only.
      Existing artifact snapshots remain byte-identical, all 150 codegen tests and strict Clippy
      pass, and executable helper parity covers fallback precedence and hostile argument keys.
      Numeric/date/digit helpers and number/currency wrappers now share runtime bodies too;
      formatter lowering has its own module, all JavaScript formatter templates are removed,
      and pinned locale behavior plus byte snapshots pass. Project locale/index entries now
      share resolution, preparation, and provider bodies too; imports, maps, and byte snapshots
      remain unchanged. Plural operands now share the common ECMAScript fragment renderer;
      the final paired runtime templates are deleted. The production source inventory contains
      no TS/JS runtime files or template includes, and an integration guard rejects their return
      alongside retired emitter names. All 151 codegen tests and strict Clippy pass; final broad
      non-regression and cross-platform gates are recorded in ESM-M11.
      The obsolete test-only schema type re-export emitter is removed; its alias test now calls
      production declaration generation. Structured import/re-export syntax shares one canonical
      module-edge template, preserving existing artifact and source-map snapshots.
      The unused `selectBranch` implementation, declarations, generic helper annotation path,
      and text-based import detection are removed. Literal helper spellings execute identically
      through ordinary and physical message/semantic modules without false runtime imports.
- [-] ESM-M11 — Rerun final non-regression gates for the completed architecture: full locked Rust
      workspace tests and strict Clippy;
      formatter/parser properties; generated snapshot and source-map review; checked JS/JSDoc and
      strict TS/`.d.ts` corpora; executable documentation; Vite 5–8; SvelteKit production; npm
      launcher; native VSIX; and Linux, Windows, and macOS CI. Close ESM-A3/A5/A7/A8/A10 and P2-1
      only after every gate passes and no direct assembly code remains. Source `1cbc4c6` passes
      all 57 local full-profile gates and strict workspace Clippy with all targets/features.
      [CI run 39110695824](https://github.com/antepodeum/Linguini/actions/runs/39110695824)
      passes all 17 jobs, including complete Linux/macOS/Windows workspace tests, Rust 1.76,
      coverage, mutation, dependency policy, public API, packaging, Vite 5–8, and SvelteKit.
      Fresh Linux npm archives compile real sources, execute generated messages, and preserve map
      provenance; the extracted native VSIX passes activation, restart, and nonempty hover with
      exact schema documentation. Canonical-root regressions and portable fixture construction
      resolve the platform failures without weakening source-map or symlink safety assertions.
      Numeric follow-up source `52bb7e3` passes 66 of 67 full-profile checks on stable 1.99.1,
      including real Chromium hydration/concurrent SSR and packaged documentation. Migration
      notes in `03a5c2c` resolve the remaining contract gate against the complete numeric change
      range, giving passing receipts for all 67 registered tasks. Rust minimum verification uses
      the independently discovered 1.87 toolchain. These are local receipts; the final architecture/
      platform matrix and open acceptance criteria still require completion.
      Adapter-ownership source `fcd6cd9` passes all 69 local full-profile gates, including locked
      workspace tests, strict Clippy, strict generated JS/TS corpora, packaged documentation,
      native CLI/tsc consumers, editor tests, and all nine real-site checks. Core, Svelte, and Vue
      packages also pass 80 tests per Vite major (5–8), plus the same 80 tests on Node 20.
      These local receipts do not close the final cross-platform or universal-WASM criteria.
      Follow-up source `a8fa133` passes all 70 local full-profile gates, including locked workspace
      tests/strict Clippy, generated JS/TS consumers, native CLI/tsc integration, packaged docs,
      editor tests and real shared-WASM Node/browser protocol checks, and all nine site gates.
      This run does not establish a fresh installed VSIX smoke, remote CI, signing/publication,
      or the final Windows/macOS matrix.
      Plural/module-edge/cookie-ownership follow-up passes 69 of 70 local full-profile gates,
      including workspace tests/Clippy, packaged docs, native consumers, shared WASM hosts,
      and all nine site gates. Updating the current npm parity consumer to the explicit cookie
      helpers repairs the remaining gate: all 6,270 assertions pass with frozen oracles unchanged.
      The strengthened 23-test Kit gate and Node 20 web/hook/graph tests pass separately, giving
      passing receipts for all 70 registered gates. Final architecture/platform criteria remain open.
      Source `0143d99` also passes 80 core/Svelte/Vue adapter tests per Vite version 5.4.21,
      6.4.6, 7.3.8, and 8.3.8. Dependency manifests/locks and pinned installs are restored afterward.
      Fresh optimized Linux native and universal VSIX archives pass installed VS Code 1.120
      Extension Host checks in native, explicit WASM, and automatic fallback modes. Automatic
      fallback runs with an empty `PATH`; hover, indexed definitions, Unicode/Markdown highlighting,
      edits, restart, and native launcher arguments pass. Archive/WASM checksums verify, and the
      native archive's binary matches the fresh release build; the universal archive has none.
      These local integration receipts leave remote CI, signing/publication, final architecture,
      and the Windows/macOS completion matrix open.
      CLDR/preview follow-up source `9e91a66` passes all 70 local full-profile gates in one run:
      locked workspace tests, strict Clippy, generated JS/TS corpora, both documentation build
      modes, native CLI/tsc integration, editor and shared-WASM hosts, and all nine site gates.
      Separate Rust 1.87 and Node 20 checks cover the formatter/preview changes. This source
      receipt preserves the open architecture, fresh installed-VSIX, remote-platform, and
      signing/publication requirements above.
      Follow-up source `ef880d8` passes 68 of 70 full-profile gates in one run, including locked
      workspace/Clippy, packaged documentation, strict JS/TS/tsc, npm/editor/shared-WASM, and
      production graph/browser checks. Two test consumers still called the retired `selectBranch`
      export. Updating those consumers preserves named-argument parity and executes prototype-key
      rejection through the site's generated dispatch; both repaired gates pass on Node 24 and
      Node 20. Together these receipts cover all 70 gates without claiming a fresh single clean
      full run, installed VSIX smoke, remote-platform run, or final architecture completion.
      Shared-planning/provenance source `8b5c107` passes all 70 local full-profile gates in
      one clean run: locked workspace tests/Clippy, strict generated JS/TS corpora, packaged
      documentation, native CLI/tsc integration, editor/shared-WASM hosts, and all nine site gates.
      Separate Rust 1.87 and Node 20 checks cover both common-layer extractions. This receipt
      leaves the open architecture, installed-VSIX, remote-platform, and release criteria intact.
      Formatted-input/preview follow-up source `3429c97` also passes all 70 local full-profile
      gates in one clean run, including native tsc integration, packaged sources, editor/WASM,
      and all production site checks. Separate Node 20 and Rust 1.87 verification covers the
      changed paths. This source receipt does not close the remaining architecture/platform work.
      Locale-identity, constant-plural, ISO date-time preview, and CI-host follow-up source
      `abc80fc` passes all 70 local full-profile gates in one clean run. This includes native
      TypeScript build/watch integration, packaged-source documentation, editor/WASM, and all
      production-site graph/browser checks. Separate dependency-policy probes, offline crate
      packaging, Node 20, and Rust 1.87 receipts cover the changed boundaries. Remote
      Windows/macOS verification of the CI fixes and the broader architecture remain open.
      Shared date/number-plan, readable-binding, and exact test-selection follow-up source
      `557d66b` passes all 70 local full-profile gates in one clean run, including native
      build/watch integrations, packaged sources, editor/WASM, and production graph/browser
      checks. Runtime fixture selection no longer includes archived copies under `target/`.
      Separate Node 20 and Rust 1.87 checks cover the changed compiler/preview boundaries.
      This receipt does not close the remaining architecture or release criteria.
      Explicit link-helper and locked local-package refresh source `1dcecc1` passes all 71 local
      full-profile gates, including its new cold/warm installer regression, native watch/build,
      packaged documentation, editor/WASM, and production graph/browser checks. Separate
      Node 20 runtime/hook/graph and installer tests plus `actionlint` cover the changed paths.
      Hosted verification and the remaining architecture/release criteria remain open.
      Parent-aware deferred-demand source `55de518` passes all 71 local full-profile gates,
      including installed TypeScript consumers, native build/watch, editor/WASM, and production
      graph/browser checks. Separate Node 20 tests cover the changed planner/linker boundary;
      all 45 existing production locale payloads retain identical paths and bytes. Differing
      unprepared demands, unsupported ownership shapes, and final architecture gates remain open.
      Automatic form-demand source `ea82945` passes all 71 local full-profile gates, including
      the new native cold/warm form-payload regression, strict generated consumers, editor/WASM,
      and production SSR/browser checks. Rust 1.87, Node 20, workspace Clippy, and complete-consumer
      measurements separately cover the shared plan. Attribute-level page delivery, conservative
      callable/eager forms, hosted verification, and the remaining architecture criteria stay open.
      Independent form-export source `23271e5` passes all 71 local full-profile gates, including
      the shared Vite 7/8 native page contract and strict physical-JS/application-TS consumers.
      Separate Node 20, Rust 1.87, workspace Clippy, and patch-level API checks against `ea82945`
      cover the changed compiler boundaries (223 compatibility checks pass per changed crate).
      Opaque forms, mixed-table coalescing, hosted
      verification, and remaining architecture/release work stay open.
      Function-scope source `1de150f` passes all 71 local full-profile gates, including runtime
      ownership, strict generated JSDoc/TypeScript, installed/native tsc build/watch, editor/WASM,
      packaged documentation, and production SSR/browser checks. Separate Node 20, Rust 1.87,
      workspace Clippy, and patch API checks pass; six complete consumers retain the same output
      and minified/gzip/Brotli sizes on Vite 7/8. This verifies ESM-A7's single-runtime completion;
      native qualification scopes and the remaining architecture/platform requirements are separate.
      Inherited-source/editor source `6c32866` passes 70 of 71 full-profile gates, including all
      Rust tests, strict generated consumers, installed tsc build/watch, editor/WASM, and production
      SSR/browser checks. The packed Vue install alone failed because `/tmp` selected a different
      default pnpm store with no cached `smol-toml` tarball. Its consumer now explicitly reuses the
      package's store, as the Kit packaging test already does. The failed Vue gate then passes all
      five tests offline, and its packed consumer passes separately on Node 20. This is a full-run
      receipt plus a targeted repair/rerun, not a second all-green full run or hosted-CI verification.
      Integrated source `da0567b` passes all 71 local full-profile gates in one clean run,
      including the repaired offline packed Vue consumer, typed native Date previews, named-call
      editor identity, compiled Gregorian year/week output, installed tsc build/watch, editor/WASM,
      documentation consumers, and production SSR/browser checks. Node 20, Rust 1.87, strict
      JS/TS 5.8/5.9/6, and patch API checks separately cover the changed formatter boundaries.
      Hosted CI and the remaining architecture/platform requirements stay open.
      Integrated first-weekday/private-function source `3589545` also passes all 71 local
      full-profile gates in one clean run, including rebuilt Node/browser WASM, installed tsc,
      packaged/offline consumers, documentation, and production SSR/browser/graph checks.
      Integrated helper-typing/private-variable source `995242a` also passes all 71 local
      full-profile gates in one clean run, including strict generated consumers, installed/native
      tsc, rebuilt editor/WASM, offline packages, and production SSR/browser/graph checks.
      Integrated public export-binding source `886c47f` also passes all 71 local full-profile
      gates in one clean run, including native namespace regressions, installed tsc, editor/WASM,
      offline packages, documentation, and production SSR/browser/graph checks (GEN-A53).
      Integrated month-week source `42ffd3e` also passes all 71 local full-profile gates in one
      clean run, including native cold/warm/preview regressions, strict date consumers, installed
      tsc, editor/WASM, offline packages, documentation, and production SSR/browser/graph checks.
      Integrated constant-family/public-classifier source `8e2beb6` also passes all 71 local
      full-profile gates in one clean run, including native week-date parity/builds, strict custom
      classifier consumers, installed tsc, editor/WASM, offline packages, documentation, and
      production SSR/browser/graph checks (GEN-A55–A56).
      The integrated programmatic type-keyword and inherited-navigation working tree also passes
      all 71 full-profile gates after the shared-body rename guard (GEN-A57–A58): locked Rust,
      strict consumers, installed/native tsc, rebuilt editor/WASM, offline packages, documentation,
      and production SSR/browser/graph checks. Broader unfinished architecture criteria stay open.
      The qualified native-date annotation follow-up also passes all 71 full-profile gates
      (GEN-A59), including strict generated consumers, installed tsc, rebuilt editor/WASM,
      offline packages, documentation, and production site/SSR/browser/graph checks.
      The integrated week-validity, date-composition, plural-preview, and digit-callback changes
      (GEN-A60–A64) also pass all 71 local full-profile gates. Separate Node 20/24 formatter and
      CLI suites, Rust 1.87, and strict JS 5.8/5.9 checks pass. Broader architecture and hosted
      platform/release verification remain open.

- [-] ESM-M12 — Move executable output syntax out of ad hoc Rust strings into canonical
      JavaScript templates or ergonomic, compile-checked typed AST quotation macros in Rust.
      Declarations must use the same shared semantic/type models; no duplicate JS/TS runtime
      implementations. Macro-authored source is allowed; checked-in generated runtime is not.
      Rust must resolve used CLDR patterns and semantic requirements during builds;
      templates emit specialized functions and only required helpers, with no runtime CLDR
      pattern interpreter or checked-in generated runtime sources. Schema dispatch, numeric,
      date, digit, plural-operand, and locale-provider helpers have extracted templates.
      Physical message exports, namespace aliases, value re-exports, and imported JSDoc typedefs
      also use shared module templates, with byte/map snapshots preserved. Declaration exports,
      overloads, callable signatures, and JSDoc now use shared templates; locale and schema message
      types share one provenance-aware namespace renderer. Web aliases, interfaces, generic
      parameters, inheritance, methods, and member types now live in one shared template model
      used by both JSDoc and declarations, preserving generated source/map bytes. Complete the
      remaining type-renderer audit and demand-selected CLDR helper generation. Primitive and
      named types plus enum literal unions now share one type-expression template; schema and
      locale JSDoc, declarations, and selector annotations reuse its escaping and type semantics.
      Current runtime/declaration/map snapshots remain byte-identical.
      Structured imports/re-exports now use one module-edge template and one named-binding
      renderer for value, type, and mixed lists. Executed native ESM checks cover side-effect,
      default, namespace, named/aliased and empty imports, escaped Unicode specifiers, re-exports,
      dependency initialization count, and omitted missing type-only modules. Declaration tests
      retain type-only/mixed edges; production schema alias coverage replaces its retired emitter.
      Static CLDR parameters must specialize the emitted implementation itself; passing a long
      vector of constants/undefined options to a generic formatter does not satisfy this criterion.
      Validated named/inline function, callable-form, and plural results omit outer coercion too;
      their string-result fact belongs to shared semantic analysis. Executed generated code
      preserves raw input conversions, eager captures, and exceptions. Form objects and nested
      dispatch bodies use consistent multiline indentation while preserving dispatch semantics.
      Generated number/currency/date results also omit redundant outer `String` calls while
      preserving raw input coercion, aliases, evaluation order, and formatter exceptions.
      No generated JS/TS runtime sources belong under Rust `src/`; generation happens during builds.
      Backend-independent type models, requirement analysis, and message dependency slicing
      live in `linguini-codegen`; ECMAScript rendering stays in `linguini-codegen-ecmascript`.

- [-] ESM-M13 — Emit one JavaScript runtime plus declarations from the shared semantic plan.
      Remove TypeScript implementation generation and per-template TypeScript runtime branches.
      Preserve strict TypeScript consumer typing without `allowJs`; integrate plain `tsc`
      builds so generated JavaScript and owned companions reach the emitted application.
      Keep bundler integration and source provenance covered through the transition.
      Runtime generation now emits canonical JS and adjacent declarations. Native `build
      --emit-dir` renders additional owned output trees with rebased source maps; declaration
      destinations prepare types while preserving prior runtime. `@antepodeum/linguini-tsc` uses
      public compiler/build/watch APIs and covers strict consumers, incremental references,
      message-only updates, separate declaration output, and an installed package.
      Linux integration tests pass with TypeScript 5.8.3 and 6.0.3; the cross-platform CI matrix
      is configured but requires a run against the final migration. Complete
      module/configuration migration and the final bundler/framework/platform matrix before
      closing this transition.

### Reusable backend and framework modules

Runtime-boundary correction: web operations and SvelteKit server handling belong to reusable
npm runtime adapters, configured by application code. The earlier Rust web and
SvelteKit generators have been removed. Rust owns message compilation, CLDR
specialization, types/maps, and dependency planning; routing/persistence settings must not change
those artifacts. Keep runtime libraries independent of generated catalogs and preserve SSR request
isolation. Verify policy-only edits without a native rebuild, package consumption, hook behavior,
and the composed payload/SSR requirements below.

- [x] MOD-A1 — Separate message compilation, build integration, and reusable runtime adapters
      with explicit ownership and minimal dependencies. `linguini-codegen` owns shared semantics,
      `linguini-codegen-ecmascript` owns canonical JS/declarations, `linguini-codegen-vite` owns
      physical message leaves/manifests, and `linguini-codegen-svelte` emits only project-specific
      imports/types. Remove the Rust web/SvelteKit generators and generic runtime templates.
      TypeScript npm packages `@antepodeum/linguini-web`, `@antepodeum/linguini-svelte`, and
      `@antepodeum/linguini-sveltekit` own reusable platform/framework behavior. SvelteKit composes
      Svelte and web; its `/vite` entry composes `@antepodeum/linguini-svelte/vite`.
      Framework adapters are TypeScript in their owning npm packages; Vue owns its SFC adapter
      and React uses the shared core directly. Native ownership
      tests enforce the boundary; real rune compilation, packed strict consumers, and the site
      production build exercise the composition. Exact page delivery remains MOD-A4–A5.
- [x] MOD-A2 — Keep Vite's build, watch, module graph, exact-message selection, locale loading,
      and source-map infrastructure framework-independent. Isolate Svelte syntax/link transforms
      and SvelteKit route/SSR behavior behind adapters that reuse this infrastructure. The JS Vite
      core accepts explicit framework capabilities. `@antepodeum/linguini-svelte/vite` owns script
      scopes, static links, and component cleanup; SvelteKit owns route/SSR behavior, and Vue owns
      SFC scope discovery through Vue's compiler. The core has no framework exports or production dependencies.
      Native helper descriptors cover thin Svelte bindings; application modules configure runtime
      policy and static-link helpers explicitly. Core/framework/packed tests pass Vite 5–8.
      Exact composed-page delivery and broader request-selected hosts remain MOD-A4–A5.
- [x] MOD-A3 — Keep runtime web policy in application code, configured where hooks and runtime
      adapters are created. Publish reusable `@antepodeum/linguini-web` and
      `@antepodeum/linguini-sveltekit` npm packages. URL/routing policy, locale-source ordering,
      cookie/storage persistence, request context, and SvelteKit hooks must not be generated by
      Rust or transported as a native module plan. Changing these settings must leave native
      message artifacts unchanged. Remove the `web()`/`svelteKit()` codegen configuration DSL
      and its project-config worker/transport machinery; ordinary frontend module loading owns
      runtime configuration. Keep `[web]` out of neutral TOML. Preserve typed ECMAScript message
      bindings and exact dependency selection, with thin integration bindings only where needed.
      The policy DSL, worker, transport, and native flags are removed. Executable documentation
      builds every documented hook variant, and launcher/project tests prove root policy files
      are not evaluated. Runtime package and site tests exercise policy without a native rebuild.
- [-] MOD-A4 — Enforce ESM-A3's exact page/locale delivery contract across composed components,
      layouts, shared functions, and finite dynamic uses. Build/SSR must select a known initial
      locale without client rediscovery; initial delivery must contain no unused messages/helpers,
      inactive locales, or extra base-locale catalog. Resolve sparse fallback into the exact payload.
      Where client code is needed, deliver one self-contained page/locale file; load additional
      payloads only for actual client transitions. Verify output graphs and initial/transition
      browser requests, not just isolated application entries or the existence of lazy chunks. The production
      browser gate now proves French SSR bootstrap fetches only the exact French route payload,
      then one English payload on an explicit switch; JavaScript-disabled rendering needs neither.
      The shared SvelteKit root load also tracks the original URL, preventing reuse of stale
      locale metadata. Real built-server data responses declare that dependency; packed strict
      consumers retain locals-only compatibility. A composed fixture now proves exact layout,
      child-component, and imported-function demand, including removal of a shared function used
      only by another page. Production output has one self-contained payload per page/locale;
      Chromium verifies preparation before cross-page/cross-locale publication, cached revisits,
      and no locale requests with scripting disabled. All four source-map modes pass. Lazy
      actions now prepare one additional selected-locale closure before module evaluation. The full
      framework/configuration matrix remains open. Deferred reads of already prepared messages
      now execute in Chromium without another message request. Demand analysis drops unused lazy
      exports, including a shared export used by another page, on both local Vite 7 and Vite 8.
      A setter-only native-output/SDK/Vite regression now emits one chunk with no message
      catalog modules or dynamic catalog imports, replacing a ten-chunk nine-locale graph.
      Generated locale helpers own the shared setter independently of message providers;
      execution and request-identity tests retain normalization and stale-change guards.
      Deferred planning now re-shakes each lazy entry's consumed exports independently.
      One shared contract runs against Vite 7 and the site's Vite 8, covering nested boundaries,
      pure/default re-exports, eager/lazy shared helpers, required module effects, escaping
      namespaces, dead lazy exports, and ES2018 output targets. Analysis-only async facades
      are not emitted into application output. Additional payloads exclude initial messages and
      unused exports; identical closures share assets across pages. Chromium proves one lazy JS
      request plus one locale file on first action, zero on repetition and cross-page reuse, and
      no additional payload before the action. Preparation lives in owning page nodes or one
      shared runtime, outside unrelated initial page graphs. Late lowering preserves native
      namespace identity, live exports, and prepare-before-evaluation. Inlined imports, callers
      shared with unrelated pages, and distinct additional demands from one lazy module on a
      page fail explicitly; exact lowering for those cases remains open.
      Root loader selection also follows build demand. Without additional closures, a Vite 8
      emitted-program regression matches the direct base loader byte-for-byte and excludes
      deferred callbacks and branches. SSR always selects the base loader; enhanced navigation
      shares payload loading and retains metadata-read counts without an extra async wrapper.
      Nested import preparation now follows build-only page-specific targets, including transit
      modules, without traversing global framework route loaders. Parent availability intersects
      all incoming paths, handles cycles conservatively, and leaves direct paths independent.
      Compiled child payloads omit guaranteed ancestor records; transit-only modules emit none.
      Executed Vite 7/8 linker cases cover both parent shapes and preserve native binding identity.
      The real site checks exact parent/child record sets across nine locales; Chromium proves
      one child-code request plus one additional locale file, cached repetition, and locale changes.
      Namespace message roots now retain exact native import identities and leaf spans, including
      callable/method tails and finite computed access. Separate metadata/control exports preserve
      their import without retaining unused catalog values; unknown roots, catalog/provider exports,
      and namespace escapes fail explicitly. The site's nested lazy module uses a namespace import
      while retaining its one-record child payload, cached action requests, and concurrent SSR.
      Export-variant compatibility also uses those guaranteed-parent facts. Different already
      prepared reads can share one remaining closure, or emit no extra file when none remains.
      Shared Vite 7/8 execution checks exact initial/parent/additional record sets and sparse
      fallback; partial parents, independent direct paths, and cyclic self-justification still
      fail. A linked two-locale owning-page program verifies prepare-before-initialization,
      one shared additional payload, repeated-import caching, native binding identity, and
      shared module state across locale changes. Existing ownership guards remain intact.
      Full Vite/Kit suites, thirteen focused Node 20 tests, and ten documentation/package/site
      gates pass. All 45 existing production locale payloads retain identical paths and bytes.
      Default/mixed declarations remain outside this receipt.
      Message/link/lifecycle insertions also share quote-aware component tag boundaries. Real
      Svelte client/server compilation covers generic constraints containing `>` and `module`,
      script-like HTML/CSS comments, and separate module/instance scripts without extra scripts.
- [-] MOD-A5 — Support SSR for every message form listed in ESM-A3, including finite dynamic
      references and sparse fallback. Resolve each request's messages/locale on the server without
      browser APIs or client-only preparation; isolate concurrent requests and prove matching
      hydration text. Preserve preload-before-switch, loader cleanup on navigation, HMR, source
      maps, typed calls, and browser/server cookie/routing policy through the migration. Verify
      both direct SSR evaluation and real framework rendering/hydration across the message matrix.
      Native unnamed form calls now pass shared analyzer validation instead of failing as unknown
      functions (GEN-A41). Alias and named/inline receiver scopes, explicit plural inputs, bad
      arity/types, incomplete variant implementations, and named-function precedence are covered.
      Native check/build, executed named/positional calls, previews, and failed-build output
      preservation verify the actual CLI path. Broader SSR and delivery criteria remain open.
      Classified `Plural` results also forward into plural function/form parameters instead of
      failing IR validation after a successful analyzer check (GEN-A43). Native check/build/preview
      and ordinary/physical English/Russian execution cover named/forwarded/inline functions,
      named/unnamed forms, and positional/named calls. Negative tests keep categories distinct
      from numeric parameters/intrinsic inputs and retain previous output on failed builds.
      This is a focused semantic correction; the full framework/SSR matrix remains open.
      The built SvelteKit server now renders concurrent French/Russian/English requests with values,
      recursive namespaces, named/positional calls, forms/plurals, and number/currency/date formatters.
      Chromium verifies French hydration text and locale switching. The production message lab
      executes finite computed values, captured computed functions, named calls, and base-only
      message fallback in concurrent French/Russian/English SSR and French/English hydration.
      Its exact five-message closure is checked across all nine locales, without an extra base
      payload. Live SSR preloads through HTTP Link hints; Chromium verifies preparation before
      framework entry execution and no message requests with scripting disabled. The full
      sparse-fallback and framework/configuration matrices remain open. Composed-page hydration
      and cross-page/cross-locale navigation execute in Chromium. Live SSR also covers the
      trailing-slash redirect-loop fix; prerendering alone did not expose that failure.
      Client controls now invalidate superseded, request-replaced, and disposed changes across
      preparation/persistence/navigation waits. Runtime tests prove older navigation completions
      cannot clear a newer override. Chromium holds an English preload across navigation to a
      Russian page, then completes it without another navigation, locale publication, or payload.
      Direct message-provider setters also avoid publication from superseded or request-replaced preparation.
      The Kit wrapper supplies identity automatically; an executed Vite consumer verifies that
      a same-locale request replacement still invalidates the older operation. Server provider
      tests forbid reading the client identity callback.
      Server calls now read request-local locale metadata through Kit request events rather than
      component context. Concurrent built-server loads execute lazy values and named number
      functions in French/Russian/English; Chromium verifies matching hydration and no-JS HTML
      without a deferred locale request. Async request access is verified on the Node platform;
      other runtime capabilities remain part of the platform matrix. Deferred runtime tests cover
      retries, overlapping locale changes, request replacement, and private incoming preparation.
      Concurrent nested server loads now read parent values, a named number formatter, and a new
      child message in French/Russian/English. Hydration preserves their server-rendered strings,
      and neither script-enabled nor no-JS rendering requests a deferred locale file for that data.
      Native source validation now also accepts zero-selector functions with named payloads,
      matching existing IR/codegen behavior. CLI execution covers zero arguments, positional/named
      strings, nested calls, inline captures, Boolean/numeric/Date payloads, and rejected invalid
      bodies without output mutation. LSP diagnostics agree, and non-String payloads retain no
      misleading `fn_without_strings` warning. Native Vue/React SSR consumers use a String helper;
      the real site's Date helper retains its existing exact payload and nine-locale SSR/browser
      checks. This receipt does not close the remaining delivery/SSR acceptance above.
- [x] MOD-A6 — Prove that Vite core can be reused outside Svelte/SvelteKit with framework-neutral
      JS/TS consumers and representative Vue/React integration boundaries. Add only the adapters
      needed for these boundaries; do not implement unrelated backends or speculative frameworks.
      The same core now passes real Vite multi-entry graph and executed-output checks with ECMAScript and custom non-Svelte entry/runtime descriptors.
      The native `lgl` manifest now works without runtime helpers; direct and aliased bindings
      survive cold/warm native builds, and eager/dynamic Vite builds execute values, functions,
      and finite computed access with zero rendered facade bytes and no inactive-locale imports.
      Native configuration is framework-neutral. Real React TSX and Vue normal/setup scripts,
      interpolations, and bound attributes now execute client/SSR parent-child builds against
      cold/warm native manifests. Each composed page retains its two used messages and function
      dependency in one chunk, with no unused message, inactive French data, or locale requests.
      `@antepodeum/linguini-vue/vite` uses Vue's parser; React needs no additional adapter.
      Fresh packed consumers preserve strict TypeScript without `allowJs`, source maps, and the
      removal of the old Vite `/svelte` export. All 80 core/framework tests pass on Vite 5–8.
      These representative base-locale boundaries do not close request-selected Vue/React hosts
      or the broader page/locale acceptance in MOD-A4–A5.
- [-] MOD-A7 — Share common lowering, dependency planning, and type semantics through
      `linguini-codegen`; keep ECMAScript syntax, web platform behavior, and framework dependencies
      within their owning backend/modules. Future language backends must not depend on web policy.
      `ProjectLocalePlan` now owns project fallback matching and finite locale-resolution table
      planning in the common crate. Exact names, aliases, CLDR parents/likely scripts, and base
      precedence have shared tests; current generated code/declaration/map snapshots are unchanged.
      ECMAScript owns only the locale artifact projection and syntax for this boundary.
      The matching implementation is now owned by the neutral CLDR service. The established
      `linguini-codegen` type forwards to that implementation without changing its Rust identity
      or signatures; source tools can share the plan without a code-generation dependency.
      Locale declaration overlays also use common `compose_locale_modules`: all six locale
      declaration families retain whole-declaration precedence, stable order, source identities,
      and complete override history. The backend retains schema validation after composition;
      current code/declaration/map snapshots remain unchanged.
      `NumberFormatPlan` also moves affix expansion, grouping thresholds, scale shifts,
      precision/rounding, scientific symbols/carry proofs, and padding into typed common models.
      The backend retains helper naming, escaping, and template projection. Shared tests cover
      reachable pinned number/percent/currency patterns and explicit boundary plans; generated
      code, declarations, and maps remain byte-identical.
      Schema/locale selection, namespace projection, and nested message-tree semantics now also
      live in the common crate. Shared tests retain path boundaries, declaration order, group docs,
      source identities, and globals. Executed ordinary imports and project factories prove that
      empty namespace reservations create neither values nor declaration leaves/transit modules,
      with declarations and tree shaking enabled or disabled. Current snapshots stay unchanged.
      Form/function dispatch traversal now also uses common plans with build-time selector/text
      projections. Shared tests cover full selector indices, named payloads, missing-dimension
      precedence, multi-key/default forms, projected-result merging, and retained selector
      evaluation. Existing backend eager/lazy/capture/error cases and byte-identical snapshots
      pass, alongside strict consumers, runtime parity, Rust 1.87, Clippy, and source packaging.
      Semantic artifacts and transitive dependency slices now share recursive source-provenance
      collection. Mixed-source tests cover calls, both inline input kinds, nested dispatch,
      map/object/direct form branches, deduplication, and effective override origins. Current
      generated code/declaration/map snapshots, native CLI tests, strict consumers, runtime parity,
      Rust 1.87, Clippy, and packaged-source documentation checks pass.
      Scoped `expression_value_type` now also unifies argument/inline result inference across
      dependency walkers, plural demand, inline lowering, and native previews. Explicit
      formatters produce strings; forwarding without an annotation retains its source type.
      Native project-IR loading/validation is also separated from ECMAScript projection and
      reused by build and previews. Existing merge/provenance/collision assertions move intact
      with their IR owner. The common locale plan is cached once per load and each fallback
      chain once per locale, rather than re-canonicalizing declared tags for every namespace.
      Physical/standalone message lookup also prioritizes exact spellings and uses matching
      locale-runtime plural bindings. Executed ordinary/physical/standalone alias cases retain
      canonical metadata, case-insensitive and canonical-only requests, and source maps, while
      rejecting undeclared locales. Native manifests retain the selected parent's source identities.
      The affected common/backend/CLI/Vite tests pass on Node 20, existing snapshots stay unchanged,
      and workspace Clippy plus Rust 1.87 pass. Eighteen consumer/documentation/site gates pass,
      including packaged sources, native tsc build/watch integration, and production SSR/browser
      checks. This follow-up preserves the earlier full-run source receipt separately.
      Shared `DatePatternPlan` now resolves selected field symbols, deduplicated day periods,
      semantic field demand, and whole-result digit-localization proofs for both ECMAScript
      emission and native previews. Codegen validation checks missing symbols before emission.
      Context, repeated-plan, quoted-field, literal/symbol-digit, and structured-error tests cover
      the common model, including over 10,000 selectable skeletons across all 508 resolved
      Gregorian records. Existing target bindings, byte/map snapshots, and date/preview semantics
      stay unchanged; extended calendars/zones/skeleton matching remain open. Common/backend/CLI
      suites pass on Node 20/24, with workspace Clippy, Rust 1.87, nineteen consumer/documentation/
      production-site gates, and final packaged-source checks passing. Explicit patch-level API
      checks pass 223 checks per affected public crate; existing error enums remain compatible.
      Native source analysis now shares inherited whole-global context without copying messages
      into translation coverage. The same fallback chain feeds check/build/preview; qualification
      sees composed declarations while retaining guarded merge/provenance. Four-locale execution
      covers CLDR-parent priority, local overrides, named/positional calls, source IDs, related-file
      diagnostics, warning promotion, and preserved output on invalid builds. Shared Vite 7/8 tests
      emit 12 exact payloads with inherited French globals and strict JS/TS consumers (GEN-A44).
      Native and memory editor hosts now attach declared fallback chains from the shared CLDR
      service, resolved only for indexed locales and reused across namespaces. Diagnostics retain
      inherited signatures across unsaved fallback edits, close/revert, and configuration changes;
      ambiguous sources fail closed and translation coverage stays local. All 63 LSP tests pass
      on Node 20; actual built Node/browser WASM workers pass the same fallback checkpoints.
      Workspace Clippy, Rust 1.87 native/WASM, and patch-level API checks pass. Broader
      backend/architecture criteria remain open. Inherited variable/function and locale enum type
      navigation now shares that declaration selection and original ownership (GEN-A58), including
      rejection of renames that would change a shared body's binding in another locale. All 71
      LSP tests, built Node 20/browser WASM protocol, Clippy, Rust 1.87 native/WASM, and 223 patch
      API checks per affected crate pass. Typed form-property navigation remains separate.
- [-] MOD-A8 — Update public APIs, migration notes, examples, the real site, package boundaries,
      and CI. Verify canonical JS + `.d.ts`, strict TS without `allowJs`, plain `tsc` build/watch/
      project references, Vite 5–8, and SvelteKit production with the new configuration and modules.
      Public adapters, executable documentation, and the site use application-owned runtime policy;
      the temporary native policy transport and SDK entries are removed. All npm
      packages use `@antepodeum/linguini` / `@antepodeum/linguini-*`; release tooling packs the
      shared frontend package and platform packages before the launcher. Local Vite, strict tsc/packed APIs,
      site production, and fresh native generation pass. Cross-version/platform and composed-page
      delivery/SSR verification remain open.
      A newer macOS [run 45866332671](https://github.com/antepodeum/Linguini/actions/runs/45866332671)
      on `93a8d78` timed out waiting for a TypeScript watch locale edit. Known source/configuration
      files now use adaptive priority polling independently of OS event delivery; directories
      retain namespace rediscovery. A regression that opens event watchers but drops every event
      fails before the change and passes after it on Node 20/24 with TypeScript 5.8.3 and 6.0.3.
      The full native TypeScript suite, including packed installation, references, watch/error
      recovery, namespace changes, and disposal, passes locally. The real watch/solution-watch
      lifecycle also passes on Node 20. Run 45868820891 on `3ec4923` now passes the complete
      remote matrix, including macOS/Windows TypeScript watch integration and Rust tests.
      Generated application code has no added watcher or polling runtime; exact composed-page
      delivery and final architecture-wide verification remain open.
      Site descriptions across all nine locales now distinguish generated JavaScript plus
      declarations from reusable web/Svelte/SvelteKit adapters and application-owned policy.
      They describe message-level fallback and preparation before reactive publication;
      all nine site gates pass, including production graphs and browser/no-JS checks.

### Specialized generated code

The [fixture review](docs/generated-code-review.md) records concrete remaining generator
findings, their current owners, and semantic/output verification requirements. Its open items
remain part of OPT-A1–A6 and MOD-A4–A5; the inventory is not a completion receipt.

- [-] OPT-A1 — Parse and resolve all used CLDR number, currency, date, calendar, context, zone,
      skeleton, plural, and composition patterns during compilation/build. Generated code must be
      the compiled result of those patterns; it must not carry raw CLDR patterns/tables for a
      runtime interpreter or parse CLDR syntax after generation. Parsing actual input values such
      as numeric strings or dates is a separate, demand-selected runtime operation.
- [-] OPT-A2 — Emit specialized formatter implementations for the actual statically used requests.
      Fixed/flexible day-period rules and label fallback now lower through the shared build-time
      plan into direct conditions; executed coverage checks 483,840 locale/width/time combinations
      against raw pinned rules, plus display precision and missing-label/merged-span cases.
      Named weekday fields preserve formatting versus standalone contexts and selected widths;
      executed Finnish/Inari Sami full requests replace the erroneous `UTC` weekday placeholder.
      Shared date-pattern compilation validates CLDR quoting/field widths before output emission;
      unsupported fields produce structured errors instead of a fabricated UTC value. Selected
      UTC names, missing-name GMT-zero fallback, and ISO-zero forms fold into merged literals.
      Eight-locale generated execution and the 508-record resolution-graph style/composition
      scan verify this boundary; selectable non-UTC zones, extended calendars, and non-exact
      skeleton matching remain open.
      Gregorian year-of-era conversion and width padding now cover 18 BCE/CE year samples and
      eight widths, sharing the UTC year read and selecting year-only padding helpers.
      Numeric affix lowering now resolves unquoted sign/percent/per-mille placeholders from
      pinned symbols before discarding quote provenance. Localized implicit negative patterns
      retain minus signs and bidi marks; quoted scaling symbols remain literals. Executed decimal,
      negative-zero, percent-pattern, standard-currency, and accounting regressions cover this
      correction without adding a runtime CLDR parser or changing the exposed style options.
      Minimum grouping digits now compile into an integer-length threshold, covering the 35
      pinned two-digit-threshold locales and two three-digit-threshold Ewe locales. Executed
      numeric/currency rounding boundaries and 96 width/padding/threshold plans include Unicode
      separators, negative zero, and 8,192-digit inputs. Existing default-threshold snapshots stay
      byte-identical; native previews apply the same threshold after rounding.
      Native preview currency symbols now use the existing pinned locale lookup rather than
      a fixed nine-code map. Eleven locale/code cases cover positive/negative values, local
      symbol variants, lowercase codes, and uppercase unknown-code fallback.
      Fold grouping sizes, symbols, precision, rounding, digit mapping, affixes, signs, calendar,
      field widths, and other known choices into code. Remove unused options/branches/data; prohibit
      the current `formatNumber(value) -> formatGeneratedNumber(value, ...21 constants...)` design.
      Keep semantic plans reusable and syntax in canonical templates; avoid recreating a generic
      formatter interpreter under a different name or moving the same parameter vector elsewhere.
- [x] OPT-A3 — Lower forms and local/inline function dispatch into direct control flow (`if`/`switch`
      and returns) from the validated semantic tree. No per-call branch-map objects, closure tables,
      `selectBranch` callbacks, nested dispatch IIFEs, or wrapper casts solely to support that design.
      Merge identical branches/defaults, remove known redundant coercions, and preserve exact
      evaluation count/order, captured values, callable attributes, plural semantics, and fallback.
      A `Delivered(Plural, Gender)` function must select its plural branch and gender result directly.
      Current form maps, named functions, and lifted inline functions all lower through the shared
      `Decision` model and canonical dispatch template. Equal results/fallbacks merge while
      preserving selector evaluation. Executed cases cover eager captures, lazy results, callable
      attributes, multi-key/nested branches, `__proto__`, fallback errors, coercions, and source maps.
      No branch-table helper remains in current runtime generation; frozen comparison fixtures
      retain their historical implementation. Exact page/helper demand remains open under OPT-A4.
- [-] OPT-A4 — Specialize helpers to the transitive operations/categories used by the page's exact
      message closure. Deduplicate genuinely shared operations without importing whole formatter,
      plural, locale, or message runtimes. Emit no unused helper, data entry, option dispatcher,
      inactive locale, or scaffolding in initial page delivery; integrate with ESM-A3/MOD-A4.
      Numeric scale/scientific/significant/padding helpers now have independent demand flags,
      verified by isolated generated/executed plans and absence checks for unselected helpers.
      Identical date-name vectors also share one binding across selected formatter requests.
      Different widths and Russian formatting/standalone values remain distinct; executed Date
      getter counts are unchanged. Real bundles keep shared tables without retaining the unused
      era formatter and drop every month table for a time-only consumer.
      Precision, rounding quantum/halfway threshold, scale shifts, and significant widths are
      compiled into deduplicated helpers; no static option validation or discarded-digit power
      expansion remains. Unit rounding skips bigint construction unless a carry is needed.
      Fixed-width currency fractions omit trimming; date helpers follow actual field demand.
      Known mandatory fractions now omit empty-fraction tests; non-scientific integer-only plans
      omit fraction/formatted bindings. Executed currency precision, optional/significant fraction,
      negative-zero, digit-localization, equal-prefix, grouping, and padding cases verify this.
      A 45-plan exact rational oracle covers precision/increment combinations, carries, huge
      values, and 8,000-digit fractions alongside the locale, currency, and scientific corpora.
      Plural decision/operand demand is shared through `linguini-codegen`; generated predicates use
      direct comparisons, common guards, and one calculation per remainder. Operands are local
      scalars emitted at first use, without operand objects, relation-helper calls, or range arrays.
      Message parameter-key arrays are initialized once per module; positional normalization
      reuses its argument array. Generated execution preserves named own-key validation, sorted
      unknown-key errors, Proxy/getter order, and frozen positional inputs.
      Fraction-digit counts avoid constructing fractional strings or bigints; Russian fractions
      return before bigint construction, verified by executing with bigint construction disabled.
      Decimal-modulus-only integer rules now compile a bounded suffix instead of a full integer.
      Suffixes of at most 15 digits use exact number arithmetic; wider suffixes use bounded bigint
      arithmetic, and the largest suffix modulus is omitted. Shared demand analysis rejects this
      optimization when any integer relation requires a full value or non-decimal modulus.
      Generated execution covers widths 1–19, mixed moduli around the safe-integer boundary,
      8,192-digit values, compact/native exponents, and bounded bigint/zero-expansion checks.
      A shared guard/operand proof now permits direct safe-number integer suffix calculation,
      avoiding string conversion and regex matching while sharing the category predicates.
      Russian/Ukrainian boundary and coercion tests preserve all other input paths and limits;
      the complete-consumer cost tradeoff is recorded under OPT-A7.
      Executed corpora cover 24 locales, fractional operators, compact exponents, and invalid inputs.
      Namespace/barrel global imports now follow shared direct semantic edges; static namespaces
      and re-export-only barrels omit them. Project and physical-leaf execution covers captured
      globals, alias-typed callable forms, simultaneous inline-input scope, named/positional calls,
      and sparse regional fallback. Transitive dependency selection remains a separate shared plan.
      Reusable TypeScript web handlers compile switch-route marker/suffix bounds when the
      application constructs its handler; native code no longer generates routing policy.
      Executed comparisons retain encoded-locale, suffix, redirect, same-origin return-target,
      and cookie/path transport behavior. Reusable TypeScript Svelte state, controls, and effects
      own their lifecycle and capabilities. Real Svelte compilation, runtime matrices, and site
      execution cover SSR, denied capabilities/persistence, preloading, navigation/state,
      callable arity, and cleanup. Initial tree-shaken runtime demand remains open.
      Accept-Language uses direct preference selection and scalar winner state, without per-locale
      filter/sort arrays or candidate objects. Explicit cases and 512 frozen-runtime comparisons
      retain specificity exclusions, quality, input/base/locale order, and navigator fallback.
      Numeric grouping compiles equal/unequal group widths into direct string loops without
      group arrays/reversal/join. A 32-plan execution corpus covers width boundaries, padding,
      Unicode separators, negative zero, and 8,192-digit inputs alongside formatter/rounding cases.
      Shared date-symbol planning now resolves Gregorian format/standalone contexts, name widths,
      locale AM/PM, and era labels before emission; identical per-formatter tables share a binding.
      Executed context/width/locale cases correct Arabic/Hindi output rather than preserving the
      previous English labels. Extended calendars, selectable non-UTC zones, non-exact skeleton matching, and additional fields stay open.
      Date digit substitution now resolves its ten Unicode symbols during generation. Numeric
      fields share a compiled callback; BMP symbols use literal indexing and consecutive
      supplementary symbols use code-point arithmetic. Other mappings compile direct branches.
      Project plural-data validation also follows lowered operation demand: numeric/literal
      Chakma output builds without unused rules, while required missing data fails before emission.
      Ordinary, standalone, and physical graphs execute; native check/sample/cold/warm builds
      preserve valid output on required-data errors.
      The shared lookup template also fixes UTF-16 indexing of default Adlam number/currency
      digits; native cold/warm, three input representations, mixed-width mappings, and strict
      JSDoc consumers execute without changing existing BMP snapshots. Executed traces replace eleven per-field symbol arrays/callbacks with
      zero arrays and one callback identity in a three-message Arabic consumer. A shared proof
      permits three whole-result substitutions instead of eleven field substitutions, preserving
      ASCII digits in literals or textual names through the original field-only path. ASCII date output
      omits digit support, and getter observations plus Date/string/timestamp output remain covered.
      Remaining helper specialization and exact composed-page delivery stay open.
- [-] OPT-A5 — Verify generated structure and executed semantics together: forbid generic static-
      parameter formatter forwarding and allocating dispatch tables, inspect/minify real page
      graphs, and execute numeric/date/CLDR, function/form, sparse fallback, and SSR/hydration corpora.
      Preserve all meaningful coverage while replacing snapshots of the superseded generated shape.
      Keep optimization and SSR acceptance open until production generation and real consumers pass.
      Explicitly formatted call arguments and inline payloads now validate and execute instead
      of silently losing annotations. Ordinary/physical graph regressions cover nested number,
      currency/date, and formatter chains, named calls, eager input order/coercion/errors, and
      unformatted forwarding. A selected numeric-only graph emits no currency/date helpers;
      native invalid annotations preserve existing output. Current snapshots stay unchanged.
      The correction passes affected Rust/editor tests, workspace Clippy, Rust 1.87, and Node 20;
      fourteen actual generated native modules pass strict checked-JavaScript validation, and
      their positive/negative TypeScript consumer passes without `allowJs`. Seventeen registered
      consumer/documentation/site gates pass, including source packaging and production SSR/browser
      checks. This follow-up does not replace the earlier full-run source receipt.
- [-] OPT-A6 — Generate readable code wherever the compiler has enough semantic information:
      use meaningful parameter/local/helper names, clear formatting, and direct control flow.
      Preserve explicit source parameter names; derive readable names for unnamed selectors from
      their semantic types and resolve collisions deterministically. Use opaque internal names
      only where needed for correctness or isolation. Readability must not compromise runtime
      performance, payload size, exact dependency delivery, evaluation semantics, source maps,
      type safety, or compatibility. Verify representative generated functions, forms, and
      formatters alongside the execution and output-size checks in OPT-A5.
      Argument-key tables also reuse the scoped form-binding encoder, replacing repeated internal
      prefixes with readable qualified names. Injectivity covers escaped names and compiler-like
      source identifiers; three executed complete consumers retain exact minified/gzip/Brotli
      sizes, with unchanged declaration and source-map snapshots (GEN-A39). The affected emitter
      and CLI suites pass on Node 20/24, alongside workspace Clippy, Rust 1.87, and fourteen
      strict-consumer, documentation/package, and production-site checks.
      Unnamed form maps now share function selectors' reference walker and allocator, preserving
      ordinary `value` bindings while avoiding locale-variable and custom-plural-helper collisions.
      Executed legacy and physical semantic/message graphs verify `value`/`value2` references,
      both branch outcomes, and positional/named calls. Other readability/optimization criteria
      above remain open. Named-function calls now keep validation's precedence over same-named
      parameters in shared dependency planning and every output mode. Private binding scopes use
      readable numeric suffixes only for collisions with global functions/helpers, retaining source
      named-object keys and declaration signatures. Executed enum/currency/date aliases, direct and
      forwarded/inline calls, classifier/form selectors, getter order, coercion, and error identity
      cover GEN-A42. Strict JS and TypeScript consumers preserve the public argument contract;
      affected Node 20/24 suites, workspace Clippy, and Rust 1.87 checks pass. Native qualification
      also preserves schema-owned message parameters and named calls through nested filesystem,
      group, function, inline, and form scopes. Real check/build/preview and executed positional/
      named calls cover getter/coercion order and error identity; signature ownership stays in
      the schema. IR/CLI suites pass on Node 20/24, with common/backend tests on Node 20, strict
      Clippy, Rust 1.87, patch API compatibility, and three documentation gates passing. These are
      focused follow-up checks, not a replacement full-run receipt. Current snapshots
      remain unchanged. Shared scope-aware string facts also emit locale text variable references
      directly across ordinary and physical modules, including aliases and imported globals.
      Executed conversion traces retain raw parameters, numeric plural inputs, coercion hints,
      and thrown errors; unproven member references still coerce.
      Intrinsic-named local functions now receive readable private bindings such as
      `__lgl_name_$String`, shared by declaration/call/import planning (GEN-A49). A 23-function
      source executes ordinary, standalone, and physical output with numeric/week formatters,
      getter/coercion/error checks, and preserved source keys. All 188 emitter tests pass on
      Node 20; generated Node 24 and strict JS/TS 5.8/5.9/6 consumers pass. Existing generated
      code/declaration/map snapshots stay unchanged. Workspace Clippy, Rust 1.87, 223 patch
      API checks, and four documentation gates pass. Other declaration roles remain under audit.
      The shared helper's dictionary annotation also avoids source `Record` type shadowing
      (GEN-A50), retaining public enum/alias names. Permanent checked-JS and no-`allowJs` consumers
      cover both forms and reject invalid source values; the registered gate reproduced TS2315
      before the correction. All 189 emitter tests and CLI tests pass on Node 20, with Node 24,
      strict TS 5.8/5.9/6, Clippy, and Rust 1.87 checks passing. Only seven current golden JSDoc
      lines change; complete Vite 7/8 consumers retain identical minified/gzip/Brotli output.
      Numeric helper typedefs now use the reserved compiler namespace too (GEN-A51), avoiding
      source `GeneratedNumeric`/`GeneratedDecimal` type imports in standalone output. Permanent
      JS/declaration/consumer fixtures preserve both enum names and negative type contracts.
      All 190 emitter tests and CLI tests pass on Node 20; all 42 codegen gates, Clippy, Rust 1.87,
      Node 24, and strict JS/TS 5.8/5.9/6 checks pass. Existing changes contain only type comments,
      and complete Vite 7/8 consumers retain identical minified/gzip/Brotli output.
      Locale variables now share private intrinsic-safe value bindings too (GEN-A52). A source
      `undefined` variable no longer corrupts standalone numeric sentinels; inline lexical inputs
      keep their own values. Executed ordinary/physical/standalone cases cover finite values,
      signed zero, NaN/infinities, bigint, forwarding/captures, getters, and caller exceptions.
      All 191 emitter tests and CLI tests pass on Node 20, with Node 24, strict JS/TS 5.8/5.9/6,
      Clippy, Rust 1.87, 223 patch API checks, and three complete-project gates passing. Existing
      snapshots stay unchanged. Complete Vite 7/8 consumers verify the repaired behavior; the cost
      audit records both compression outcomes, including Vite 8's five-byte Brotli increase.
      Public message/namespace exports now separate source names from local compiler bindings
      (GEN-A53), preserving named/default APIs and strict declaration overloads. Native cold/warm
      builds, previews, ordinary/physical modules, and SDK standalone/helper collisions execute
      successfully. All 193 emitter tests and CLI tests pass on Node 20, with Clippy, Rust 1.87,
      223 patch API checks, three complete-project gates, and strict JS/application TS 5.8/5.9/6.
      Existing runtime/declaration/map goldens stay unchanged; four Vite 7/8 consumers execute
      as one self-contained chunk each. This closes these concrete binding defects; broader
      identifier, optimization, and page-delivery requirements stay open.
      The public plural-function generator now shares safe local bindings and static export
      aliases too (GEN-A56). Ten requested names across four canonical rule families retain
      public exports, category strings/passthrough, numeric/compact/fraction semantics, and
      invalid-input errors. A permanent strict-JS fixture covers positive/negative calls and
      private-export exclusion. All 196 emitter tests and CLI tests pass on Node 20, with Clippy,
      Rust 1.87, 223 patch API checks, strict JS 5.8/5.9/6, Node 24 custom-name execution, and
      eight runtime/project/documentation gates passing. Existing output snapshots
      remain unchanged; executed Vite 7/8 consumers retain ordinary-name minified/gzip/Brotli sizes.
      Accepted programmatic IR type keywords now use a separate type-identifier policy
      (GEN-A57), shared by definitions, references, imports, and short aliases. Enum/alias
      projects cover 21 names, valid contextual controls, qualified short aliases, and a source
      compiler-prefix collision; values and argument keys retain their naming. Permanent
      positive/negative checked-JS/no-`allowJs` contracts pass on TS 5.8/5.9/6, preserving finite
      enum types and source String aliases. All 197 emitter tests and CLI tests pass on Node 20;
      ordinary/physical/standalone calls and getter order execute correctly. Node 24, Clippy, Rust
      1.87, 223 patch API checks, and eight schema/project/documentation gates pass. Existing output
      snapshots stay unchanged, and complete Vite 7/8 before/after consumers retain byte-identical
      bundles. Native case rules remain intact.
      Native date annotations now share qualified host-type templates (GEN-A59), preventing an
      accepted unqualified source `Date` type from capturing native date inputs/helper results.
      Public source `Date`/`globalThis` types retain their own identities. Executed ordinary,
      physical, and standalone modules cover native Dates, epoch numbers, strings, named getters,
      invalid inputs, and caller errors. Both strict consumer gates failed before the correction;
      positive/negative checked JS and application TS now pass on 5.8/5.9/6. All 198 emitter tests
      and CLI tests pass on Node 20, with the new corpus on Node 24, Clippy, Rust 1.87, 223 patch
      API checks, and eleven consumer/documentation gates passing. Eleven changed existing JS
      goldens retain identical executable syntax; existing source maps and frozen comparisons
      stay unchanged. Complete Vite 7/8 consumers execute with byte-identical before/after bundles.

- [-] OPT-A7 — Audit internal representations and allocation costs from current generator output.
      Digit replacement callbacks now share one compiled function per alphabet across number,
      currency, and date requests (GEN-A64). Scientific mantissas/exponents share it too;
      distinct alphabets stay separate, and ASCII-only output emits none. Eight complete
      Vite 7/8 consumers retain identical text and one self-contained chunk each. Five rounds
      expose one callback instead of eleven, and minified bundles shrink by 88–92 bytes.
      The cost audit records compression differences, including a three-byte gzip increase.
      Shared form demand projects standalone closures and generated projects to used attributes
      and their transitive dependencies before inline lowering. Physical form modules expose
      independent attributes as named exports through namespace imports, so each page retains
      its own tables/functions. Ordinary, physical, and standalone execution preserves lookup,
      prototype/coercion errors, eager initialization, callable rows, and named calls. Shared native
      Vite 7/8 contracts execute six page shapes and two locales, including self-attribute calls,
      opaque callable/eager forms, sparse message fallback, and strict JS/TS consumers. Gender-only
      pages save 370/363 Brotli bytes; mixed pages add 3/6 bytes and their table/row literals grow
      from three to six (GEN-A40). Callable/eager forms and mixed-table coalescing remain
      conservative/open, alongside the broader page-delivery and allocation criteria below.
      The [cost audit](docs/generated-code-cost-audit.md) records category-name versus numeric
      classifier measurements and GEN-A15–A20. Numeric literals alone show no useful memory gain
      in that local experiment. Compile selector outcomes from the actual branches, merge equivalent
      categories, and omit irrelevant rules/operands; use numeric codes where complete-consumer
      measurements justify them. Preserve public plural names and distinguish raw numeric operands
      from already classified internal results. Successful named validation now creates no
      unknown-key collection or conversion arrays and performs no sort; executed arities 0–128
      preserve classification/conversion phases and observable inputs. Date validation now uses
      Gregorian arithmetic: valid text constructs only its returned Date, invalid calendar fields
      construct none, and supplied Dates retain identity. All four-digit years, three host zones,
      leap centuries/year zero, offsets, malformed/time-invalid inputs, and overridden methods
      execute against explicit expectations and native calendar rollover.
      Scientific rounding carries now use shared build-time proofs and compiled literal digits
      when the result is known. Fixed/optional/minimum/significant/increment precision, signed
      exponents, and numeric input representations execute with exact expectations; unproven
      increments retain the general path, and full engineering notation remains outside this receipt.
      Shared integer-modulo exhaustiveness proofs also replace redundant final category tests
      with direct returns and remove operands/moduli no longer demanded. Complete finite periods
      establish the result during generation; no residue table or CLDR evaluator is shipped.
      Proven native integer suffixes also bypass parsing: executed spies observe zero String
      conversions and regex matches for safe integer counts. The measured complete Russian
      message grows by 22 Brotli bytes; local integer-call time changes from 0.409 to 0.071 µs.
      String and mixed inputs retain output parity. Bounded week-format selectors now compile
      only live wording outcomes from weeks 1–53, removing constant operands and redundant
      modulo operations while preserving overlapping category order. Literal-only alternatives
      share numeric field evaluation. Eight complete Vite 7/8 consumers execute in one chunk;
      strict JS/TS and native previews cover the same compiled plan (GEN-A47). These receipts
      do not close repeated normalization, general message-selector outcomes, or the broader
      page-delivery criteria.
      Constant numeric plural decisions now bypass parsing for every finite native number and
      omit the unreachable native-number regex. Japanese/Chinese/Korean/Thai and a constant-one
      model retain coercion, category passthrough, compact syntax, invalid-input errors, and digit
      bounds; conversion/regex spies verify the fast path. A complete consumer saves 22 Brotli
      bytes and improves local numeric calls, without a comparable string-input gain. Backend
      and CLI suites pass on Node 20 and Node 24 with unchanged snapshots; strict generated
      Japanese JavaScript, workspace Clippy, Rust 1.87, source packaging, and seventeen consumer/
      documentation/production-site gates pass. Selector-specific outcome demand stays open.
      Number/currency parsing also bypasses decimal regexes for safe native integers and
      bigints, preserving negative zero, caller coercion, and the 8,192-digit bound. Executed
      tests disable regex execution on that path. A complete two-message consumer adds 44
      Brotli bytes; local integer/bigint calls take roughly half the previous time and mixed
      inputs improve by about 23%, while fractional/string handling shows no material gain.
      Full consumer measurements and their limits are recorded in the generated-code cost audit.
      Unit fraction rounding now reuses unchanged canonical records, omitting duplicate records,
      slices, padding, and optional trimming on that path. Required visible minima remain in the
      formatter; executed identity/immutability checks complement the unchanged rational-arithmetic
      oracle and numeric/scientific/increment corpora. A two-message consumer adds 16 Brotli bytes;
      local number integer calls roughly halve and mixed inputs improve about 23%. Currency
      timing shows no comparable gain. The broader representation/page-demand audit stays open.

- [-] OPT-A8 — Minimize localization requests, transferred bytes, and runtime work. SSR must
      return final translated text in initial HTML without fetching messages in the browser;
      JavaScript-disabled pages need no locale payload. For hydration, place only the selected
      page/locale dependency closure in existing page JavaScript where possible; otherwise use
      one preloaded, self-contained page/locale artifact. Do not add per-message, per-component,
      or per-layout request fan-out, inactive locales, extra base catalogs, duplicate helpers,
      raw CLDR patterns, or a general interpreter. Load additional messages/locales only for
      actual client actions, and reuse prepared/cached payloads without redundant requests.
      Known bindings should compile to direct values/calls without locale lookups, wrappers,
      dispatch, or allocations unless their runtime behavior requires them. Verify initial
      and transition request counts, compressed/uncompressed bytes, dependency duplication,
      and executed operations on real composed pages. Keep this open until MOD-A4 is proven.
      The composed-page production gate checks exact initial identities for three pages across
      nine locales and reports raw/gzip payload sizes. Chromium verifies one selected preload
      before framework execution, no requests without scripting, and cache reuse on revisit.
      Additional closures have exact title/function identities across nine locales, no initial
      messages or unused exports, and no imports. The browser gate checks lazy action requests,
      same-page locale preparation, canceled requests, and cross-page module/data reuse. Setter-only
      builds retain no catalogs. Remaining work includes unsupported deferred shapes, broader
      configuration coverage and unnecessary stateful facade work. Optional loader-hook branches
      are now absent when no additional closures exist, through build-selected implementations
      rather than mutable-registry shaking. These receipts do not establish zero runtime overhead.
      Nested payloads also omit messages guaranteed by their ancestors. Production checks require
      exactly one parent and one child file per locale, with no empty transit files or unused
      records; browser checks retain the existing one-file initial delivery and defer the child
      payload until its own action.

The canonical JS migration now resolves actual formatter requests (including schema alias
inheritance and explicit overrides), compiles number/currency choices and selected Gregorian
widths into value-only functions, and obtains currency symbols from pinned CLDR data. Named and
inline dispatch use static decision trees; inline captures become explicit helper arguments.
Executed ESM fixtures cover nested branches, callable form attributes, multi-key branches,
`__proto__`, fallback errors, source provenance, and naming collisions. Selector names derive from
semantic types, and common form/qualified bindings retain readable source components with
collision-safe escaping. These receipts do not close expanded calendar/context/zone/skeleton support,
remaining emitter extraction, exact page delivery, or framework SSR/hydration verification.

### Documentation identity through codegen

- [x] API-D1 — Preserve docs on every declaration kind through semantic IR. Exhaustive schema and
  locale lowering coverage verifies enums, aliases, variables, implementations, `form`, `fn`,
  messages, nested messages, and groups, including empty paragraph lines.
- [x] API-D2 — Preserve group docs in recursive namespace metadata.
- [x] API-D3 — Use schema docs as canonical public API prose.
- [x] API-D4 — Attach docs to exact JSDoc exports and `.d.ts` leaves/overloads.
- [x] API-D5 — Preserve paragraphs/line breaks and escape comment terminators. One shared renderer
  now emits every runtime and declaration kind, combines consecutive source comments into a single
  JSDoc block, retains empty paragraph lines, and neutralizes embedded `*/` terminators.
- [x] API-D6 — Generate backend tags separately from source prose. Positional and named-object
  overloads derive typed `@param` and `@returns` tags from the shared signature/`TypeModel`, append
  them independently of canonical prose, and escape hostile source property names.
- [x] API-D7 — Preserve source identity for hover/navigation and add golden tests. A cross-file
  golden resolves canonical schema prose by namespace while rejecting same-named unrelated docs,
  then navigates to the exact schema URI, `SourceId`, byte span, and declaration text.

### CLI/LSP/VSIX distribution

- [x] DIST-A1 — Publish a launcher plus six version-locked native optional packages.
- [x] DIST-A2 — Stage one native binary into each target-specific VSIX.
- [x] DIST-A3 — Build a universal browser-worker/WASM LSP fallback. The universal VSIX contains
      the shared Rust server plus browser/Node workers; native VSIX packages reuse that same
      checksummed runtime. Installed Linux native and universal packages pass real Extension Host
      hover/navigation/highlighting/restart checks, including explicit launcher wrappers and auto
      fallback with an empty `PATH`. Actual WASM protocol tests pass in Node 20/24 and Chromium.
- [x] DIST-A4 — Resolve explicit, project, bundled, then PATH server candidates.
- [x] DIST-A5 — Perform a compiler/LSP protocol handshake.
- [x] DIST-A6 — Avoid first-activation executable downloads.
- [x] DIST-A7 — Share compiler semantics with WASM and vary only host services.
      The native and memory hosts share the full Tower LSP backend, configuration/namespace
      logic, bounded snapshots, source IDs, and diagnostics/version guards. A serialized-protocol
      parity test covers navigation, rename, completion, symbols, tokens, formatting, highlighting,
      diagnostics, and edit recovery; custom source roots under build directories remain indexed.
      The transport-only `linguini-lsp-wasm` crate runs that same service in browser and Node
      workers, with host-provided saved files and scheduling. Rust plans valid configured roots
      alongside invalid-project errors; TypeScript owns filesystem access and URI mapping.
      The worker corpus verifies saved-file refresh, namespace isolation, Unicode coordinates,
      stale-edit rejection, and error recovery. Native and WASM strict Clippy and the WASM
      dependency tree's Rust 1.87 `cargo-msrv verify` check pass locally.
- [-] DIST-A8 — Complete signed/checksummed release and marketplace CI.
      Release jobs now build/test one WASM artifact, reuse it across six native VSIX jobs, package
      a universal VSIX, and verify archive SHA-256 sidecars before Open VSX publication. Local
      package/installed-host checks and workflow lint pass; current remote cross-platform jobs,
      signing/provenance, and actual marketplace publication remain unverified.

### Bundler-native nested message API

- [x] BUNDLE-A1 — Generate recursive declarations for the complete `l` namespace.
- [x] BUNDLE-A2 — Transform static `l.*` leaf access into exact virtual imports.
- [x] BUNDLE-A3 — Keep parameterless public access as a value while rewriting its internal use to
      a reactive virtual-message call.
- [x] BUNDLE-A4 — Generate one ESM module per referenced message.
- [x] BUNDLE-A5 — Include only each message's transitive semantic dependencies.
- [x] BUNDLE-A6 — Invalidate only affected virtual modules.
- [x] BUNDLE-A7 — Reject dynamic lookup in strict mode and permit only explicit finite computed
      lookup escapes through manifest metadata, generated-module resolution, frozen dispatches,
      and real-site production-graph verification.
- [x] BUNDLE-A8 — Share the single-message compiler with a physical-module backend.
- [x] BUNDLE-A9 — Let Vite own route/shared chunks, preload, and network loading.
- [x] BUNDLE-A10 — Express optional locale splitting through bundler-visible dynamic ESM
      boundaries with preload-before-switch behavior, one coalesced virtual entry per effective
      locale, no per-message client request fanout, inactive-locale client chunks, synchronous SSR
      modules, and real-site production-graph verification.
- [x] BUNDLE-A11 — Emit formatter, date, number, currency, and plural helpers once per locale and
      import only required names from physical message modules.
- [x] BUNDLE-A12 — Share reusable variables, forms, and local functions across physical messages
      without broadening their transitive bundle graph.
- [x] BUNDLE-A13 — Emit locale globals once per locale and import their bindings from root
      and namespace modules instead of copying declarations into every generated module.
- [x] BUNDLE-A14 — Coalesce exact message facades behind one dynamic virtual entry per effective
      locale, without per-message client entries, while preserving locale-granular HMR invalidation.
- [x] BUNDLE-A15 — Centralize dynamic locale preparation in one virtual registry per application
      scope: each registry owns one cache, ref-counted loader lifecycle, and literal dynamic import
      per effective locale; message facades contain no per-message loader maps, preparation caches,
      or unprepared-locale crash branch.
- [-] BUNDLE-A16 — Scope client message facades and locale entries by normalized application path,
      emit only that application's exact finite message set in each lazy locale payload, release
      inactive Svelte route loaders after navigation, keep SSR eager, and prove build/preview
      behavior on a three-message production route across English, Russian, and French. Existing
      application-scoped verification is a baseline; page/component/layout coalescing and the new
      adapter/configuration path still require the MOD-A4/A5 integration coverage.
- [x] BUNDLE-A17 — Encode scoped locale payloads as stable indexed arrays and compile numeric
      facade lookups so canonical message paths do not survive in browser chunks, while preserving
      sparse-locale fallback, parameterless values, parameterized calls, and existing unscoped APIs.

### Local repository verification

- [x] LOCAL-A1 — Provide one root runner with ordered quick, full, and real-site profiles across
      every project/package in this repository. Runtime fixture tasks select explicit relative
      file paths, following Bun's exact-file convention, rather than substring discovery filters.
      A regression rejects bare filters and missing registered files. All eighteen runner tests
      pass on Node 20/24, and the nine corrected tasks each execute their one intended fixture
      with archived semver-check copies still present under `target/`.
      Repository installation now refreshes local directory packages by recreating their
      consumer's modules from the frozen graph. It preserves manifest, workspace-policy, and
      lockfile bytes, without resolving registry dependencies again. Cold/warm offline fixtures
      verify added/replaced/deleted files and registry dependencies with an empty metadata cache.
      Failed installation or graph drift restores the prior modules as well as locked metadata;
      successful/failed refreshes leave no backup directory. Registry-only consumers keep normal
      frozen installation. All twenty-three installer/runner tests pass on Node 20/24. Runtime
      preparation and actual Svelte/tsc setup paths pass, including empty metadata caches. The
      existing CI job already runs the expanded regression. Quick/full profiles include it;
      hosted confirmation of this fix remains required after pushing (run 45954831211).
- [x] LOCAL-A2 — Generate, type-check, build, and inspect the real site production graph locally.
      Site builds generate current native messages and highlight tokens into ignored directories;
      archived output and its restore/package workflow are removed. A repository gate rejects
      committed application build output. Runtime tests, source-hash freshness checks, Svelte
      checking, production builds, and scoped graph assertions pass locally. This baseline does not
      close the composed-page, request/network, or hydration requirements in MOD-A4/A5.
- [x] LOCAL-A3 — Start the real site development server without generated-output watcher storms,
      rebuild after locale edits, and verify English and localized routes in a browser.

### Positional and named-object generated calls

- [x] CALL-A1 — Preserve positional calls.
- [x] CALL-A2 — Add named-object overloads without changing Linguini syntax.
- [x] CALL-A3 — Normalize both call forms once at the function boundary.
- [x] CALL-A4 — Render JSDoc and `.d.ts` overloads from one signature model.
- [x] CALL-A5 — Reject missing/unknown object properties through generated types.
- [x] CALL-A6 — Keep parameterless messages as values only.

### Dedented and raw multiline messages

- [x] TEXT-A1 — Represent `Dedented` and `Raw` modes explicitly.
- [x] TEXT-A2 — Parse multiline content as a real text pattern with placeholders.
- [x] TEXT-A3 — Remove structural edge blank lines for ordinary blocks.
- [x] TEXT-A4 — Remove the common whitespace prefix once.
- [x] TEXT-A5 — Preserve relative indentation, internal blanks, and trailing spaces.
- [x] TEXT-A6 — Normalize ordinary line endings deterministically.
- [x] TEXT-A7 — Preserve every raw-block byte and protect it from formatting.
- [x] TEXT-A8 — Keep interpolation active and brace escaping separate.
- [x] TEXT-A9 — Preserve source mappings across normalization.
- [x] TEXT-A10 — Complete all specified syntax/format/codegen golden cases. Parser and formatter
  matrices cover empty, one-line, quote-rich, interpolated, CRLF, Unicode, indentation, internal
  blank, and trailing-space cases; codegen now asserts exact dedented and byte-preserving raw
  expressions from parsed semantic IR.
- [x] TEXT-A11 — Keep quotes literal after an unquoted value start, including unmatched quotes
  and text after placeholders; preserve explicit quoted escapes and both multiline modes.
  The failure was reproduced and the lexer now tracks the selected text mode. Parser, formatter,
  generated execution, editor scopes, and documentation syntax checks pass; the full Rust
  workspace passes, and regenerating the real site retains its prior archive hash.

### Arbitrarily nested message groups

- [x] GROUP-A1 — Represent groups recursively in syntax and semantic models.
- [x] GROUP-A2 — Use canonical qualified paths across semantic tooling.
- [x] GROUP-A3 — Detect duplicate segments and message/group collisions.
- [x] GROUP-A4 — Preserve source IDs/spans for every path segment.
- [x] GROUP-A5 — Generate nested types/runtime without copying unrelated symbols.
- [x] GROUP-A6 — Bound traversal depth safely.
- [x] GROUP-A7 — Define and validate empty-group behavior.

## Prioritized exit criteria

### P0

- [x] P0-1 — Hermetic read-only CLDR compilation.
- [x] P0-2 — Safe manifest-owned output root.
- [x] P0-3 — Correct form IR plus JS/JSDoc/`.d.ts` runtime corpus. Exact JavaScript message and
      semantic-form/function leaves now share the TypeScript dependency closure and structured
      emitter, preserve overload JSDoc, and reject TypeScript-only syntax in regression coverage;
      the complete project corpus now checks native JavaScript with and without declarations,
      TypeScript, namespace calls, fallback, lazy preparation, and provider execution. All 57
      full-profile gates pass after removal of the duplicate runtime templates.
- [x] P0-4 — Validated project capability as the only emitter entry.
- [x] P0-5 — Blocking CLI error severity.
- [x] P0-6 — Atomic cross-kind locale indexes.
- [x] P0-7 — Symbol-resolved workspace rename.
- [x] P0-8 — Safe `TempDir` test support.

### P1

- [x] P1-1 — Keep native ESM targets separate from application-owned web/Svelte/SvelteKit runtime
  adapters. Remove web/framework policy from neutral TOML and the temporary native policy DSL.
  Workspace and packaged executable documentation plus real-site production pass on `c665a88`.
- [x] P1-2 — Freeze the tested v0.1 language specification. The versioned normative document is
  linked from the public guides, guarded against contract drift, and backed by the positive and
  negative syntax corpora plus documentation conformance gates.
- [x] P1-3 — Carry recursive namespaces through generated output.
- [x] P1-4 — Complete multiline conformance across all backends. Syntax normalization, typed IR
  mode identity, semantics-preserving/idempotent formatting, and exact TypeScript codegen are all
  covered for dedented and raw blocks.
- [x] P1-5 — Preserve call/declaration kinds, sources, and spans.
- [x] P1-6 — Finish the single project symbol/type/exhaustiveness database. Schema and locale
      exhaustiveness/type checks share `SchemaDatabase`; the unused parallel locale-scope database
      has been removed.
- [x] P1-7 — Make CLI and LSP consume that same database. Both consume it for diagnostics and
      coverage, message completion/hover, and fix discovery. Reference/definition/rename operations
      share one immutable navigation snapshot built from the schema-owned semantic occurrence walk.
      Named calls now retain function identity inside same-named function/inline value bindings
      (GEN-A45), matching native validation/generation. Exact definition/reference/rename spans,
      foreign-locale isolation, and semantic token kinds execute through the shared walk. The
      call-scope correction passes the 64-test LSP suite and actual Node/browser WASM protocol.
      Inherited variable/function and locale enum type navigation now reuses the analyzer's
      borrowed whole-declaration selection with explicit source ownership (GEN-A58). Definition,
      reference, and rename operations distinguish nearer overrides and unrelated locales;
      duplicate fallback sources and effective-global rename conflicts are guarded. Shared tests
      retain original roots/spans, value/call pairs, and same-named lexical bindings. Shared body
      scopes also reject renames with different effective bindings in consuming locales, while
      allowing unrelated globals and the inherited container itself to be renamed. All 71 LSP
      tests pass on Node 20, including native/memory protocol parity and fallback configuration
      changes and refused ambiguous renames. Built Node 20/browser WASM protocol, workspace
      Clippy, Rust 1.87 native/WASM, and 223 patch API checks per affected crate pass. Typed
      form-property navigation remains separate.
      Navigation now returns the existing URI/span/declaration result directly, keeping full
      documents in the snapshot instead of copying text and line indices into every match.
      A warmed debug SDK query over 16 locales, 282,545 source bytes, and 1,025 references
      retains identical ordered URI/source/span/declaration results while reducing peak
      additional requested heap from 19,854,910 to 669,505 bytes. This measures the query's
      requested allocations, not production latency or client JavaScript. All 71 LSP tests,
      actual Node/browser WASM protocol, Clippy, Rust 1.87 native/WASM, and 223 patch API checks
      pass; public signatures and generated artifacts remain unchanged.
- [x] P1-8 — Implement one CLDR locale fallback service.

### P2

- [-] P2-1 — Complete the canonical JavaScript template backend and shared semantic models.
  `linguini-codegen` owns common types, requirements, and message dependency slicing. Remaining
  runtime extraction, TS implementation removal, and module boundaries require ESM-M12/M13 and
  MOD-A1–A8 verification; earlier paired-target matrix receipts remain historical baselines.
- [x] P2-2 — End-to-end documentation propagation. Parser attachment survives schema/locale
  semantic lowering for every declaration kind and reaches exact runtime exports, recursive
  namespace properties, overloads, and `.d.ts` declarations through one safe JSDoc renderer.
- [x] P2-3 — Emit both multiline modes from semantic text IR. End-to-end codegen coverage starts
  from parsed CRLF input, verifies retained `Dedented`/`Raw` identities, and asserts normalized
  dedented text plus byte-exact raw whitespace in generated runtime expressions.
- [x] P2-4 — Collision-safe identifiers and filenames.
- [x] P2-5 — Bundler-visible lazy locale boundaries and deduplicated formatter data.
- [x] P2-6 — Compile-time typed `l` namespace transform. Recursive generated declarations type the
  source facade before Vite replaces exact static leaves and bounded dynamic paths; 45 plugin tests
  plus real-site positive/negative Svelte/TypeScript checks pass.
- [x] P2-7 — Positional plus named-object overloads.
- [-] P2-8 — Typed application runtime configuration with reusable web/Svelte/SvelteKit npm
  packages and thin project bindings. The `[web]` TOML contract, native policy plan, and generic
  runtime generation are removed; verify exact composed-page delivery and bundle demand.
- [x] P2-9 — One shared browser/server locale transition plan.
- [x] P2-10 — Safe streaming-compatible link localization.
- [x] P2-11 — Fully incremental/cancellable namespace-aware LSP. Cached versioned document state,
  bounded discovery, cancellation/debouncing, dependent diagnostics, workspace symbol resolution,
  and context/type-aware completion are active through the production server.
- [-] P2-12 — Native npm/VSIX plus universal WASM distribution.
- [x] P2-13 — Real TOML parsing and migration diagnostics.
- [x] P2-14 — Atomic writes and conflict-aware fixes.

### P3

- [x] P3-1 — Declared Rust minimum/stable and Linux/Windows/macOS/offline/package CI.
  [Run 45868820891](https://github.com/antepodeum/Linguini/actions/runs/45868820891) on
  `3ec4923` passes the full CI matrix: stable workspace tests on all three operating systems,
  Rust 1.87 native/WASM minimum checks, dependency policy, packaged crates, editor/WASM,
  TypeScript 5.8/6 and platform integrations, Vite 5–8, adapters, and the production site.
  Native Windows contexts now match file-URI buffer identities, and host parity waits for
  versioned diagnostic events under delayed scheduling. Offline package/source checks also
  pass locally. Final architecture-wide verification remains ESM-M11.
- [x] P3-2 — Parser, formatter, plural, and path property/fuzz coverage. Seeded generated cases
  cover Unicode/recovery/nesting/size parser invariants, formatter no-panic/idempotence/semantic
  preservation, plural modulo arithmetic, and portable locale ancestor chains.
- [x] P3-3 — Reverify the documentation-to-runtime corpus after the runtime migration. On
  `c665a88`, all 101 registered blocks pass the full profile: syntax/configuration, workspace
  and packaged JavaScript/TypeScript execution, and every documented SvelteKit source variant.
  Later public contract changes still require companion documentation and executable verification.
- [x] P3-4 — ICU/CLDR differential tests cover 152 representative cardinal cases across eight
  structurally distinct plural-rule families.
- [-] P3-5 — Vite/native VSIX/npm CLI/WASM integration matrix.
- [x] P3-6 — Dependency policy, pinned actions, reproducible JS packaging, synced versions.
  Cargo policy/advisory jobs, immutable action SHAs, enforced pnpm boundaries and reviewed build
  scripts, exact native optional versions, and repository-wide version checks are all active.
  The same remote run exposed path-only private test helpers as wildcard dependencies. The
  scoped private/development-path allowance now preserves Cargo's omission of unpublished test
  helpers from registry manifests. All dependency/advisory/license/source checks and offline
  dry-packaging of publishable crates pass locally. Four isolated policy probes accept private
  and development paths while still rejecting registry and public-production path wildcards.

## Final verification

- [x] Run the complete Rust workspace test suite on Linux, Windows, and macOS. Verified against
  source `1cbc4c6` in CI run 39110695824.
- [x] Run strict Clippy for all targets and features.
- [x] Run declared Rust minimum and stable checks. Current discovery measures 1.87;
  older Rust 1.76 receipts describe their historical source revisions.
- [x] Run offline/package/source-archive verification.
- [x] Run dependency, license, advisory, and public-API compatibility checks. Local verification
  passed with cargo-deny 0.20.2, cargo-audit 0.22.2, and cargo-semver-checks 0.50.0 against
  `origin/dev`; the workspace crates remain prerelease `0.1.0-alpha.4`, so semver-checks correctly
  treated the unchanged prerelease versions as major-compatible and skipped stable-release lints.
- [x] Run Vite 5–8, SvelteKit, npm launcher, native VSIX, and WASM integration tests. Local Linux
  receipts cover the full-profile npm/site/WASM checks, 80 adapter tests per major on `0143d99`,
  and fresh installed native/universal VSIX packages in native, explicit WASM, and empty-PATH
  automatic modes. The final migrated-source cross-platform matrix remains the separate gate below.
- [x] Parse, analyze, generate, typecheck, and execute every normative documentation fixture.
- [ ] Rerun the full locked Rust/Clippy/MSRV, package/offline, documentation, strict TS/JSDoc,
  `tsc`, Vite 5–8, SvelteKit, and cross-platform CI matrix against the final migrated source.
- [ ] Re-read all 369 findings and every architecture criterion against final production paths.
- [ ] Mark this checklist complete only when no `[-]` or `[ ]` items remain.
