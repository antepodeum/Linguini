# Linguini audit remediation checklist

Source of truth: [`Linguini_Audit_EN_FINAL_WITH_DOCS.md`](./Linguini_Audit_EN_FINAL_WITH_DOCS.md).

Status legend:

- `[x]` implemented and verified locally;
- `[-]` partially implemented, but an integration, contract, or verification gap remains;
- `[ ]` not yet completed.

This file is intentionally stricter than a change log. An item stays partial until the
production path uses the fix and its relevant tests pass.

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
  legacy runtime so per-message wrappers can resolve locale without importing every message.
- `18792cf` — added opt-in typed bundler source discovery and a deterministic application
  manifest with exact hashes, byte spans, binding provenance, arity, and unresolved references.
- `1e5ae83` — added strict-module import parsing, stable binding identities, exact removal spans,
  and binding-wide transform-safety metadata to analyzer and the bundler manifest.
- `1ce930b` — split Svelte browser effects from the eager message runtime, added metadata-only web
  policy, and generated distinct plain-Svelte and SvelteKit reactive locale implementations.
- `1804d1f` — added lightweight Svelte client controls and compatibility-safe SvelteKit control
  hooks so transformed applications can avoid eager message barrels without breaking legacy APIs.
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

## Numbered findings

### Cross-cutting architecture and product claims

- [-] #1 — Align specification with shipped syntax, linting, typing, and ESM-only scope.
- [x] #2 — Enforce validated IR as the only production codegen boundary.
- [-] #3 — Consolidate duplicated semantic walks into one project semantic database.
- [x] #4 — Implement argument, selector, and reference type checks retained by the spec.
- [x] #5 — Implement retained exhaustiveness and unreachable-pattern checks.
- [x] #6 — Carry stable source identity through syntax, diagnostics, and semantic IR.
- [-] #7 — Use one namespace model for every declaration kind and every pipeline.
- [x] #8 — Use one CLDR-aware locale canonicalization and fallback algorithm everywhere.
- [ ] #9 — Expand CLDR formatting to the documented production contract.
- [-] #10 — Keep builds hermetic; the untracked site output is regenerated through Cargo until
  a source-checkout-native packaged generator path replaces it.
- [x] #11 — Make accepted web strategies exactly match generated runtime capabilities.
- [-] #12 — Hide invalid mutable public model states behind validated constructors.
- [x] #13 — Provide stable diagnostic codes, categories, severities, and source IDs.
- [-] #14 — Execute normative documentation as conformance fixtures. All 91 code blocks are
  registered and standalone Linguini syntax plus TOML config fences are enforced; generated
  TypeScript/Svelte runtime fixtures remain.
- [-] #15 — Add property, platform, generated-code, and real integration coverage.
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
- [-] #53 — Further restrict public AST mutation behind invariant-preserving builders.
- [x] #54 — Attach source IDs to spans.
- [x] #55 — Preserve complete override metadata and span.
- [x] #56 — Quarantine recovered/error nodes so invalid bytes cannot stitch declarations.
- [x] #57 — Make documentation attachment deterministic.
- [x] #58 — Validate deep property-path shapes.
- [-] #59 — Complete Unicode, recovery, nesting, and size fuzz/property coverage.

### `linguini-schema`

- [x] #60 — Diagnose duplicate enum variants before map construction.
- [x] #61 — Detect self-aliases and alias cycles.
- [x] #62 — Preserve formatter annotations on aliases.
- [x] #63 — Diagnose duplicate message parameters.
- [x] #64 — Keep recursive group membership and the message index consistent.
- [x] #65 — Enforce schema naming constraints.
- [x] #66 — Attach unknown-type diagnostics to exact type spans.
- [x] #67 — Preserve related-file source identity.
- [-] #68 — Make the schema builder a view over the shared semantic database.
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
- [-] #93 — Replace mutable public vectors with validated collections.
- [x] #94 — Diagnose duplicates before set/map insertion.
- [-] #95 — Finish splitting schema, locale, and validated project IR types.
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
- [-] #136 — Complete migration from heuristic tokens to the canonical syntax model.
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
- [ ] #158 — Expand date/calendar/context/time-zone/skeleton support.
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
- [-] #176 — Generate non-Latin numbering and broader calendar data. Locale-default numeric
      systems are compiled and emitted; broader calendar data remains open.
- [x] #177 — Parse text-direction JSON structurally.
- [ ] #178 — Implement complete CLDR number-pattern semantics.
- [x] #179 — Validate model limits before narrowing integer casts.
- [x] #180 — Share the runtime plural grammar. Generated Rust predicates and TypeScript predicates
  both lower the canonical `linguini-cldr::PluralRule`; neither target reparses CLDR rule text.
- [x] #181 — Parse the CLDR manifest as real TOML.
- [x] #182 — Verify the vendored archive/tree cryptographically.
- [x] #183 — Validate the packaged full-corpus manifest and golden hash.

### `linguini-codegen-ts` core emitter

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
- [-] #199 — Emit source maps back to Linguini sources. Bundler-native physical message and
      semantic modules have truthful maps; legacy project-wide string emitters remain unmapped.
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
- [-] #208 — Use the shared CLDR fallback graph. Rust codegen and CLI share the canonical CLDR
      chain; generated locale runtime still combines truncation with precomputed overrides.
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
- [-] #277 — Complete context/type-aware completion beyond indexed names.
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
- [x] #305 — Share preview semantics with the production evaluator.
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
- [x] #344 — Add lexer-derived grammar and language-configuration golden tests.
- [x] #345 — Perform a compiler/LSP protocol version handshake.
- [x] #346 — Verify clean extension install, compile, package, and VSIX contents.

### Site and documentation

- [-] #347 — Build/check the site without Cargo, network, `git`, or a Rust toolchain; generated
  output is no longer tracked, so clean verification currently needs the Rust generator.
- [x] #348 — Remove unshipped syntax/CJS claims and align the reference with conformance.
- [x] #349 — Reframe incomplete type, exhaustiveness, and unused-message guarantees.
- [x] #350 — Replace the unsupported site `preferredLanguage` strategy.
- [x] #351 — Define project and path namespace behavior unambiguously.
- [-] #352 — Execute examples through compile, typecheck, and runtime golden tests. The actual
  Getting Started config, schema, locales, JavaScript, and TypeScript direct-use example pass;
  the complete locale-provider example also generates and passes strict Svelte checking. Remaining
  normative web-guide TypeScript/Svelte snippets are registered but not yet executable fixtures.
- [x] #353 — Document actual CLDR formatter limits and fallback behavior.
- [x] #354 — Resolve the unfinished contracts tracked by `REPOSITORY_STATE`.
- [x] #355 — Synchronize version, copy, and release status across every surface.

### CI, release, and supply chain

- [x] #356 — Make the declared Rust 1.76 MSRV pass as a dedicated CI job.
- [x] #357 — Compile from packaged, offline, read-only CLDR data.
- [-] #358 — Land and execute Linux, Windows, and macOS CI.
- [x] #359 — Land dependency, license, and advisory checks.
- [-] #360 — Add enforceable coverage plus parser/formatter property jobs.
- [-] #361 — Real-site generated TypeScript/JSDoc/`.d.ts` overloads have positive/negative type
  checks and runtime parity tests, and one standalone JavaScript/TypeScript example executes; a
  broader standalone JavaScript/JSDoc corpus remains.
- [-] #362 — Complete real Vite, native VSIX, npm CLI, and WASM LSP integration CI.
- [x] #363 — Land and verify packaging for every publishable crate.
- [x] #364 — Land commit-SHA-pinned GitHub Actions.
- [x] #365 — Land the pinned `rust-toolchain.toml`.
- [-] #366 — Standardize and reproducibly verify JavaScript package-manager boundaries.
- [x] #367 — Land automatic release-version synchronization checks.
- [x] #368 — Add public API/semantic-version compatibility checks.
- [-] #369 — Make documentation and examples an executable conformance suite. The root full
  profile registers all 91 code blocks, enforces Linguini syntax plus TOML config fences, and
  builds, typechecks, and executes the Getting Started JavaScript/TypeScript example; packaged and
  remaining web-guide fixture coverage remains. The complete SvelteKit provider also typechecks.

## Required documentation migration

### `README.md`

- [x] DOC-R1 — State SvelteKit-first, ESM-only scope.
- [x] DOC-R2 — Remove CJS, multi-framework, and overstated type-safety claims.
- [x] DOC-R3 — Use value leaves, positional calls, and named-object overloads correctly.
- [x] DOC-R4 — Explain that Vite owns tree-shaking, chunks, preload, and loading.
- [x] DOC-R5 — Make the npm CLI the default JS installation; retain Cargo as an option.

### `docs/getting-started.md`

- [x] DOC-G1 — Replace flat web configuration with minimal nested policy.
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
- [ ] DOC-C2 — Build docs against packaged CLI/codegen artifacts.
- [ ] DOC-C3 — Snapshot config schema, CLI help, declarations, and examples in CI.
- [x] DOC-C4 — Require docs, migration notes, and fixtures for every public contract change. The
  quick/full runner and CI diff gate classify public CLI/config/syntax/codegen/Vite/LSP/editor
  boundaries and reject changes missing any companion evidence class.

## Additional architecture acceptance criteria

### Nested compile-time SvelteKit web policy

- [x] WEB-A1 — Parse typed optional `[web.routing]`, `[web.locale]`, `[web.cookie]`,
      `[web.local_storage]`, `[web.links]`, `[web.routes]`, and `[web.switch_route]`.
- [x] WEB-A2 — Replace `prefix_default_locale` with `always | except-default | never`.
- [x] WEB-A3 — Default routing to `except-default`.
- [x] WEB-A4 — Replace `redirect` with `canonical = redirect | preserve`.
- [x] WEB-A5 — Remove configured origin, base path, and trailing slash.
- [x] WEB-A6 — Restrict sources to path, cookie, local-storage, and accept-language.
- [x] WEB-A7 — Derive source defaults from routing and validate contradictions.
- [x] WEB-A8 — Validate cookie namespace defaults and `SameSite=None` security.
- [x] WEB-A9 — Validate/derive local-storage keys only when selected.
- [x] WEB-A10 — Replace `localize_links` with transform/runtime/manual.
- [x] WEB-A11 — Validate deterministic route exclusions.
- [x] WEB-A12 — Lower configuration into a closed `WebFeatures` set.
- [x] WEB-A13 — Generate only selected feature modules and no generic strategy loop.
- [x] WEB-A14 — Compile one `LocaleSwitchPlan` for browser and server transports.
- [x] WEB-A15 — Use `localizeHref` for all path-based transitions.
- [x] WEB-A16 — Generate a safe optional switch route with validated return targets.
- [x] WEB-A17 — Reject switch routes with no server-writable transition.
- [x] WEB-A18 — Share one route matcher across resolution, redirects, links, and switching.
- [x] WEB-A19 — Diagnose local-storage-only SSR expectations.
- [x] WEB-A20 — Remove every legacy flat field from runtime, templates, and docs. Public runtime
      policy is nested, generated SvelteKit adapters inject `$app/paths.base`, request/browser
      context owns origin, SvelteKit owns trailing slashes, and the Pages build no longer mutates
      generated source after codegen.

### One SvelteKit/ESM ECMAScript backend

- [x] ESM-A1 — Remove `targets.ts.module` from the model and all public surfaces. A value-level
      error probe preserves its migration diagnostic without accepting it into the raw model.
- [x] ESM-A2 — Treat SvelteKit as the primary supported adapter for now; it is not the only one
      planned for the future.
- [-] ESM-A3 — Introduce one structured ECMAScript module emitter; the single-message compiler
      now uses the structured emitter, while legacy project/runtime generation still uses direct
      TypeScript string assembly.
- [ ] ESM-A4 — Introduce one shared language-neutral `TypeModel`.
- [ ] ESM-A5 — Render JavaScript plus JSDoc from the shared models.
- [ ] ESM-A6 — Render `.d.ts` from the same `TypeModel`.
- [ ] ESM-A7 — Avoid a parallel TypeScript runtime implementation.
- [-] ESM-A8 — Emit source maps and shared runtime helpers from the common backend; exact message
      modules now have truthful source maps and demand-selected helpers, but project-wide output
      has not migrated.
- [x] ESM-A9 — Make parameterless public leaves values in both surfaces.
- [ ] ESM-A10 — Keep JavaScript generation first-class and near-zero-config.

### Documentation identity through codegen

- [-] API-D1 — Preserve docs on every declaration kind through semantic IR.
- [x] API-D2 — Preserve group docs in recursive namespace metadata.
- [x] API-D3 — Use schema docs as canonical public API prose.
- [x] API-D4 — Attach docs to exact JSDoc exports and `.d.ts` leaves/overloads.
- [-] API-D5 — Preserve paragraphs/line breaks and escape comment terminators; recursive namespace
      output preserves multiline docs and escapes terminators, but all declaration kinds have not
      migrated to the shared renderer.
- [ ] API-D6 — Generate backend tags separately from source prose.
- [ ] API-D7 — Preserve source identity for hover/navigation and add golden tests.

### CLI/LSP/VSIX distribution

- [x] DIST-A1 — Publish a launcher plus six version-locked native optional packages.
- [x] DIST-A2 — Stage one native binary into each target-specific VSIX.
- [ ] DIST-A3 — Build a universal browser-worker/WASM LSP fallback.
- [x] DIST-A4 — Resolve explicit, project, bundled, then PATH server candidates.
- [x] DIST-A5 — Perform a compiler/LSP protocol handshake.
- [x] DIST-A6 — Avoid first-activation executable downloads.
- [ ] DIST-A7 — Share compiler semantics with WASM and vary only host services.
- [-] DIST-A8 — Complete signed/checksummed release and marketplace CI.

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
- [x] BUNDLE-A13 — Emit locale globals once per locale and import their bindings from legacy root
      and namespace modules instead of copying declarations into every generated module.
- [x] BUNDLE-A14 — Coalesce exact message facades behind one dynamic virtual entry per effective
      locale, without per-message client entries, while preserving locale-granular HMR invalidation.
- [x] BUNDLE-A15 — Centralize dynamic locale preparation in one virtual registry per application
      scope: each registry owns one cache, ref-counted loader lifecycle, and literal dynamic import
      per effective locale; message facades contain no per-message loader maps, preparation caches,
      or unprepared-locale crash branch.
- [x] BUNDLE-A16 — Scope client message facades and locale entries by normalized application path,
      emit only that application's exact finite message set in each lazy locale payload, release
      inactive Svelte route loaders after navigation, keep SSR eager, and prove build/preview
      behavior on a three-message production route across English, Russian, and French.
- [x] BUNDLE-A17 — Encode scoped locale payloads as stable indexed arrays and compile numeric
      facade lookups so canonical message paths do not survive in browser chunks, while preserving
      sparse-locale fallback, parameterless values, parameterized calls, and legacy unscoped APIs.

### Local repository verification

- [x] LOCAL-A1 — Provide one root runner with ordered quick, full, and real-site profiles across
      every project/package in this repository.
- [x] LOCAL-A2 — Generate, type-check, build, and inspect the real site production graph locally.
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
- [-] TEXT-A10 — Complete all specified syntax/format/codegen golden cases.

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
- [-] P0-3 — Correct form IR plus JS/JSDoc/`.d.ts` runtime corpus.
- [x] P0-4 — Validated project capability as the only emitter entry.
- [x] P0-5 — Blocking CLI error severity.
- [x] P0-6 — Atomic cross-kind locale indexes.
- [x] P0-7 — Symbol-resolved workspace rename.
- [x] P0-8 — Safe `TempDir` test support.

### P1

- [x] P1-1 — Complete SvelteKit/ESM-only configuration and documentation cleanup.
- [-] P1-2 — Freeze the tested v0.1 language specification.
- [x] P1-3 — Carry recursive namespaces through generated output.
- [-] P1-4 — Complete multiline conformance across all backends.
- [x] P1-5 — Preserve call/declaration kinds, sources, and spans.
- [-] P1-6 — Finish the single project symbol/type/exhaustiveness database.
- [-] P1-7 — Make CLI and LSP consume that same database.
- [x] P1-8 — Implement one CLDR locale fallback service.

### P2

- [ ] P2-1 — Shared ECMAScript backend and `TypeModel`.
- [ ] P2-2 — End-to-end documentation propagation.
- [-] P2-3 — Emit both multiline modes from semantic text IR.
- [x] P2-4 — Collision-safe identifiers and filenames.
- [x] P2-5 — Bundler-visible lazy locale boundaries and deduplicated formatter data.
- [x] P2-6 — Compile-time typed `l` namespace transform. Recursive generated declarations type the
  source facade before Vite replaces exact static leaves and bounded dynamic paths; 45 plugin tests
  plus real-site positive/negative Svelte/TypeScript checks pass.
- [x] P2-7 — Positional plus named-object overloads.
- [x] P2-8 — Nested web config plus generated feature modules.
- [x] P2-9 — One shared browser/server locale transition plan.
- [x] P2-10 — Safe streaming-compatible link localization.
- [-] P2-11 — Fully incremental/cancellable namespace-aware LSP.
- [-] P2-12 — Native npm/VSIX plus universal WASM distribution.
- [x] P2-13 — Real TOML parsing and migration diagnostics.
- [x] P2-14 — Atomic writes and conflict-aware fixes.

### P3

- [-] P3-1 — Rust 1.76/stable and Linux/Windows/macOS/offline/package CI.
- [-] P3-2 — Parser, formatter, plural, and path property/fuzz coverage.
- [-] P3-3 — Full documentation-to-runtime conformance corpus. All 91 blocks are registered;
  Linguini syntax, canonical configuration, and the Getting Started JavaScript/TypeScript runtime
  execute; the SvelteKit provider typechecks, while packaged and remaining web-guide stages remain.
- [x] P3-4 — ICU/CLDR differential tests cover 152 representative cardinal cases across eight
  structurally distinct plural-rule families.
- [-] P3-5 — Vite/native VSIX/npm CLI/WASM integration matrix.
- [-] P3-6 — Dependency policy, pinned actions, reproducible JS packaging, synced versions.

## Final verification

- [ ] Run the complete Rust workspace test suite on Linux, Windows, and macOS.
- [x] Run strict Clippy for all targets and features.
- [x] Run Rust 1.76 MSRV checks and stable checks.
- [x] Run offline/package/source-archive verification.
- [ ] Run dependency, license, advisory, and public-API compatibility checks.
- [ ] Run Vite 5–8, SvelteKit, npm launcher, native VSIX, and WASM integration tests.
- [ ] Parse, analyze, generate, typecheck, and execute every normative documentation fixture.
- [ ] Re-read all 369 findings and every architecture criterion against final production paths.
- [ ] Mark this checklist complete only when no `[-]` or `[ ]` items remain.
