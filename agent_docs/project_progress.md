# Completed deployment: Closed generated web features

## Goal

Lower validated web policy into one closed `WebFeatures` model and generate only selected capability
modules without regressing locale resolution, shared transition plans, plain Svelte, SvelteKit SSR,
or bundler-native locale chunks.

## Constraints

- Heavy route; the main agent owns integration, Git state, checklist, and this status file.
- Treat validated config as the only source of generated feature selection.
- Preserve the public standalone web runtime compatibility surface.
- Keep plain-Svelte and SvelteKit generation distinct and executable.
- Do not include the legacy synchronous locale index redesign tracked by #236.

## Ordered work packages

1. [x] `WEB-C0` — Define a closed generated `WebFeatures` model and lower validated policy into it.
2. [x] `WEB-C1` — Split source and browser/server capabilities into selected generated modules instead
   of one generic source-strategy loop.
3. [x] `WEB-C2` — Verify path, cookie, local-storage, accept-language, and mixed feature matrices across
   plain Svelte, SvelteKit, and public-runtime compatibility paths.
4. [x] `WEB-C3` — Run strict focused and full repository gates, then reconcile #215, WEB-A12/A13, P2-8.

## Acceptance and verification

- Generated output contains only selected source/capability modules and no generic strategy loop.
- Closed features retain ordered resolution and the shared locale-switch plan exactly.
- Browser navigation/persistence and server cookie behavior remain capability-gated.
- Public runtime custom-source fallback remains compatible.
- Focused codegen/runtime tests and root site/full profiles pass; `git diff --check` remains clean.

## Completed predecessor

- Bundler deployment `BUNDLE-B0` through `BUNDLE-B5` is complete. Checklist `BUNDLE-A1` through
  `BUNDLE-A10` and `LOCAL-A1` through `LOCAL-A2` are verified and checked.
- The production site uses v1 manifest dynamic locale loading. Its graph gate proves 585 physical
  locale modules remain outside the initial static client graph while SSR stays synchronous.
- Public docs now describe the nested value/callable API, exact message imports, strict bounded
  dynamic access, dynamic locale preparation, and root verification profiles (`e8be06d`).
- Final predecessor regression gate: `pnpm test:full` passed all 11 registered tasks.

## Resumed successor

- `CALL-B0` and `CALL-B1` are complete. Runtime remediation no longer blocks generated-call work;
  checklist `CALL-A1` through `CALL-A6` and `P2-7` are verified and checked.

## Current state

- `HELPER-B0` identified self-contained physical closure emission as the source of duplicated
  formatter/date/plural helper bodies and reusable semantic dependencies; remediation is complete.
- `HELPER-B1` through `HELPER-B3` were completed in `d68e432`. Standalone compilation remains
  self-contained; project and physical output share one runtime per locale. The current active v1
  manifest carries exact runtime source IDs; Vite invalidates runtime descriptor/source changes.
- The real site gate passes all five tasks. Its 585 physical modules contain zero helper bodies,
  54 import exact runtime names, and nine runtime modules cover all nine effective locales.
- `HELPER-B4` is complete in `0f8cfc3`. Forty-seven exact semantic modules cover the real site;
  27 physical leaves import them, no physical leaf contains an inline semantic declaration, and
  physical source fell from 113,413 to 95,374 bytes. The unreleased active manifest is v1 only.
- `CALL-B1` is complete in `76059bc`. Positional and named-object calls share one generated body
  and boundary normalizer; parameterless leaves remain values. Rust codegen, Vite, generated
  declarations, negative TypeScript fixtures, runtime parity, and all five real-site gates pass.
- Legacy project output now emits variables, forms, functions, and locale enums once in each
  locale's reserved `_globals.ts` module (`e0a82a3`). Root and namespace modules import bindings
  without copying declarations. All nine site locales generate this boundary; codegen, CLI, Vite,
  runtime, type/Svelte, production build, and graph gates pass. Checklist #202, #204, and
  `BUNDLE-A13` are complete.
- The strict codegen Clippy blocker introduced by the physical-message import surface was removed
  in `bd9c67e` by grouping import inputs without changing generated output.
- Direct project codegen now rejects every visible schema message without an effective
  fallback-composed locale implementation (`8b2bf7f`). Tree-shaken selection is respected and
  sparse regional locales remain valid when their base supplies the message. Codegen (103 unit +
  3 web safety), CLI (78 unit + 24 integration), and strict codegen Clippy pass; checklist #200 is
  complete.
- Browser locale negotiation now uses guarded `navigator.languages` / `navigator.language` when
  request headers are unavailable (`6542ce2`), preventing server-selected locale state from
  becoming uninitialized during browser startup.
- Bundler locale loading now coalesces exact message facades behind one virtual entry per effective
  locale (`cd630fa`). The site routes 58 message facades through nine dynamic locale entries, emits
  zero physical message client entries and 21 total client JavaScript files instead of roughly 549,
  keeps SSR static, and invalidates locale aggregators through exact HMR deltas.
- Bundler parameterless leaves now export raw values while standalone compilation preserves its
  callable compatibility contract. The full 11-task root profile passed; independent read-only
  verification found no defects in runtime initialization, chunk topology, SSR, or leaf shape.
- Dynamic client facades now share one virtual locale registry (`7ce324f`). The real site emits
  nine locale imports instead of 522 repeated message-by-locale imports; its route node fell from
  154,807 to 72,546 bytes (28,971 to 22,917 gzip). One active locale chunk loads initially,
  parameterless leaves stay values, and no per-facade preparation/crash branch remains.
- Dynamic client registries and locale entries are now application-scoped (`711b5a0`). The dedicated
  `/chunk-lab` production route references exactly three messages, owns nine lazy locale entries,
  and every emitted locale payload contains only those three messages with no static imports.
  Ref-counted Svelte lifecycle leases unregister inactive route loaders while SSR remains eager.
  Clean English, Russian, and French preview profiles rendered the correct active locale and loaded
  its matching scoped chunk; the full repository profile passed all 11 tasks. `BUNDLE-A15` was
  refined to the scoped design and `BUNDLE-A16` is complete.
- Scoped locale payloads now use stable numeric indices instead of canonical string keys
  (`0a57506`). The production graph imports the emitted lab chunks and proves exact value-array
  shape while rejecting every leaked `main.*` path across both route scopes. The lab English chunk
  fell from 182 to 120 bytes; the main route node fell from 67.83 to 65.27 kB and from 21.36 to
  20.88 kB gzip. Sparse fallback, callable messages, eager SSR, and legacy unscoped objects remain
  covered. The full 11-task profile passes and `BUNDLE-A17` is complete.
- `WEB-B0` through `WEB-B3` are complete in `5e81c9a` and `193c83f`. Validated config now lowers one
  explicit locale-switch plan into generated TypeScript; browser controls gate path navigation,
  cookie writes, and local-storage writes through it, while both SvelteKit adapters use the same
  plan for server cookie persistence. Public runtime fallback plans derive from custom source order.
- Codegen/runtime matrices cover default, explicit, path-only, cookie-only, local-storage-only, and
  accept-language-only plans. Independent verification found and closed one fallback defect; the
  final full root profile passed all 11 tasks, including zero Svelte errors and the production graph.
- `WEB-C0` is complete and the locale-source slice of `WEB-C1`/`WEB-C2` landed in `26675a5`.
  Validated config lowers one closed `WebFeatures`; generated projects emit only selected path,
  cookie, local-storage, and accept-language resolver modules in validated order. Empty source sets
  resolve to the base locale, and pathless controls contain no navigation imports or URL writes.
  Standalone runtime dispatch remains compatible. The full 11-task profile and 32 Bun tests pass.
- `WEB-C1` through `WEB-C3` are complete in `719c580`, `13f9c13`, `372d4ea`, `d373c75`, and
  `687e943`. Generated projects select physical link-transform, runtime-link, server-cookie,
  route-matcher, and switch-route modules; safe static anchors transform at compile time.
- The optional SvelteKit switch route validates locale and same-origin returns, reuses the shared
  transition plan and route matcher, and rejects configurations without a server transport.
  Local-storage-only SvelteKit SSR policy now fails with a focused diagnostic.
- Checklist #215, #220, WEB-A13, WEB-A15 through WEB-A19, P2-8, and DOC-W1 through DOC-W8 are
  closed. WEB-A20 and DOC-W9 remain separate legacy/runtime integration work outside this plan.
- Final verification: `pnpm test:full` passed all 11 gates, including 110 codegen tests, four web
  safety tests, 79 CLI unit tests, 24 CLI integration tests, strict Clippy, zero Svelte diagnostics,
  the production build, and graph assertions.
