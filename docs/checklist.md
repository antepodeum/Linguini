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
- [ ] Add unit test structure for every core crate
  - Note:
  - Evidence:
- [x] Add golden fixture directory for `.lqs` and `.lgl` projects
  - Note: completed on 2026-05-12. Expanded schema and Russian locale golden fixtures to cover enums, formatter annotations, docs, forms, selector maps, plural-shaped maps, nested form attributes, helper functions, messages, placeholders, and grouped messages.
  - Evidence: tests/fixtures/golden/schema/shop.lqs; tests/fixtures/golden/locale/ru.lgl; `cargo test -p linguini-syntax`
- [ ] Add `insta` snapshot review workflow
  - Note:
  - Evidence:
- [ ] Add CLI integration test harness with `assert_cmd` and `tempfile`
  - Note:
  - Evidence:
- [ ] Add generated TypeScript validation fixture
  - Note:
  - Evidence:
- [ ] Add generated JavaScript validation fixture
  - Note:
  - Evidence:
- [ ] Add generated Rust validation fixture
  - Note:
  - Evidence:
- [ ] Add regression-test rule to contribution docs
  - Note:
  - Evidence:
- [ ] Add coverage measurement command for core crates
  - Note:
  - Evidence:
- [ ] Add CI job that runs unit, snapshot, CLI, and generated-output tests
  - Note:
  - Evidence:

Checkpoint acceptance:

- [ ] No implementation task can be marked complete without test evidence
  - Note:
  - Evidence:
- [ ] CI runs the full required test suite
  - Note:
  - Evidence:
- [ ] Coverage report exists for core crates
  - Note:
  - Evidence:

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
- [ ] Validate form enum coverage
  - Note:
- [ ] Validate selector enum coverage
  - Note:
- [ ] Validate `other` branch requirement
  - Note:
- [ ] Validate placeholder variables
  - Note:
- [ ] Validate form property access
  - Note:
- [ ] Validate function calls
  - Note:
- [ ] Validate function arity
  - Note:
- [ ] Validate tuple patterns
  - Note:
- [ ] Detect reference cycles
  - Note:
- [ ] Resolve implicit plural arguments
  - Note:
- [ ] Reject ambiguous implicit plural arguments
  - Note:

Checkpoint acceptance:

- [ ] `delivery = {delivered(fruit.gender)} {size(fruit.gender)} {fruit.nom}` passes
  - Note:
- [ ] Missing enum variant is reported
  - Note:
- [ ] Unknown form property is reported
  - Note:
- [ ] Ambiguous plural access is reported
  - Note:

---

## 8. CLDR ingestion

- [ ] Implement CLDR cache directory
  - Note:
- [ ] Implement `linguini cldr fetch`
  - Note:
- [ ] Implement `linguini cldr status`
  - Note:
- [ ] Load plural rules from CLDR JSON
  - Note:
- [ ] Load number formatting data
  - Note:
- [ ] Load date formatting data
  - Note:
- [ ] Load currency formatting data
  - Note:
- [ ] Add cache integrity checks
  - Note:
- [ ] Add offline build mode
  - Note:

Checkpoint acceptance:

- [ ] Normal `linguini build` does not download CLDR
  - Note:
- [ ] Cached CLDR data is reused
  - Note:
- [ ] Missing cache produces actionable error
  - Note:

---

## 9. CLDR plural expression parser

- [ ] Define plural rule AST
  - Note:
- [ ] Parse operands `n i v w f t c e`
  - Note:
- [ ] Parse logical operators
  - Note:
- [ ] Parse modulo
  - Note:
- [ ] Parse ranges
  - Note:
- [ ] Parse comma-separated range lists
  - Note:
- [ ] Parse equality and inequality
  - Note:
- [ ] Parse `in`
  - Note:
- [ ] Parse `within`
  - Note:
- [ ] Add tests for Russian rules
  - Note:
- [ ] Add tests for English rules
  - Note:
- [ ] Add tests for Arabic rules
  - Note:

Checkpoint acceptance:

- [ ] Plural categories match CLDR examples for selected locales
  - Note:
- [ ] Generated plural functions pass snapshot tests
  - Note:

---

## 10. IR

- [ ] Define IR nodes
  - Note:
- [ ] Lower schema messages to IR
  - Note:
- [ ] Lower locale messages to IR
  - Note:
- [ ] Lower forms to IR
  - Note:
- [ ] Lower local functions to IR
  - Note:
- [ ] Lower plural maps to IR
  - Note:
- [ ] Lower formatting operations to IR
  - Note:
- [ ] Ensure IR has no unresolved references
  - Note:

Checkpoint acceptance:

- [ ] IR snapshot for delivery example is stable
  - Note:
- [ ] IR snapshot for counted example is stable
  - Note:

---

## 11. TypeScript codegen

- [ ] Generate TypeScript enums
  - Note:
- [ ] Generate typed message functions
  - Note:
- [ ] Generate forms
  - Note:
- [ ] Generate local functions
  - Note:
- [ ] Generate plural functions
  - Note:
- [ ] Generate formatter helpers
  - Note:
- [ ] Generate `.d.ts`
  - Note:
- [ ] Add tree-shaking mode
  - Note:
- [ ] Add deterministic output tests
  - Note:

Checkpoint acceptance:

- [ ] Generated TS compiles
  - Note:
- [ ] Delivery example returns expected Russian strings
  - Note:
- [ ] Counted example returns expected plural strings
  - Note:

---

## 12. JavaScript codegen

- [ ] Generate ESM output
  - Note:
- [ ] Generate optional CommonJS output
  - Note:
- [ ] Generate JSDoc types
  - Note:
- [ ] Reuse TS backend IR lowering
  - Note:
- [ ] Add deterministic output tests
  - Note:

Checkpoint acceptance:

- [ ] Generated JS runs in Node
  - Note:
- [ ] Output has no TypeScript dependency
  - Note:

---

## 13. Rust codegen

- [ ] Generate Rust module tree
  - Note:
- [ ] Generate Rust enums
  - Note:
- [ ] Generate typed message functions
  - Note:
- [ ] Generate forms
  - Note:
- [ ] Generate local functions
  - Note:
- [ ] Generate plural functions
  - Note:
- [ ] Avoid allocation for static messages
  - Note:
- [ ] Add deterministic output tests
  - Note:

Checkpoint acceptance:

- [ ] Generated Rust crate compiles
  - Note:
- [ ] Delivery example returns expected Russian strings
  - Note:
- [ ] Static messages return `&'static str` where possible
  - Note:

---

## 14. Formatter

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

## 15. LSP

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

## 16. Syntax highlighting

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

## 17. Locale management

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

## 18. Formatting data

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

## 19. Packages

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

## 20. Real-project validation

- [ ] Test in a TypeScript app
  - Note:
- [ ] Test in a JavaScript app
  - Note:
- [ ] Test in a Rust app
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
- [ ] Real Rust project can use generated Linguini output
  - Note:
- [ ] Developer can add missing locale messages via LSP quick fixes
  - Note:
