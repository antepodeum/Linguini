# Active deployment: Shared locale transition plan

## Goal

Lower the validated web `LocaleSwitchPlan` once and make generated browser and SvelteKit controls
consume that shared contract without regressing locale initialization, preload-before-switch,
cookie/path/local-storage behavior, SSR, or bundler-native locale chunks.

## Constraints

- Heavy route; the main agent owns integration, Git state, checklist, and this status file.
- Treat the validated config model as the only source of transition capabilities.
- Emit one deterministic transport plan instead of duplicating browser/server policy decisions.
- Preserve dynamic-locale preparation before browser-visible locale changes.
- Keep server-only, browser-only, and pathless configurations executable and explicit.

## Ordered work packages

1. `WEB-B0` — Add an explicit generated `LocaleSwitchPlan` model and lower validated config into it.
2. `WEB-B1` — Make browser and SvelteKit controls execute the same generated plan for navigation,
   cookie persistence, local-storage persistence, and unsupported transport states.
3. `WEB-B2` — Add codegen snapshots plus browser/server runtime matrices covering path, cookie,
   local-storage, and accept-language source combinations.
4. `WEB-B3` — Verify focused Rust/runtime/plugin/site gates, then reconcile WEB-A14 and P2-9.

## Acceptance and verification

- Generated TypeScript contains one literal transition plan derived from validated config.
- Browser and server controls do not independently infer allowed write transports.
- `setLocale` writes/navigates only through enabled plan transports and retains locale preloading.
- Browser startup and SvelteKit SSR resolve compatible initial locale state.
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
- `WEB-B0` through `WEB-B3` are complete in `5e81c9a` and `193c83f`. Validated config now lowers one
  explicit locale-switch plan into generated TypeScript; browser controls gate path navigation,
  cookie writes, and local-storage writes through it, while both SvelteKit adapters use the same
  plan for server cookie persistence. Public runtime fallback plans derive from custom source order.
- Codegen/runtime matrices cover default, explicit, path-only, cookie-only, local-storage-only, and
  accept-language-only plans. Independent verification found and closed one fallback defect; the
  final full root profile passed all 11 tasks, including zero Svelte errors and the production graph.
- Next: resume the remaining remediation checklist from the first dependency-ready unfinished
  work package after the shared locale transition plan.
