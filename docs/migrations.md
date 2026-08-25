# Migration notes

This file records required application changes when a public Linguini contract changes. The CI
contract gate requires source changes to public configuration, CLI, generated API, Vite, LSP, or
editor boundaries to update this file, public documentation, and executable tests together.

## 0.1.0-alpha.4 remediation baseline

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
- Generated documentation now preserves consecutive lines and blank paragraphs in one safe JSDoc
  block across runtime and declaration output. Typed backend tags are generated separately from
  canonical source prose.
- Generated output is ESM-only. Remove `targets.ts.module` and CommonJS output assumptions.
- SvelteKit web policy uses nested `[web.*]` tables. Replace legacy flat routing, source, cookie,
  local-storage, link, exclusion, and switch-route fields with the mappings in
  [Web and SvelteKit](./web-sveltekit.md).
- Parameterless generated messages are string values. Parameterized messages accept positional
  arguments or one named object with every declared property.
- Bundler-mode `l.*` access must be statically resolvable unless each finite dynamic path is listed
  under `[targets.ts.bundler.dynamic]` with `mode = "bundle"`.
- Dynamic locale loading prepares a locale before publishing it. Direct framework-agnostic callers
  must await `prepareLinguini(locale)` before synchronous non-base access.
