# Repository State

This file tracks the current repository surface and the remaining desloppifying work.

## Current State

- Rust workspace with compiler, analyzer, formatter, CLDR, codegen, CLI, and LSP crates.
- TypeScript codegen currently targets plain TypeScript plus Svelte 5 and SvelteKit runtimes.
- Website lives in `site/` and is prerendered with locale-aware SvelteKit routing.
- VS Code extension lives in `editors/vscode/`.
- Vite plugin lives in `plugins/vite/`.
- CLDR plural and formatting data is generated at Rust compile time from pinned CLDR JSON.

## Desloppifying Progress

- [ ] `crates/linguini-analyzer`
- [ ] `crates/linguini-cldr`
- [ ] `crates/linguini-cldr-macros`
- [ ] `crates/linguini-cli`
- [ ] `crates/linguini-codegen-ts`
- [ ] `crates/linguini-config`
- [ ] `crates/linguini-core`
- [ ] `crates/linguini-format`
- [ ] `crates/linguini-ir`
- [ ] `crates/linguini-locale`
- [ ] `crates/linguini-lsp`
- [ ] `crates/linguini-schema`
- [ ] `crates/linguini-syntax`
- [ ] `crates/linguini-test-support`
- [ ] `plugins/vite`
- [ ] `editors/vscode`
- [ ] `site`
