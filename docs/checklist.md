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

- [ ] Create Rust workspace
  - Note:
- [ ] Add crates listed in the technical specification
  - Note:
- [ ] Add CI
  - Note:
- [ ] Add file-size check: warn at 400 LOC, fail at 500 LOC
  - Note:
- [ ] Add generated/vendor exclusions for file-size check
  - Note:
- [ ] Add formatting/linting pipeline
  - Note:
- [ ] Add test fixture directory
  - Note:
- [ ] Add snapshot test setup
  - Note:

Checkpoint acceptance:

- [ ] `cargo test` runs successfully
  - Note:
- [ ] CI fails on source files above 500 LOC
  - Note:
- [ ] Workspace has no large catch-all implementation files
  - Note:

---

## 0.1 Testing policy and gates

- [ ] Add mandatory test policy to repository docs
  - Note:
  - Evidence:
- [ ] Add unit test structure for every core crate
  - Note:
  - Evidence:
- [ ] Add golden fixture directory for `.lqs` and `.lgl` projects
  - Note:
  - Evidence:
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

- [ ] Implement `linguini.toml` parser
  - Note:
- [ ] Validate required config fields
  - Note:
- [ ] Implement schema path discovery
  - Note:
- [ ] Implement locale path discovery
  - Note:
- [ ] Parse locale file names as BCP 47-like tags
  - Note:
- [ ] Derive schema namespaces from paths
  - Note:
- [ ] Derive locale namespaces from paths
  - Note:
- [ ] Implement top-down scope path collection
  - Note:
- [ ] Implement `linguini init`
  - Note:

Checkpoint acceptance:

- [ ] `linguini init` creates a valid project
  - Note:
- [ ] `linguini check` lists discovered schema files
  - Note:
- [ ] `linguini check` lists discovered locale files
  - Note:
- [ ] `locale/shop/delivery/ru.lgl` sees parent scope files
  - Note:

---

## 2. Lexer

- [ ] Define token model with spans
  - Note:
- [ ] Implement code mode
  - Note:
- [ ] Implement raw text mode after `=`
  - Note:
- [ ] Implement raw text mode after `=>`
  - Note:
- [ ] Implement multiline text mode
  - Note:
- [ ] Implement placeholder mode
  - Note:
- [ ] Implement ordinary comments
  - Note:
- [ ] Implement doc comments
  - Note:
- [ ] Add lexer error recovery
  - Note:
- [ ] Add lexer snapshot tests
  - Note:

Checkpoint acceptance:

- [ ] Lexer handles `.lqs` examples
  - Note:
- [ ] Lexer handles `.lgl` examples
  - Note:
- [ ] Lexer reports spans correctly
  - Note:
- [ ] Lexer supports Cyrillic raw text
  - Note:

---

## 3. Parser

- [ ] Implement schema parser
  - Note:
- [ ] Implement locale parser
  - Note:
- [ ] Parse enums
  - Note:
- [ ] Parse custom scalar types
  - Note:
- [ ] Parse message signatures
  - Note:
- [ ] Parse grouped messages
  - Note:
- [ ] Parse forms
  - Note:
- [ ] Parse selector maps
  - Note:
- [ ] Parse plural-map-shaped branches
  - Note:
- [ ] Parse local functions
  - Note:
- [ ] Parse placeholders
  - Note:
- [ ] Parse formatter annotations
  - Note:
- [ ] Preserve source spans for all AST nodes
  - Note:
- [ ] Add parser recovery
  - Note:

Checkpoint acceptance:

- [ ] All valid fixtures parse
  - Note:
- [ ] Invalid fixtures produce diagnostics
  - Note:
- [ ] Parser does not require semantic information
  - Note:

---

## 4. Diagnostics

- [ ] Add Ariadne renderer
  - Note:
- [ ] Add diagnostic severity levels
  - Note:
- [ ] Add related spans
  - Note:
- [ ] Add quick-fix hint model
  - Note:
- [ ] Add CLI diagnostic output tests
  - Note:

Checkpoint acceptance:

- [ ] Syntax errors show highlighted spans
  - Note:
- [ ] Analyzer errors show related declarations
  - Note:
- [ ] Diagnostics are stable in snapshots
  - Note:

---

## 5. Schema symbol table

- [ ] Register schema enums
  - Note:
- [ ] Register enum variants
  - Note:
- [ ] Register custom scalar types
  - Note:
- [ ] Register public messages
  - Note:
- [ ] Register grouped messages
  - Note:
- [ ] Store schema doc comments
  - Note:
- [ ] Detect duplicate declarations
  - Note:
- [ ] Resolve type references
  - Note:

Checkpoint acceptance:

- [ ] Unknown schema type is reported
  - Note:
- [ ] Duplicate enum is reported
  - Note:
- [ ] Doc comments are available to analyzer and LSP
  - Note:

---

## 6. Locale scope model

- [ ] Load root locale scope file
  - Note:
- [ ] Load parent directory scope files
  - Note:
- [ ] Merge scope declarations in order
  - Note:
- [ ] Implement explicit `override`
  - Note:
- [ ] Register local enums
  - Note:
- [ ] Register local functions
  - Note:
- [ ] Register forms
  - Note:
- [ ] Register message implementations
  - Note:
- [ ] Detect duplicate declarations
  - Note:
- [ ] Detect invalid shadowing
  - Note:

Checkpoint acceptance:

- [ ] Child locale files can use parent local enums
  - Note:
- [ ] Child locale files can use parent functions
  - Note:
- [ ] Invalid shadowing is reported
  - Note:

---

## 7. Analyzer

- [ ] Match locale messages to schema messages
  - Note:
- [ ] Validate missing public messages
  - Note:
- [ ] Validate unknown public messages
  - Note:
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
