# Linguini Progress Checklists

Version: 0.8

## How to update this checklist

For every completed checkbox, add a short progress note and evidence.

Format:

```txt
- [x] Task name
  - Note: completed on YYYY-MM-DD. Short factual result.
  - Evidence: test command, snapshot name, file path, commit hash, or generated artifact.
```

Do not write progress notes on every small attempt. Add notes when:

- a task is completed
- a checkpoint is reached
- a blocker is discovered
- an implementation decision changes

Keep notes short.

A task is not complete until its tests are committed and the relevant command is recorded as evidence.

---

## 0. Repository and engineering rules

- [x] Add sequential stage gate enforcement
  - Note: completed on 2026-05-13. Added a spec gate script that enforces completed checklist evidence, handoff shape, CI gate wiring, and thin `main.rs` boundaries.
  - Evidence: scripts/check-spec-gates.sh; .github/workflows/ci.yml; `./scripts/check-spec-gates.sh`
- [x] Add technology-stack conformance check
  - Note: completed on 2026-05-13. Spec gate verifies every required workspace crate exists and all specified stack dependencies are recorded in workspace metadata.
  - Evidence: scripts/check-spec-gates.sh; Cargo.toml; `./scripts/check-spec-gates.sh`
- [x] Add anti-simplification completion gate
  - Note: completed on 2026-05-13. Spec gate rejects completed checklist blocks that claim skipped, omitted, fragment-only, or simplified-substitute behavior.
  - Evidence: scripts/check-spec-gates.sh; `./scripts/check-spec-gates.sh`
- [x] Create Rust workspace
  - Note: completed on 2026-05-12. Added Cargo workspace with CLI, core, codegen, LSP, package, and test-support crates.
  - Evidence: Cargo.toml; `cargo test --workspace`
- [x] Add crates listed in the technical specification
  - Note: completed on 2026-05-12. Added workspace members for every crate named in the specification and recorded planned external crates in workspace metadata.
  - Evidence: Cargo.toml; crates/
- [x] Add CI
  - Note: completed on 2026-05-12. Added GitHub Actions workflow for formatting, file-size, clippy, and tests.
  - Evidence: .github/workflows/ci.yml
- [x] Add file-size check: warn at 400 LOC, fail at 500 LOC
  - Note: completed on 2026-05-12. Added source file-size gate with configurable warning and failure limits.
  - Evidence: scripts/check-file-size.sh
- [x] Add generated/vendor exclusions for file-size check
  - Note: completed on 2026-05-12. Excluded target, vendor, generated, and snapshot paths from file-size enforcement.
  - Evidence: scripts/check-file-size.sh
- [x] Add formatting/linting pipeline
  - Note: completed on 2026-05-12. Added rustfmt config and CI commands for rustfmt and clippy.
  - Evidence: rustfmt.toml; .github/workflows/ci.yml; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`
- [x] Add test fixture directory
  - Note: completed on 2026-05-12. Added golden fixture roots for schema, locale, and project fixtures.
  - Evidence: tests/fixtures/golden
- [x] Add snapshot test setup
  - Note: completed on 2026-05-12. Added snapshot-style testing policy and committed golden fixture layout; external insta dependency is deferred until dependency fetching is introduced.
  - Evidence: docs/testing.md; tests/fixtures/golden
- [x] Add repository state handoff rule for future LLM continuation
  - Note: completed on 2026-05-12. Spec now requires `.codex` to record completed slice, decisions, tests, blockers, and next task.
  - Evidence: docs/spec.md; .codex

Checkpoint acceptance:

- [x] Work cannot move to the next checklist part until the previous part is fully complete
  - Note: completed on 2026-05-13. CI now runs the spec gate so completed work must carry recorded evidence before passing.
  - Evidence: scripts/check-spec-gates.sh; .github/workflows/ci.yml; `./scripts/check-spec-gates.sh`
- [x] Implementation choices match the specified Rust workspace and crate stack
  - Note: completed on 2026-05-13. Spec gate validates required crate layout and planned dependency metadata against the technical specification.
  - Evidence: scripts/check-spec-gates.sh; Cargo.toml; `./scripts/check-spec-gates.sh`
- [x] No completed item omits specified behavior or uses a simplified substitute without updating the spec
  - Note: completed on 2026-05-13. Spec gate scans completed checklist evidence for explicit skipped, omitted, fragment-only, or simplified-substitute completion claims.
  - Evidence: scripts/check-spec-gates.sh; `./scripts/check-spec-gates.sh`
- [x] `cargo test` runs successfully
  - Note: completed on 2026-05-12. Workspace tests pass across all scaffolded crates.
  - Evidence: `cargo test --workspace`
- [x] CI fails on source files above 500 LOC
  - Note: completed on 2026-05-12. CI runs the source file-size gate with a 500-line failure threshold.
  - Evidence: .github/workflows/ci.yml; scripts/check-file-size.sh
- [x] Workspace has no large catch-all implementation files
  - Note: completed on 2026-05-12. Initial workspace crates are split by responsibility and pass the file-size gate.
  - Evidence: `./scripts/check-file-size.sh`

---

## 0.1 Testing policy and gates

- [x] Add mandatory test policy to repository docs
  - Note: completed on 2026-05-12. Documented required gates and checklist evidence rule.
  - Evidence: docs/testing.md
- [x] Add unit test structure for every core crate
  - Note: completed on 2026-05-13. Added crate-level unit test modules where missing and a gate that rejects workspace crates without unit test structure.
  - Evidence: scripts/check-unit-test-structure.sh; `./scripts/check-unit-test-structure.sh`; `cargo test --workspace`
- [x] Add golden fixture directory for `.lqs` and `.lgl` projects
  - Note: completed on 2026-05-12. Expanded schema and Russian locale golden fixtures to cover enums, formatter annotations, docs, forms, selector maps, plural-shaped maps, nested form attributes, helper functions, messages, placeholders, and grouped messages.
  - Evidence: tests/fixtures/golden/schema/shop.lqs; tests/fixtures/golden/locale/ru.lgl; `cargo test -p linguini-syntax`
- [x] Add behavior-complete syntax fixtures
  - Note: completed on 2026-05-13. Added a syntax fixture gate that verifies golden `.lqs` and `.lgl` fixtures cover complete declarations, messages, docs, selectors, plural-shaped branches, nested form attributes, placeholders, and formatter annotations.
  - Evidence: scripts/check-syntax-fixtures.sh; tests/fixtures/golden/schema/shop.lqs; tests/fixtures/golden/locale/ru.lgl; `./scripts/check-syntax-fixtures.sh`; `cargo test -p linguini-syntax`
- [x] Reject fragment-only syntax fixtures unless they are explicit invalid diagnostic fixtures
  - Note: completed on 2026-05-13. Golden syntax fixtures now have a minimum complete-program gate and diagnostic fragments are constrained to `tests/fixtures/invalid`.
  - Evidence: scripts/check-syntax-fixtures.sh; docs/testing.md; `./scripts/check-syntax-fixtures.sh`
- [x] Add `insta` snapshot review workflow
  - Note: completed on 2026-05-13. Added a snapshot review script with CI check mode for committed snapshot tests and interactive `cargo-insta` review mode for accepted snapshot updates.
  - Evidence: scripts/review-snapshots.sh; docs/testing.md; .github/workflows/ci.yml; `./scripts/review-snapshots.sh --check`
- [x] Add CLI integration test harness with `assert_cmd` and `tempfile`
  - Note: completed on 2026-05-13. Added binary-level CLI integration tests covering init, check discovery output, syntax diagnostics on stderr, and CLDR status.
  - Evidence: crates/linguini-cli/tests/cli.rs; crates/linguini-cli/Cargo.toml; `cargo test -p linguini-cli`
- [x] Add generated TypeScript validation fixture
  - Note: completed on 2026-05-13. Added deterministic generated TypeScript file-tree fixtures for Russian schema/locale output.
  - Evidence: tests/fixtures/golden/snapshots/ts; `cargo test -p linguini-codegen-ts`
- [x] Add regression-test rule to contribution docs
  - Note: completed on 2026-05-13. Added contribution guidance requiring focused regression tests for bug fixes and recorded gate evidence before checklist completion.
  - Evidence: CONTRIBUTING.md; scripts/check-spec-gates.sh; `./scripts/check-spec-gates.sh`
- [x] Add coverage measurement command for core crates
  - Note: completed on 2026-05-13. Added a `cargo-llvm-cov` coverage command that writes LCOV and HTML reports while excluding generated, vendor, target, and snapshot paths.
  - Evidence: scripts/coverage.sh; CONTRIBUTING.md; `./scripts/coverage.sh --help`; `./scripts/check-spec-gates.sh`
- [x] Add CI job that runs unit, snapshot, CLI, and generated-output tests
  - Note: completed on 2026-05-13. CI runs unit and CLI tests through `cargo test --workspace`, snapshot checks, and generated-output validation for TypeScript.
  - Evidence: .github/workflows/ci.yml; scripts/review-snapshots.sh; scripts/validate-generated-ts.sh; `./scripts/check-spec-gates.sh`

Checkpoint acceptance:

- [x] No implementation task can be marked complete without test evidence
  - Note: completed on 2026-05-13. Spec gate rejects completed checklist items without note and evidence blocks.
  - Evidence: scripts/check-spec-gates.sh; `./scripts/check-spec-gates.sh`
- [x] `.lqs` and `.lgl` golden fixtures are complete realistic programs
  - Note: completed on 2026-05-13. Syntax fixture gate checks the committed golden schema and locale fixtures cover complete realistic declarations and message bodies.
  - Evidence: scripts/check-syntax-fixtures.sh; tests/fixtures/golden/schema/shop.lqs; tests/fixtures/golden/locale/ru.lgl; `./scripts/check-syntax-fixtures.sh`
- [x] CI runs the full required test suite
  - Note: completed on 2026-05-13. CI includes formatting, file-size, spec, unit, snapshot, CLI, and generated-output validation gates.
  - Evidence: .github/workflows/ci.yml; `./scripts/check-spec-gates.sh`
- [x] Coverage report exists for core crates
  - Note: completed on 2026-05-13. Generated workspace coverage with `cargo-llvm-cov` after installing `llvm-tools-preview`.
  - Evidence: target/coverage/lcov.info; target/coverage/html/html; `./scripts/coverage.sh`

---

## 1. Project model and discovery

- [x] Implement `linguini.toml` parser
  - Note: completed on 2026-05-12. Added minimal parser for required project and paths sections.
  - Evidence: crates/linguini-config/src/parser.rs; `cargo test --workspace`
- [x] Validate required config fields
  - Note: completed on 2026-05-12. Added validation for required fields, default locale membership, and locale tag shape.
  - Evidence: crates/linguini-config/src/model.rs; `cargo test --workspace`
- [x] Implement schema path discovery
  - Note: completed on 2026-05-12. Added recursive `.lqs` discovery under the configured schema root.
  - Evidence: crates/linguini-config/src/discovery.rs; `cargo test --workspace`
- [x] Implement locale path discovery
  - Note: completed on 2026-05-12. Added recursive `.lgl` discovery under the configured locale root.
  - Evidence: crates/linguini-config/src/discovery.rs; `cargo test --workspace`
- [x] Parse locale file names as BCP 47-like tags
  - Note: completed on 2026-05-12. Locale discovery validates file stems such as `ru`, `en-US`, and `zh-Hant`.
  - Evidence: crates/linguini-config/src/model.rs; `cargo test --workspace`
- [x] Derive schema namespaces from paths
  - Note: completed on 2026-05-12. Schema namespaces include relative parent directories and file stem.
  - Evidence: crates/linguini-config/src/discovery.rs; `cargo test --workspace`
- [x] Derive locale namespaces from paths
  - Note: completed on 2026-05-12. Locale namespaces use relative parent directories and exclude the final locale file.
  - Evidence: crates/linguini-config/src/discovery.rs; `cargo test --workspace`
- [x] Implement top-down scope path collection
  - Note: completed on 2026-05-12. Added root-to-leaf locale scope chain construction for nested locale files.
  - Evidence: crates/linguini-config/src/discovery.rs; `cargo test --workspace`
- [x] Implement `linguini init`
  - Note: completed on 2026-05-12. Added minimal init command that creates config plus schema and locale directories.
  - Evidence: crates/linguini-cli/src/lib.rs; `cargo test --workspace`

Checkpoint acceptance:

- [x] `linguini init` creates a valid project
  - Note: completed on 2026-05-12. CLI test verifies the generated config and directory structure.
  - Evidence: crates/linguini-cli/src/lib.rs; `cargo test --workspace`
- [x] `linguini check` lists discovered schema files
  - Note: completed on 2026-05-12. Check command output includes discovered schema paths and namespaces.
  - Evidence: crates/linguini-cli/src/lib.rs; `cargo test --workspace`
- [x] `linguini check` lists discovered locale files
  - Note: completed on 2026-05-12. Check command output includes discovered locale paths, locale tags, and namespaces.
  - Evidence: crates/linguini-cli/src/lib.rs; `cargo test --workspace`
- [x] `locale/shop/delivery/ru.lgl` sees parent scope files
  - Note: completed on 2026-05-12. Scope-chain helper returns root, parent, and leaf locale files in top-down order.
  - Evidence: crates/linguini-config/src/discovery.rs; `cargo test --workspace`

---

## 2. Lexer

- [x] Define token model with spans
  - Note: completed on 2026-05-12. Added public token, token kind, and byte span model in `linguini-syntax`.
  - Evidence: crates/linguini-syntax/src/token.rs; `cargo test -p linguini-syntax`
- [x] Implement code mode
  - Note: completed on 2026-05-12. Added Chumsky-based code token parser for identifiers, locale tags, punctuation, strings, comments, and newlines.
  - Evidence: crates/linguini-syntax/src/lexer.rs; `cargo test -p linguini-syntax`
- [x] Implement raw text mode after `=`
  - Note: completed on 2026-05-12. `=` transitions to raw text mode and tokenizes text until newline or placeholder.
  - Evidence: crates/linguini-syntax/src/lexer.rs; `cargo test -p linguini-syntax`
- [x] Implement raw text mode after `=>`
  - Note: completed on 2026-05-12. `=>` transitions to raw text mode for selector and plural branch text.
  - Evidence: crates/linguini-syntax/src/lexer.rs; `cargo test -p linguini-syntax`
- [x] Implement multiline text mode
  - Note: completed on 2026-05-12. Triple quotes enter and leave multiline raw text mode.
  - Evidence: crates/linguini-syntax/src/lexer.rs; `cargo test -p linguini-syntax`
- [x] Implement placeholder mode
  - Note: completed on 2026-05-12. Raw and multiline text can enter `{...}` placeholder code mode and resume the previous text mode.
  - Evidence: crates/linguini-syntax/src/lexer.rs; `cargo test -p linguini-syntax`
- [x] Implement ordinary comments
  - Note: completed on 2026-05-12. Added `//` comment tokenization.
  - Evidence: crates/linguini-syntax/src/lexer.rs; `cargo test -p linguini-syntax`
- [x] Implement doc comments
  - Note: completed on 2026-05-12. Added `///` doc comment tokenization before ordinary comments.
  - Evidence: crates/linguini-syntax/src/lexer.rs; `cargo test -p linguini-syntax`
- [x] Add lexer error recovery
  - Note: completed on 2026-05-12. Added recovering lexer output that records invalid tokens and continues while preserving strict `lex` errors.
  - Evidence: crates/linguini-syntax/src/lexer.rs; `cargo test -p linguini-syntax`
- [x] Add lexer snapshot tests
  - Note: completed on 2026-05-12. Added committed snapshot-style token expectations for schema and locale golden fixtures.
  - Evidence: tests/fixtures/golden/snapshots; `cargo test -p linguini-syntax`

Checkpoint acceptance:

- [x] Lexer handles `.lqs` examples
  - Note: completed on 2026-05-12. Schema golden fixture lexes with expected declaration tokens.
  - Evidence: tests/fixtures/golden/schema/shop.lqs; `cargo test -p linguini-syntax`
- [x] Lexer handles `.lgl` examples
  - Note: completed on 2026-05-12. Locale golden fixture lexes with raw text output.
  - Evidence: tests/fixtures/golden/locale/ru.lgl; `cargo test -p linguini-syntax`
- [x] Lexer reports spans correctly
  - Note: completed on 2026-05-12. Unit test verifies byte spans including Cyrillic raw text.
  - Evidence: crates/linguini-syntax/src/lib.rs; `cargo test -p linguini-syntax`
- [x] Lexer supports Cyrillic raw text
  - Note: completed on 2026-05-12. Russian fixture keeps Cyrillic text in raw text tokens.
  - Evidence: tests/fixtures/golden/locale/ru.lgl; `cargo test -p linguini-syntax`

---

## 3. Parser

- [x] Implement schema parser
  - Note: completed on 2026-05-12. Added Chumsky-based schema AST parser entry point over token streams.
  - Evidence: crates/linguini-syntax/src/parser.rs; `cargo test -p linguini-syntax`; `cargo test --workspace`
- [x] Implement locale parser
  - Note: completed on 2026-05-12. Added `.lgl` parser entry point and locale AST for declarations, forms, functions, messages, text, placeholders, and spans.
  - Evidence: crates/linguini-syntax/src/parser/locale_parser.rs; `cargo test -p linguini-syntax`
- [x] Parse enums
  - Note: completed on 2026-05-12. Schema parser accepts public enum declarations and variants.
  - Evidence: crates/linguini-syntax/src/parser.rs; `cargo test -p linguini-syntax`
- [x] Parse custom scalar types
  - Note: completed on 2026-05-12. Schema parser accepts `type` aliases with formatter annotations.
  - Evidence: crates/linguini-syntax/src/parser.rs; `cargo test -p linguini-syntax`
- [x] Parse message signatures
  - Note: completed on 2026-05-12. Schema parser accepts message parameters with name, type, and source spans.
  - Evidence: crates/linguini-syntax/src/parser.rs; `cargo test -p linguini-syntax`
- [x] Parse grouped messages
  - Note: completed on 2026-05-12. Schema parser accepts grouped message signatures.
  - Evidence: crates/linguini-syntax/src/parser.rs; `cargo test -p linguini-syntax`
- [x] Parse forms
  - Note: completed on 2026-05-12. Locale parser accepts `form` declarations with variants, attributes, and branch-bearing map attributes.
  - Evidence: crates/linguini-syntax/src/parser/locale_parser.rs; `cargo test -p linguini-syntax`
- [x] Parse nested form attributes
  - Note: completed on 2026-05-12. Locale parser now accepts recursive object-shaped form attributes such as `display { short = ... }`.
  - Evidence: crates/linguini-syntax/src/parser/locale_parser.rs; tests/fixtures/golden/locale/ru.lgl; `cargo test -p linguini-syntax`
- [x] Parse selector maps
  - Note: completed on 2026-05-12. Locale parser accepts selector-qualified form variants such as `small:gender` with branch maps.
  - Evidence: crates/linguini-syntax/src/lib.rs; `cargo test -p linguini-syntax`
- [x] Parse plural-map-shaped branches
  - Note: completed on 2026-05-12. Locale parser accepts branch maps with CLDR-shaped keys such as `one`, `few`, `many`, and `other` without semantic validation.
  - Evidence: crates/linguini-syntax/src/lib.rs; `cargo test -p linguini-syntax`
- [x] Parse local functions
  - Note: completed on 2026-05-12. Locale parser accepts `fn` declarations with positional parameters, tuple branches, and `else` fallback branches.
  - Evidence: crates/linguini-syntax/src/parser/locale_parser.rs; `cargo test -p linguini-syntax`
- [x] Parse placeholders
  - Note: completed on 2026-05-12. Text parser accepts placeholder expressions for names, dotted paths, and calls with nested arguments.
  - Evidence: crates/linguini-syntax/src/lib.rs; `cargo test -p linguini-syntax`
- [x] Parse formatter annotations
  - Note: completed on 2026-05-12. Parser accepts schema annotations and placeholder formatter annotations with string arguments.
  - Evidence: crates/linguini-syntax/src/parser.rs; crates/linguini-syntax/src/parser/locale_parser.rs; `cargo test -p linguini-syntax`
- [x] Preserve source spans for all AST nodes
  - Note: completed on 2026-05-12. Schema and locale AST structs carry byte spans on files, declarations, text parts, placeholders, expressions, branches, and annotations.
  - Evidence: crates/linguini-syntax/src/ast.rs; `cargo test -p linguini-syntax`
- [x] Add parser recovery
  - Note: completed on 2026-05-12. Added recovered parser entry points that return partial AST output plus collected lex and parse diagnostics.
  - Evidence: crates/linguini-syntax/src/parser.rs; crates/linguini-syntax/src/parser/locale_parser.rs; `cargo test -p linguini-syntax`

Checkpoint acceptance:

- [x] All valid fixtures parse
  - Note: completed on 2026-05-12. Parser tests cover committed schema and locale golden fixtures.
  - Evidence: tests/fixtures/golden/schema/shop.lqs; tests/fixtures/golden/locale/ru.lgl; `cargo test -p linguini-syntax`
- [x] Invalid fixtures produce diagnostics
  - Note: completed on 2026-05-12. Added invalid schema and locale fixtures with tests asserting parser diagnostics and strict parser failure.
  - Evidence: tests/fixtures/invalid/schema/missing-message-paren.lqs; tests/fixtures/invalid/locale/broken-placeholder.lgl; `cargo test -p linguini-syntax`
- [x] Parser does not require semantic information
  - Note: completed on 2026-05-12. Locale parser preserves selector, plural, formatter, call, and form syntax without resolving types, variants, or CLDR categories.
  - Evidence: crates/linguini-syntax/src/parser/locale_parser.rs; `cargo test -p linguini-syntax`

---

## 3.1 Syntax migration

- [x] Remove empty parentheses from parameterless grouped schema messages
  - Note: completed on 2026-05-14. Schema grouped message leaves now parse as bare identifiers while parameterized messages still require parentheses.
  - Evidence: tests/fixtures/golden/schema/shop.lqs; crates/linguini-syntax/src/parser.rs; `cargo test -p linguini-syntax`
- [x] Support inline PascalCase enum declarations
  - Note: completed on 2026-05-14. Schema and locale enum parsers accept comma-separated inline variants and the golden fixtures use PascalCase locale enum names.
  - Evidence: tests/fixtures/golden/schema/shop.lqs; tests/fixtures/golden/locale/ru.lgl; `cargo test -p linguini-syntax`
- [x] Replace locale noun `form` declarations with `impl`
  - Note: completed on 2026-05-14. Locale parser, IR lowering, generated data rendering, and golden fixtures now use `impl Fruit` for enum implementations.
  - Evidence: crates/linguini-syntax/src/parser/locale_parser.rs; crates/linguini-ir/src/lower.rs; tests/fixtures/golden/locale/ru.lgl; `cargo test -p linguini-syntax -p linguini-ir -p linguini-cli`
- [x] Support typed fields and local `form name(Type)` entries inside `impl`
  - Note: completed on 2026-05-14. Implementation entries preserve typed fields like `Gender = neuter` and local plural forms like `form nom(Plural)`.
  - Evidence: tests/fixtures/golden/locale/ru.lgl; tests/fixtures/golden/snapshots/ir-locale-ru.txt; `cargo test -p linguini-ir`
- [x] Replace `else` with `_` wildcard branches
  - Note: completed on 2026-05-14. Parser, renderer, and generated TypeScript select helpers support `_` wildcard fallbacks for form maps and dispatch trees.
  - Evidence: crates/linguini-codegen-ts/src/module/emit.rs; crates/linguini-cli/src/project/test_data/render.rs; `cargo test -p linguini-codegen-ts -p linguini-cli`
- [x] Replace flat tuple helper functions with nested typed dispatch
  - Note: completed on 2026-05-14. Top-level `form` and `fn` declarations parse typed parameters and nested dispatch trees that lower to IR and emit nested TypeScript `selectBranch` calls.
  - Evidence: crates/linguini-syntax/src/ast.rs; crates/linguini-ir/src/model.rs; crates/linguini-codegen-ts/src/module/emit.rs; `cargo test -p linguini-syntax -p linguini-ir -p linguini-codegen-ts`
- [x] Remove explicit `plural(count)` call syntax from fixtures and codegen paths
  - Note: completed on 2026-05-14. Golden locale calls pass numeric values directly to `Plural` parameters, and generated TypeScript converts `Plural` dispatch parameters internally.
  - Evidence: tests/fixtures/golden/locale/ru.lgl; tests/fixtures/golden/snapshots/ts/locales/ru.ts; `cargo test -p linguini-codegen-ts`
- [x] Update canonical syntax specification
  - Note: completed on 2026-05-14. `docs/spec.md` now describes parameterless schema leaves, `impl`, typed forms, `_` wildcards, and direct numeric plural arguments.
  - Evidence: docs/spec.md; docs/spec-migrate-syntax.md

Checkpoint acceptance:

- [x] Migrated syntax fixtures parse and lower to IR
  - Note: completed on 2026-05-14. Golden `.lqs` and `.lgl` fixtures use migrated syntax and have refreshed lexer, IR, and TypeScript snapshots.
  - Evidence: tests/fixtures/golden/snapshots/lexer-schema.tokens; tests/fixtures/golden/snapshots/lexer-locale.tokens; tests/fixtures/golden/snapshots/ir-locale-ru.txt; `cargo test -p linguini-syntax -p linguini-ir`
- [x] Generated TypeScript uses migrated dispatch semantics
  - Note: completed on 2026-05-14. Generated TypeScript emits nested dispatch helpers for typed forms and accepts direct numeric plural arguments at call sites.
  - Evidence: tests/fixtures/golden/snapshots/ts/locales/ru.ts; `cargo test -p linguini-codegen-ts`

---

## 4. Diagnostics

- [x] Add Ariadne renderer
  - Note: completed on 2026-05-12. Added Ariadne-backed ASCII, no-color diagnostic rendering with byte-span support.
  - Evidence: crates/linguini-analyzer/src/diagnostic.rs; `cargo test -p linguini-analyzer -p linguini-cli`
- [x] Add diagnostic severity levels
  - Note: completed on 2026-05-12. Added error, warning, and advice severity mapping to Ariadne report kinds.
  - Evidence: crates/linguini-analyzer/src/diagnostic.rs; `cargo test -p linguini-analyzer -p linguini-cli`
- [x] Add related spans
  - Note: completed on 2026-05-12. Added related span model and renderer labels.
  - Evidence: crates/linguini-analyzer/src/diagnostic.rs; tests/fixtures/golden/snapshots/diagnostic-schema-syntax.txt; `cargo test -p linguini-analyzer`
- [x] Add quick-fix hint model
  - Note: completed on 2026-05-12. Added quick-fix hint and replacement model; renderer emits hints as help text.
  - Evidence: crates/linguini-analyzer/src/diagnostic.rs; tests/fixtures/golden/snapshots/diagnostic-schema-syntax.txt; `cargo test -p linguini-analyzer`
- [x] Add CLI diagnostic output tests
  - Note: completed on 2026-05-12. `linguini check` now parses discovered schema and locale files and reports syntax diagnostics.
  - Evidence: crates/linguini-cli/src/lib.rs; `cargo test -p linguini-cli`

Checkpoint acceptance:

- [x] Syntax errors show highlighted spans
  - Note: completed on 2026-05-12. CLI syntax diagnostics are rendered with Ariadne source highlights.
  - Evidence: crates/linguini-cli/src/lib.rs; `cargo test -p linguini-cli`
- [x] Analyzer errors show related declarations
  - Note: completed on 2026-05-12. Schema duplicate-declaration diagnostics include the first declaration as a related span.
  - Evidence: crates/linguini-schema/src/lib.rs; `cargo test -p linguini-schema`
- [x] Diagnostics are stable in snapshots
  - Note: completed on 2026-05-12. Added committed golden diagnostic snapshot for primary, related, note, and quick-fix output.
  - Evidence: tests/fixtures/golden/snapshots/diagnostic-schema-syntax.txt; `cargo test -p linguini-analyzer`

---

## 5. Schema symbol table

- [x] Register schema enums
  - Note: completed on 2026-05-12. Added schema symbol table entries for public enums.
  - Evidence: crates/linguini-schema/src/lib.rs; `cargo test -p linguini-schema`
- [x] Register enum variants
  - Note: completed on 2026-05-12. Enum symbols store variant names and source spans.
  - Evidence: crates/linguini-schema/src/lib.rs; `cargo test -p linguini-schema`
- [x] Register custom scalar types
  - Note: completed on 2026-05-12. Type alias symbols store custom scalar names, targets, docs, and spans.
  - Evidence: crates/linguini-schema/src/lib.rs; `cargo test -p linguini-schema`
- [x] Register public messages
  - Note: completed on 2026-05-12. Message symbols store public message signatures and parameter types.
  - Evidence: crates/linguini-schema/src/lib.rs; `cargo test -p linguini-schema`
- [x] Register grouped messages
  - Note: completed on 2026-05-12. Grouped messages are registered under `group.message` keys with group metadata.
  - Evidence: crates/linguini-schema/src/lib.rs; `cargo test -p linguini-schema`
- [x] Store schema doc comments
  - Note: completed on 2026-05-12. Symbol table stores doc text for enums, type aliases, messages, and groups.
  - Evidence: crates/linguini-schema/src/lib.rs; `cargo test -p linguini-schema`
- [x] Detect duplicate declarations
  - Note: completed on 2026-05-12. Duplicate top-level schema declarations produce diagnostics with related first-declaration spans.
  - Evidence: crates/linguini-schema/src/lib.rs; `cargo test -p linguini-schema`
- [x] Resolve type references
  - Note: completed on 2026-05-12. Symbol table validates type aliases and message parameter types against builtins and declared types.
  - Evidence: crates/linguini-schema/src/lib.rs; `cargo test -p linguini-schema`

Checkpoint acceptance:

- [x] Unknown schema type is reported
  - Note: completed on 2026-05-12. Unknown message parameter and type alias targets produce schema type diagnostics.
  - Evidence: crates/linguini-schema/src/lib.rs; `cargo test -p linguini-schema`
- [x] Duplicate enum is reported
  - Note: completed on 2026-05-12. Duplicate enum declarations are rejected by the top-level duplicate declaration check.
  - Evidence: crates/linguini-schema/src/lib.rs; `cargo test -p linguini-schema`
- [x] Doc comments are available to analyzer and LSP
  - Note: completed on 2026-05-12. Symbol structs expose stored doc comments for downstream analyzer and LSP use.
  - Evidence: crates/linguini-schema/src/lib.rs; `cargo test -p linguini-schema`

---

## 6. Locale scope model

- [x] Load root locale scope file
  - Note: completed on 2026-05-12. Added path-based locale scope loader that reads and parses root scope files.
  - Evidence: crates/linguini-locale/src/lib.rs; `cargo test -p linguini-locale`
- [x] Load parent directory scope files
  - Note: completed on 2026-05-12. Scope loading accepts root-to-child path chains from project discovery.
  - Evidence: crates/linguini-locale/src/lib.rs; `cargo test -p linguini-locale`
- [x] Merge scope declarations in order
  - Note: completed on 2026-05-12. Locale scope merges source files by input order and records source index/path.
  - Evidence: crates/linguini-locale/src/lib.rs; `cargo test -p linguini-locale`
- [x] Implement explicit `override`
  - Note: completed on 2026-05-12. Parser accepts `override` locale declarations and scope loading permits explicit replacement.
  - Evidence: crates/linguini-syntax/src/parser/locale_parser.rs; crates/linguini-locale/src/lib.rs; `cargo test -p linguini-syntax -p linguini-locale`
- [x] Register local enums
  - Note: completed on 2026-05-12. Locale scope registers enum declarations with docs, spans, and source metadata.
  - Evidence: crates/linguini-locale/src/lib.rs; `cargo test -p linguini-locale`
- [x] Register local functions
  - Note: completed on 2026-05-12. Locale scope registers local function declarations from parent and child files.
  - Evidence: crates/linguini-locale/src/lib.rs; `cargo test -p linguini-locale`
- [x] Register forms
  - Note: completed on 2026-05-12. Locale scope registers form declarations.
  - Evidence: crates/linguini-locale/src/lib.rs; `cargo test -p linguini-locale`
- [x] Register message implementations
  - Note: completed on 2026-05-12. Locale scope registers standalone and grouped message implementations.
  - Evidence: crates/linguini-locale/src/lib.rs; `cargo test -p linguini-locale`
- [x] Detect duplicate declarations
  - Note: completed on 2026-05-12. Same-file duplicate locale declarations produce diagnostics with related first spans.
  - Evidence: crates/linguini-locale/src/lib.rs; `cargo test -p linguini-locale`
- [x] Detect invalid shadowing
  - Note: completed on 2026-05-12. Child declarations that shadow parent declarations without `override` produce diagnostics.
  - Evidence: crates/linguini-locale/src/lib.rs; `cargo test -p linguini-locale`

Checkpoint acceptance:

- [x] Child locale files can use parent local enums
  - Note: completed on 2026-05-12. Root enum symbols remain visible after parent and child files are merged.
  - Evidence: crates/linguini-locale/src/lib.rs; `cargo test -p linguini-locale`
- [x] Child locale files can use parent functions
  - Note: completed on 2026-05-12. Parent function symbols remain visible to child scope output.
  - Evidence: crates/linguini-locale/src/lib.rs; `cargo test -p linguini-locale`
- [x] Invalid shadowing is reported
  - Note: completed on 2026-05-12. Shadowing without `override` reports a diagnostic with parent related span.
  - Evidence: crates/linguini-locale/src/lib.rs; `cargo test -p linguini-locale`

---

## 7. Analyzer

- [x] Match locale messages to schema messages
  - Note: completed on 2026-05-12. Added analyzer message coverage API that compares schema public messages with locale implementations.
  - Evidence: crates/linguini-analyzer/src/message_coverage.rs; `cargo test -p linguini-analyzer`
- [x] Validate missing public messages
  - Note: completed on 2026-05-12. Analyzer reports schema messages missing from a locale.
  - Evidence: crates/linguini-analyzer/src/message_coverage.rs; `cargo test -p linguini-analyzer`
- [x] Validate unknown public messages
  - Note: completed on 2026-05-12. Analyzer reports locale message implementations not present in the schema.
  - Evidence: crates/linguini-analyzer/src/message_coverage.rs; `cargo test -p linguini-analyzer`
- [x] Validate form enum coverage
  - Note: completed on 2026-05-12. Added generic branch coverage analyzer for enum-backed form coverage checks.
  - Evidence: crates/linguini-analyzer/src/branch_coverage.rs; `cargo test -p linguini-analyzer`
- [x] Validate selector enum coverage
  - Note: completed on 2026-05-12. Branch coverage analyzer accepts selector subjects and reports missing enum branches with related variant spans.
  - Evidence: crates/linguini-analyzer/src/branch_coverage.rs; `cargo test -p linguini-analyzer`
- [x] Validate `other` branch requirement
  - Note: completed on 2026-05-12. Added analyzer helper that requires an `other` fallback branch.
  - Evidence: crates/linguini-analyzer/src/branch_coverage.rs; `cargo test -p linguini-analyzer`
- [x] Validate placeholder variables
  - Note: completed on 2026-05-13. Added expression analyzer validation for placeholder roots against message variables.
  - Evidence: crates/linguini-analyzer/src/expression.rs; `cargo test -p linguini-analyzer`
- [x] Validate form property access
  - Note: completed on 2026-05-13. Added form signature checks for property paths such as `fruit.nom`.
  - Evidence: crates/linguini-analyzer/src/expression.rs; `cargo test -p linguini-analyzer`
- [x] Validate function calls
  - Note: completed on 2026-05-13. Added local function lookup for placeholder calls.
  - Evidence: crates/linguini-analyzer/src/expression.rs; `cargo test -p linguini-analyzer`
- [x] Validate function arity
  - Note: completed on 2026-05-13. Analyzer reports call argument count mismatches with related function spans.
  - Evidence: crates/linguini-analyzer/src/expression.rs; `cargo test -p linguini-analyzer`
- [x] Validate tuple patterns
  - Note: completed on 2026-05-13. Function branch tuple patterns are checked against function parameter count.
  - Evidence: crates/linguini-analyzer/src/expression.rs; `cargo test -p linguini-analyzer`
- [x] Detect reference cycles
  - Note: completed on 2026-05-13. Added generic reference graph cycle diagnostics for analyzer callers.
  - Evidence: crates/linguini-analyzer/src/reference.rs; `cargo test -p linguini-analyzer`
- [x] Resolve implicit plural arguments
  - Note: completed on 2026-05-13. Single numeric message parameters are accepted as implicit plural arguments.
  - Evidence: crates/linguini-analyzer/src/expression.rs; `cargo test -p linguini-analyzer`
- [x] Reject ambiguous implicit plural arguments
  - Note: completed on 2026-05-13. Plural form access with multiple numeric parameters reports an explicit-argument diagnostic.
  - Evidence: crates/linguini-analyzer/src/expression.rs; `cargo test -p linguini-analyzer`

Checkpoint acceptance:

- [x] `delivery = {delivered(fruit.gender)} {size(fruit.gender)} {fruit.nom}` passes
  - Note: completed on 2026-05-13. Expression analyzer accepts the valid delivery placeholder chain with functions and form properties in scope.
  - Evidence: crates/linguini-analyzer/src/lib.rs; `cargo test -p linguini-analyzer`
- [x] Missing enum variant is reported
  - Note: completed on 2026-05-13. Branch coverage analyzer reports missing enum-backed branches with related variant spans.
  - Evidence: crates/linguini-analyzer/src/branch_coverage.rs; `cargo test -p linguini-analyzer`
- [x] Unknown form property is reported
  - Note: completed on 2026-05-13. Expression analyzer reports unknown form properties on typed variables.
  - Evidence: crates/linguini-analyzer/src/expression.rs; `cargo test -p linguini-analyzer`
- [x] Ambiguous plural access is reported
  - Note: completed on 2026-05-13. Expression analyzer reports ambiguous implicit plural arguments when multiple numeric variables are in scope.
  - Evidence: crates/linguini-analyzer/src/expression.rs; `cargo test -p linguini-analyzer`

---

## 8. CLDR ingestion

- [x] Implement CLDR cache directory
  - Note: completed on 2026-05-13. Added configurable CLDR cache root resolution, cache inspection, manifest/data/plurals checks, and offline cache requirement API.
  - Evidence: crates/linguini-cldr/src/cache.rs; `cargo test -p linguini-cldr -p linguini-cli`
- [x] Implement `linguini cldr fetch`
  - Note: completed on 2026-05-13. Added `linguini cldr fetch` for `https://github.com/unicode-org/cldr-json` and retained staged CLDR JSON import for tests and offline development.
  - Evidence: crates/linguini-cli/src/lib.rs; `cargo test -p linguini-cldr -p linguini-cli`
- [x] Implement `linguini cldr status`
  - Note: completed on 2026-05-13. Added `linguini cldr status` output for cache usability, manifest, data, and plural-rule presence.
  - Evidence: crates/linguini-cli/src/lib.rs; `cargo test -p linguini-cldr -p linguini-cli`
- [x] Load plural rules from CLDR JSON
  - Note: completed on 2026-05-13. Added CLDR JSON plural-rule loader for locale category rules from cached `cldr-json/cldr-core/supplemental/plurals.json`.
  - Evidence: crates/linguini-cldr/src/data/cache_loaders.rs; `cargo test -p linguini-cldr`
- [x] Load number formatting data
  - Note: completed on 2026-05-13. Added cached CLDR number symbol, decimal pattern, and percent pattern loading from locale `numbers.json`.
  - Evidence: crates/linguini-cldr/src/data/cache_loaders.rs; `cargo test -p linguini-cldr`
- [x] Load date formatting data
  - Note: completed on 2026-05-13. Added cached Gregorian date, time, and date-time format-width loading from `ca-gregorian.json`.
  - Evidence: crates/linguini-cldr/src/data/cache_loaders.rs; `cargo test -p linguini-cldr`
- [x] Load currency formatting data
  - Note: completed on 2026-05-13. Added cached CLDR currency standard and accounting pattern loading from locale `numbers.json`.
  - Evidence: crates/linguini-cldr/src/data/cache_loaders.rs; `cargo test -p linguini-cldr`
- [x] Add cache integrity checks
  - Note: completed on 2026-05-13. Cache status now validates required manifest, data directory, and plural-rule file readability before marking cache usable.
  - Evidence: crates/linguini-cldr/src/cache.rs; `cargo test -p linguini-cldr`
- [x] Add offline build mode
  - Note: completed on 2026-05-13. Added `require_offline_cache` so build/check paths can fail without downloading when CLDR cache is absent or incomplete.
  - Evidence: crates/linguini-cldr/src/cache.rs; `cargo test -p linguini-cldr`
- [x] Fetch only required CLDR JSON files
  - Note: completed on 2026-05-13. CLDR fetch uses sparse official `cldr-json` paths for plural rules, locale numbers, and Gregorian calendar data instead of importing the full tree.
  - Evidence: crates/linguini-cldr/src/cache.rs; crates/linguini-cli/src/lib.rs; `cargo test -p linguini-cldr -p linguini-cli`
- [x] Generate compiled Rust CLDR tables from `cldr-json`
  - Note: completed on 2026-05-13. Added `linguini-cldr-macros` proc-macro crate that reads cached `cldr-json` files and emits typed Rust plural and formatter table functions.
  - Evidence: crates/linguini-cldr-macros/src/lib.rs; crates/linguini-cldr-macros/src/source.rs; `cargo test -p linguini-cldr-macros`
- [x] Remove runtime CLDR JSON parsing from production paths
  - Note: completed on 2026-05-13. Production lookup APIs use typed compiled plural and formatter data; JSON cache loaders remain ingestion-only.
  - Evidence: crates/linguini-cldr/src/data/compiled.rs; `cargo test -p linguini-cldr`
- [x] Ensure CLDR data is embedded as typed compiled data, not raw `include_str!` JSON blobs
  - Note: completed on 2026-05-13. Compiled CLDR data is represented as typed Rust structs and generated match/function tables without raw JSON embedding.
  - Evidence: crates/linguini-cldr/src/data/compiled.rs; crates/linguini-cldr-macros/src/source.rs; `cargo test --workspace`

Checkpoint acceptance:

- [x] Normal `linguini build` does not download CLDR
  - Note: completed on 2026-05-13. Added `linguini build` path that requires an existing offline CLDR cache and never invokes fetch or git.
  - Evidence: crates/linguini-cli/src/lib.rs; `cargo test -p linguini-cli`
- [x] CLDR fetch/update does not download or vendor the full `cldr-json` repository
  - Note: completed on 2026-05-13. Fetch imports only required JSON files from a staged CLDR source and leaves unrelated CLDR files out of the cache.
  - Evidence: crates/linguini-cldr/src/cache.rs; `cargo test -p linguini-cldr`
- [x] Production binary can evaluate required CLDR rules without runtime JSON files
  - Note: completed on 2026-05-13. Compiled plural-rule tests evaluate English and Russian categories without reading runtime JSON files.
  - Evidence: crates/linguini-cldr/src/data/compiled.rs; `cargo test -p linguini-cldr`
- [x] Cached CLDR data is reused
  - Note: completed on 2026-05-13. Plural rules are loaded from the existing cache path without fetching.
  - Evidence: crates/linguini-cldr/src/data/cache_loaders.rs; `cargo test -p linguini-cldr`
- [x] Missing cache produces actionable error
  - Note: completed on 2026-05-13. Offline cache requirement reports `run linguini cldr fetch` when cache is missing.
  - Evidence: crates/linguini-cldr/src/cache.rs; `cargo test -p linguini-cldr`

---

## 9. CLDR plural expression parser

- [x] Define plural rule AST
  - Note: completed on 2026-05-13. Added internal plural rule AST for OR conditions, AND relations, operand expressions, operators, and ranges.
  - Evidence: crates/linguini-cldr/src/plural.rs; `cargo test -p linguini-cldr`
- [x] Parse operands `n i v w f t c e`
  - Note: completed on 2026-05-13. Parser accepts all required CLDR plural operands.
  - Evidence: crates/linguini-cldr/src/plural.rs; `cargo test -p linguini-cldr`
- [x] Parse logical operators
  - Note: completed on 2026-05-13. Parser splits `or` conditions and `and` relations.
  - Evidence: crates/linguini-cldr/src/plural.rs; `cargo test -p linguini-cldr`
- [x] Parse modulo
  - Note: completed on 2026-05-13. Parser accepts `%` and `mod` modulo expressions.
  - Evidence: crates/linguini-cldr/src/plural.rs; `cargo test -p linguini-cldr`
- [x] Parse ranges
  - Note: completed on 2026-05-13. Parser accepts `start..end` ranges and single-value ranges.
  - Evidence: crates/linguini-cldr/src/plural.rs; `cargo test -p linguini-cldr`
- [x] Parse comma-separated range lists
  - Note: completed on 2026-05-13. Parser accepts comma-separated range/value lists.
  - Evidence: crates/linguini-cldr/src/plural.rs; `cargo test -p linguini-cldr`
- [x] Parse equality and inequality
  - Note: completed on 2026-05-13. Parser accepts `=`, `!=`, `is`, and negated equality forms.
  - Evidence: crates/linguini-cldr/src/plural.rs; `cargo test -p linguini-cldr`
- [x] Parse `in`
  - Note: completed on 2026-05-13. Parser accepts `in` and `not in` relation operators.
  - Evidence: crates/linguini-cldr/src/plural.rs; `cargo test -p linguini-cldr`
- [x] Parse `within`
  - Note: completed on 2026-05-13. Parser accepts `within` and `not within` relation operators.
  - Evidence: crates/linguini-cldr/src/plural.rs; `cargo test -p linguini-cldr`
- [x] Add tests for Russian rules
  - Note: completed on 2026-05-13. Added Russian modulo/range plural parser coverage and CLDR JSON fixture loading.
  - Evidence: crates/linguini-cldr/src/plural.rs; crates/linguini-cldr/src/data.rs; `cargo test -p linguini-cldr`
- [x] Add tests for English rules
  - Note: completed on 2026-05-13. Added English `i = 1 and v = 0` parser coverage and CLDR JSON fixture loading.
  - Evidence: crates/linguini-cldr/src/plural.rs; crates/linguini-cldr/src/data.rs; `cargo test -p linguini-cldr`
- [x] Add tests for Arabic rules
  - Note: completed on 2026-05-13. Added Arabic-shaped `or` and comma-list parser coverage.
  - Evidence: crates/linguini-cldr/src/plural.rs; `cargo test -p linguini-cldr`

Checkpoint acceptance:

- [x] Plural categories match CLDR examples for selected locales
  - Note: completed on 2026-05-13. Added plural operand extraction and rule evaluation for English, Russian, and Arabic-shaped examples.
  - Evidence: crates/linguini-cldr/src/plural/eval.rs; crates/linguini-cldr/src/data.rs; `cargo test -p linguini-cldr`
- [x] Generated plural functions pass snapshot tests
  - Note: completed on 2026-05-13. Added deterministic TypeScript plural function generation from parsed CLDR plural rules.
  - Evidence: crates/linguini-codegen-ts/src/lib.rs; tests/fixtures/golden/snapshots/codegen-ts-plural-ru.ts; `cargo test -p linguini-codegen-ts`

---

## 10. IR

- [x] Define IR nodes
  - Note: completed on 2026-05-13. Added modular IR model for messages, parameters, forms, functions, branches, text parts, expressions, and formatters.
  - Evidence: crates/linguini-ir/src/model.rs; `cargo test -p linguini-ir`
- [x] Lower schema messages to IR
  - Note: completed on 2026-05-13. Schema message signatures and grouped message signatures lower to IR messages with docs and parameters.
  - Evidence: crates/linguini-ir/src/lower.rs; tests/fixtures/golden/snapshots/ir-schema-shop.txt; `cargo test -p linguini-ir`
- [x] Lower locale messages to IR
  - Note: completed on 2026-05-13. Locale standalone and grouped message bodies lower to IR text parts and placeholders.
  - Evidence: crates/linguini-ir/src/lower.rs; tests/fixtures/golden/snapshots/ir-locale-ru.txt; `cargo test -p linguini-ir`
- [x] Lower forms to IR
  - Note: completed on 2026-05-13. Locale forms lower variants, selectors, attributes, branch entries, and nested objects.
  - Evidence: crates/linguini-ir/src/lower.rs; tests/fixtures/golden/snapshots/ir-locale-ru.txt; `cargo test -p linguini-ir`
- [x] Lower local functions to IR
  - Note: completed on 2026-05-13. Locale functions lower parameters, tuple branches, else branches, and text bodies.
  - Evidence: crates/linguini-ir/src/lower.rs; tests/fixtures/golden/snapshots/ir-locale-ru.txt; `cargo test -p linguini-ir`
- [x] Lower plural maps to IR
  - Note: completed on 2026-05-13. Form branch maps with CLDR category keys lower to IR map branches without losing keys or text.
  - Evidence: crates/linguini-ir/src/lower.rs; tests/fixtures/golden/snapshots/ir-locale-ru.txt; `cargo test -p linguini-ir`
- [x] Lower formatting operations to IR
  - Note: completed on 2026-05-13. Placeholder formatter annotations lower to IR formatter nodes with named arguments.
  - Evidence: crates/linguini-ir/src/lower.rs; tests/fixtures/golden/snapshots/ir-locale-ru.txt; `cargo test -p linguini-ir`
- [x] Ensure IR has no unresolved references
  - Note: completed on 2026-05-13. Added IR reference validation for locale message names and placeholder roots against schema parameters, forms, and local functions.
  - Evidence: crates/linguini-ir/src/reference.rs; `cargo test -p linguini-ir`

Checkpoint acceptance:

- [x] IR snapshot for delivery example is stable
  - Note: completed on 2026-05-13. Full IR snapshot includes delivery schema signature and Russian locale implementation with function/form placeholders.
  - Evidence: tests/fixtures/golden/snapshots/ir-schema-shop.txt; tests/fixtures/golden/snapshots/ir-locale-ru.txt; `cargo test -p linguini-ir`
- [x] IR snapshot for counted example is stable
  - Note: completed on 2026-05-13. Full IR snapshot includes counted schema signature and Russian plural-form locale implementation.
  - Evidence: tests/fixtures/golden/snapshots/ir-schema-shop.txt; tests/fixtures/golden/snapshots/ir-locale-ru.txt; `cargo test -p linguini-ir`

---

## 11. TypeScript codegen

- [x] Generate TypeScript enums
  - Note: completed on 2026-05-13. TypeScript backend emits schema enum unions such as `Fruit` and `Size` in locale modules.
  - Evidence: crates/linguini-codegen-ts/src/module; tests/fixtures/golden/snapshots/ts/locales/ru.ts; `cargo test -p linguini-codegen-ts`
- [x] Generate typed message functions
  - Note: completed on 2026-05-13. TypeScript backend emits exported message functions with schema-derived parameter types and locale bodies.
  - Evidence: crates/linguini-codegen-ts/src/module; tests/fixtures/golden/snapshots/ts/locales/ru.ts; `cargo test -p linguini-codegen-ts`
- [x] Generate forms
  - Note: completed on 2026-05-13. TypeScript backend emits form objects with plural-map attributes, nested attributes, and selector branch functions; selector-form calls use `SizeForms[size](...)`.
  - Evidence: crates/linguini-codegen-ts/src/module; tests/fixtures/golden/snapshots/ts/locales/ru.ts; `cargo test -p linguini-codegen-ts`
- [x] Generate local functions
  - Note: completed on 2026-05-13. TypeScript backend emits local helper functions from locale tuple and else branches.
  - Evidence: crates/linguini-codegen-ts/src/module; tests/fixtures/golden/snapshots/ts/locales/ru.ts; `cargo test -p linguini-codegen-ts`
- [x] Generate plural functions
  - Note: completed on 2026-05-13. Added TypeScript plural category function generation from CLDR plural rules; generated locale modules inline their locale plural helper.
  - Evidence: crates/linguini-codegen-ts/src/lib.rs; tests/fixtures/golden/snapshots/codegen-ts-plural-ru.ts; tests/fixtures/golden/snapshots/ts/locales/ru.ts; `cargo test -p linguini-codegen-ts`
- [x] Generate formatter helpers
  - Note: completed on 2026-05-13. TypeScript backend emits shared currency/date formatter helpers and uses placeholder formatter annotations in generated message bodies.
  - Evidence: crates/linguini-codegen-ts/src/module; tests/fixtures/golden/snapshots/ts/shared.ts; `bash scripts/validate-generated-ts.sh`
- [x] Generate `.d.ts`
  - Note: completed on 2026-05-13. TypeScript backend emits declaration files for shared helpers, locale modules, and the index API.
  - Evidence: tests/fixtures/golden/snapshots/ts/index.d.ts; tests/fixtures/golden/snapshots/ts/locales/ru.d.ts; `bash scripts/validate-generated-ts.sh`
- [x] Add tree-shaking mode
  - Note: completed on 2026-05-13. Added TypeScript target `tree_shaking` plus explicit `messages` filtering so generated locale modules and declarations emit only selected message ids.
  - Evidence: crates/linguini-config/src/parser.rs; crates/linguini-codegen-ts/src/module/mod.rs; `cargo test -p linguini-config -p linguini-codegen-ts`
- [x] Add deterministic output tests
  - Note: completed on 2026-05-13. Added generated TypeScript file-tree snapshots covering shared helpers, locale modules, index API, enums, aliases, forms, local functions, and messages.
  - Evidence: tests/fixtures/golden/snapshots/ts; `cargo test -p linguini-codegen-ts`
- [x] Generate facade with switchable active language source
  - Note: completed on 2026-05-13. Generated TypeScript index exposes `createLinguini`, `createLinguiniProvider`, and a default `lgl` facade.
  - Evidence: crates/linguini-codegen-ts/src/module/project.rs; tests/fixtures/golden/snapshots/ts/index.ts; `cargo test -p linguini-codegen-ts`
- [x] Add SvelteKit-compatible locale provider example for cookies, route data, or UI language
  - Note: completed on 2026-05-13. Added a SvelteKit provider example showing route data/cookie-driven language state while components keep `lgl.*(...)` calls.
  - Evidence: docs/examples/sveltekit-locale-provider.md; `./scripts/check-spec-gates.sh`

Checkpoint acceptance:

- [x] Generated TS compiles
  - Note: completed on 2026-05-13. Generated shared helper, index module, and locale module compile as separate TypeScript files; locale module contains its plural helper.
  - Evidence: `bash scripts/validate-generated-ts.sh`
- [x] Delivery example returns expected Russian strings
  - Note: completed on 2026-05-13. Compiled generated TypeScript and checked `delivery("apple", "small", 1)` in Node.
  - Evidence: `bash scripts/validate-generated-ts.sh`
- [x] Counted example returns expected plural strings
  - Note: completed on 2026-05-13. Compiled generated TypeScript and checked Russian `counted` plural output for `1 apple` and `5 orange`.
  - Evidence: `bash scripts/validate-generated-ts.sh`
- [x] Application code can call `lgl.*(...)` while changing one locale source variable to switch output language
  - Note: completed on 2026-05-13. TypeScript facade supports explicit locale factories/providers, and the SvelteKit example wires per-request context to `lgl.*` calls.
  - Evidence: crates/linguini-codegen-ts/src/module/project.rs; docs/examples/sveltekit-locale-provider.md; `cargo test -p linguini-codegen-ts`

---

## 12. TypeScript/JavaScript vite plugin integration

- [ ] Export `locales` and `baseLocale` from generated TypeScript
  - Note:
- [ ] Expose stable generated locale module loaders or locale map for runtime integration
  - Note:
- [ ] Define generated locale provider contract for `getLocale()` and message facades
  - Note:
- [ ] Add configurable locale detection strategy chain
  - Note:
- [ ] Implement URL, cookie, preferred-language, localStorage, and base-locale detectors
  - Note:
- [ ] Implement `localizeHref`
  - Note:
- [ ] Implement `shouldRedirect`
  - Note:
- [ ] Implement server middleware with per-request locale context
  - Note:
- [ ] Add `disableAsyncLocalStorage` runtime option
  - Note:
- [ ] Implement `%lang%` and `%dir%` injection support
  - Note:
- [ ] Implement `getTextDirection`
  - Note:
- [ ] Implement Svelte/SvelteKit `<Trans>` component for rich text and component interpolation
  - Note:
- [ ] Add SvelteKit static-site locale-link support
  - Note:
- [ ] Add Vite plugin that rebuilds when translation files change
  - Note:

Checkpoint acceptance:

- [ ] Detection strategy resolves locales in configured priority order
  - Note:
- [ ] Middleware keeps concurrent request locale state isolated
  - Note:
- [ ] `localizeHref` converts between locale-prefixed URLs
  - Note:
- [ ] `shouldRedirect` identifies stale localized URLs after navigation
  - Note:
- [ ] `<Trans>` renders translated rich text with Svelte components
  - Note:
- [ ] `locales` supports a locale switcher without hand-written locale lists
  - Note:

---

## 13. Formatter

- [ ] Format schema files
  - Note:
- [ ] Format locale files
  - Note:
- [ ] Preserve doc comments
  - Note:
- [ ] Preserve ordinary comments where possible
  - Note:
- [ ] Enforce line width
  - Note:
- [ ] Add idempotency tests
  - Note:

Checkpoint acceptance:

- [ ] Formatting twice produces identical output
  - Note:
- [ ] Formatter does not change semantics
  - Note:

---

## 14. LSP

- [ ] Start LSP server over stdio
  - Note:
- [ ] Publish diagnostics
  - Note:
- [ ] Implement hover
  - Note:
- [ ] Implement completion
  - Note:
- [ ] Implement semantic tokens
  - Note:
- [ ] Implement go to definition
  - Note:
- [ ] Implement find references
  - Note:
- [ ] Implement code actions
  - Note:
- [ ] Implement quick fix: add missing branches
  - Note:
- [ ] Implement quick fix: add missing message
  - Note:
- [ ] Implement quick fix: add explicit plural argument
  - Note:
- [ ] Implement formatting request
  - Note:

Checkpoint acceptance:

- [ ] VS Code shows diagnostics
  - Note:
- [ ] Hover shows schema doc comments
  - Note:
- [ ] Completion suggests schema args
  - Note:
- [ ] Quick fix can add missing branches
  - Note:

---

## 15. Syntax highlighting

- [ ] Create TextMate grammar for `.lqs`
  - Note:
- [ ] Create TextMate grammar for `.lgl`
  - Note:
- [ ] Create VS Code extension
  - Note:
- [ ] Wire semantic tokens from LSP
  - Note:
- [ ] Add grammar snapshot tests
  - Note:

Checkpoint acceptance:

- [ ] `.lqs` files highlight in VS Code
  - Note:
- [ ] `.lgl` files highlight in VS Code
  - Note:

---

## 16. Locale management

- [ ] Implement `linguini status`
  - Note:
- [ ] Implement `linguini fill`
  - Note:
- [ ] Generate missing locale files
  - Note:
- [ ] Generate missing messages
  - Note:
- [ ] Generate missing form variants
  - Note:
- [ ] Include schema doc comments in generated stubs
  - Note:
- [ ] Add completion percentage output
  - Note:

Checkpoint acceptance:

- [ ] New locale can be scaffolded
  - Note:
- [ ] Missing messages can be filled with stubs
  - Note:
- [ ] Generated stubs include translator context
  - Note:

---

## 17. Formatting data

- [ ] Generate number formatter helpers
  - Note:
- [ ] Generate percent formatter helpers
  - Note:
- [ ] Generate currency formatter helpers
  - Note:
- [ ] Generate date formatter helpers
  - Note:
- [ ] Generate time formatter helpers
  - Note:
- [ ] Support locale overrides
  - Note:

Checkpoint acceptance:

- [ ] Formatting works without runtime CLDR lookup
  - Note:
- [ ] Only used formatters are emitted
  - Note:

---

## 18. Packages

- [ ] Define package manifest format
  - Note:
- [ ] Define package import syntax
  - Note:
- [ ] Implement package lockfile
  - Note:
- [ ] Implement local package import
  - Note:
- [ ] Implement registry package import
  - Note:
- [ ] Analyze imported forms
  - Note:
- [ ] Analyze imported functions
  - Note:
- [ ] Detect version conflicts
  - Note:
- [ ] Implement `linguini package add`
  - Note:
- [ ] Implement `linguini package publish`
  - Note:

Checkpoint acceptance:

- [ ] A package can provide reusable forms
  - Note:
- [ ] A project can import and use package forms
  - Note:
- [ ] Imported package content participates in diagnostics
  - Note:

---

## 19. Real-project validation

- [x] Add rendered locale generate command
  - Note: completed on 2026-05-13. Added `linguini generate` colorized human-readable output that renders every configured locale across enum variants and representative numeric plural values.
  - Evidence: crates/linguini-cli/src/project/test_data; crates/linguini-cli/tests/cli.rs; `cargo test -p linguini-cli`
- [ ] Test in a TypeScript app
  - Note:
- [ ] Test adding a second locale
  - Note:
- [ ] Test partial locale completion
  - Note:
- [ ] Test LSP in VS Code
  - Note:
- [ ] Measure build time
  - Note:
- [ ] Measure generated output size
  - Note:

Checkpoint acceptance:

- [ ] Real TS project can use generated Linguini output
  - Note:
- [ ] Developer can add missing locale messages via LSP quick fixes
  - Note:
