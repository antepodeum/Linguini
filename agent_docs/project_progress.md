# Active deployment: Bundler-native nested message API

## Goal

Ship checklist `BUNDLE-A1` through `BUNDLE-A10`, completing direct blockers first and preserving the public nested value/function API.

## Constraints

- Heavy route; main agent owns integration, Git state, and this status file.
- Preserve existing uncommitted IR/group work and commit each coherent verified step.
- One shared single-message compiler must serve virtual and physical module backends.
- Vite owns module graph, chunking, preload, loading, and HMR propagation.
- No public API may require calling parameterless message leaves.

## Ordered work packages

1. `BUNDLE-B0` — Finish explicit recursive group metadata across IR lowering, qualification, merge/fallback, validation, and tests. This blocks trustworthy recursive namespace docs/types.
2. `BUNDLE-B1` — Add a single-message compiler API with deterministic transitive semantic dependency closure and source-mapped ESM output.
3. `BUNDLE-B2` — Generate complete recursive `l` declarations and implement static Svelte/TypeScript leaf transformation into exact per-message imports.
4. `BUNDLE-B3` — Serve per-message virtual modules, add a shared physical-module backend, reject dynamic lookup in strict mode, and expose an explicit bounded bundle escape hatch.
5. `BUNDLE-B4` — Track source-to-message dependencies for selective invalidation and express optional locale loading as bundler-visible dynamic ESM boundaries.
6. `BUNDLE-B5` — Reconcile public docs/checklist, run focused and broad regression gates, and record remaining unrelated remediation separately.

## Acceptance and verification

- Recursive namespace declarations cover empty/documented/nested groups without inferring groups from dotted project paths.
- Static `l.shop.main.title` and `l.shop.cart.items(count)` become imports of only their exact message modules; user syntax keeps parameterless leaves as values.
- Each compiled leaf contains only its transitive forms/functions/variables/types/formatter/CLDR dependencies.
- Dynamic lookup fails with a precise strict-mode diagnostic; explicit escape hatch has bounded declared scope.
- Virtual and physical outputs are byte/semantic-equivalent where paths differ only by backend.
- A changed source invalidates only dependent message virtual modules.
- Real Vite integration proves route/shared chunk ownership and optional locales remain dynamic imports.
- Focused Rust and Node tests pass after each package; final relevant workspace/plugin regressions pass; `git diff --check` stays clean.

## Dependencies and ownership

- `BUNDLE-B0` precedes recursive API/docs work.
- `BUNDLE-B1` precedes plugin virtual/physical backends.
- `BUNDLE-B2` and `BUNDLE-B3` integrate through stable virtual IDs and single-message compiler contract.
- `BUNDLE-B4` depends on compiler dependency metadata plus plugin module registry.
- Main agent reviews central interfaces and commits. Executor owns each bounded implementation package; tester independently owns deterministic test gates after executor handoff; doc-writer handles verified durable public behavior only.

## Current state

- `BUNDLE-B0` complete: explicit recursive group metadata is preserved through IR lowering,
  qualification, validation, merge/fallback, and codegen projections (`4e648f4`).
- `BUNDLE-B1` complete: the public single-message compiler uses deterministic transitive semantic
  dependency closure and emits source-mapped TypeScript ESM (`f2faa80`, `6caec55`).
- `BUNDLE-B2` is active: recursive `LinguiniMessages`, exact application references, physical
  message artifacts, reactive locale state, and manifest-v2 application bridging are committed
  (`cae571a`, `b7758fd`, `591c532`, `bed72e0`, `18792cf`). Safe import-removal metadata and a
  lightweight Svelte side-effect entry are the current parallel blockers for the Vite transform.
- `BUNDLE-B3` is partial: the physical backend is complete; virtual module serving, strict dynamic
  diagnostics, and the bounded escape hatch remain.
- `BUNDLE-B4` and `BUNDLE-B5` remain pending.
- Next action: verify and commit the transform-safety/runtime-side-effect blockers, then implement
  exact Vite virtual imports and parameterless value rewrites before selective HMR and locale
  dynamic boundaries.
