# Active deployment: Generated positional and named-object calls

## Goal

Complete checklist `CALL-A1` through `CALL-A5` without regressing the finished
bundler-native nested message API or changing Linguini source syntax.

## Constraints

- Heavy route; the main agent owns integration, Git state, checklist, and this status file.
- Preserve positional calls and the public nested value/function API.
- Normalize positional and named-object inputs once at the generated function boundary.
- Render runtime functions, declarations, and JSDoc from one signature model.
- Keep exact per-message compilation and Vite transforms backend-neutral.

## Ordered work packages

1. `CALL-B0` — Audit the shared signature model, generated runtime/declaration surfaces, and
   existing positional-call tests; define exact compatibility and diagnostics requirements.
2. `CALL-B1` — Add one signature representation that can render positional and named-object
   overloads without duplicating argument/type/default logic.
3. `CALL-B2` — Normalize both call forms once at the generated function boundary while retaining
   parameterless leaves as values and positional runtime behavior.
4. `CALL-B3` — Emit matching `.d.ts` overloads and JSDoc from the shared signature model, including
   missing/unknown-property rejection through TypeScript.
5. `CALL-B4` — Verify single-message and project backends, Svelte/Vite transforms, generated
   snapshots, real-site behavior, public docs, and broad regressions.

## Acceptance and verification

- Every parameterized message accepts its existing positional form and a typed named object.
- Both forms reach the same generated implementation body and produce identical output.
- Parameterless public leaves remain values and do not gain a call overload.
- Generated types reject missing, unknown, and incorrectly typed object properties.
- Shared and single-message compilers emit consistent signatures and documentation.
- Focused Rust/codegen/plugin tests pass per package; the final root full and site profiles pass;
  `git diff --check` remains clean.

## Completed predecessor

- Bundler deployment `BUNDLE-B0` through `BUNDLE-B5` is complete. Checklist `BUNDLE-A1` through
  `BUNDLE-A10` and `LOCAL-A1` through `LOCAL-A2` are verified and checked.
- The production site uses manifest v4 dynamic locale loading. Its graph gate proves 522 physical
  locale modules remain outside the initial static client graph while SSR stays synchronous.
- Public docs now describe the nested value/callable API, exact message imports, strict bounded
  dynamic access, dynamic locale preparation, and root verification profiles (`e8be06d`).
- Final predecessor regression gate: `pnpm test:full` passed all 11 registered tasks.

## Current state

- `CALL-B0` is active. Checklist `CALL-A1` through `CALL-A5` remain unchecked pending targeted
  evidence and implementation.
- Next action: inspect the shared codegen signature/runtime boundary and its TypeScript contract
  tests, then commit the smallest verified compatibility baseline before adding object overloads.
