# Active deployment: Bundler-native message APIs

## Goal

Keep physical message output exact and deduplicated, then resume the named-object call API without
regressing positional calls, tree-shaking, locale splitting, source maps, or HMR.

## Constraints

- Heavy route; the main agent owns integration, Git state, checklist, and this status file.
- Preserve the public standalone single-message compiler as a self-contained backend.
- Make generated bundler modules import shared, locale-granular ESM runtime dependencies.
- Keep imports exact enough for Vite/Rollup tree-shaking and dynamic locale boundaries.
- Record shared runtime files in the manifest so source-driven HMR cannot serve stale helpers.

## Ordered work packages

1. `HELPER-B0` — Map duplicated formatter/plural and semantic closure emission across standalone,
   project, physical-module, manifest, and HMR boundaries.
2. `HELPER-B1` — Generate one shared formatter/plural runtime per effective locale and make project
   plus physical message modules import only required names.
3. `HELPER-B2` — Record runtime modules in the bundler manifest and invalidate them through exact
   source-driven Vite HMR transitions.
4. `HELPER-B3` — Verify source maps, atomic replacement, direct codegen, CLI, Vite, real-site
   development, production chunks, and a generated-source size regression.
5. `HELPER-B4` — Move reusable variables, forms, and local functions behind shared semantic ESM
   boundaries rather than copying their transitive implementations into each message.

## Acceptance and verification

- Standalone single-message compilation remains deterministic and self-contained.
- No physical bundler message contains formatter, date-coercion, numeric-parser, or plural helper
  implementations; it imports only the names its dependency closure requires.
- Each effective locale owns one shared helper runtime consumed by both project and physical output.
- Locale source changes invalidate changed runtime modules and their consuming message graph.
- Focused Rust/codegen/CLI/plugin tests, the real development server, root full/site profiles, and
  generated/built graph size assertions pass; `git diff --check` remains clean.

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
- Next: resume the remaining remediation checklist from the first dependency-ready unfinished
  work package after the bundler-native message API sequence.
