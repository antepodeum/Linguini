# Generated-code fixture review

This review starts from the fixtures at `d42c133`. The inventory contains 910 files:
195 JavaScript, 515 TypeScript (including declarations and frozen comparison implementations),
155 source maps, 21 JSON, two Rust files, and 22 other input/expectation files. Executable
JavaScript/TypeScript/Rust contents form 244 distinct groups after identical copies are grouped.
The generated JavaScript compiler scan covers 171 roots, excluding consumers and the oldest
`snapshots/js` comparison implementation. Findings below are traced to their current generators;
frozen comparison code is evidence of earlier behavior, not a production implementation to edit.

## Acceptance

Resolve CLDR patterns, symbols, conditions, precision, and other static choices in Rust during
generation. Runtime parsing of actual user numeric/date values remains necessary. Runtime
parsing or revalidation of already compiled CLDR syntax and choices does not satisfy this
contract. Preserve exact arithmetic, negative zero, visible decimal digits, callable forms,
argument validation, source maps, SSR, and typed consumers while specializing code.

For each fix, inspect freshly regenerated output and execute representative cases. A smaller
snapshot alone does not establish correctness or faster execution. Page delivery still needs
the composed-page graph and request checks specified in ESM-A3/MOD-A4.

## Findings

| ID | Status | Evidence and generator responsibility | Required change and verification |
| --- | --- | --- | --- |
| GEN-A1 | Fixed in `d42c133` | `codegen-js-plural-ru.js`; `src/plural.rs`, plural templates, shared plural plans | Direct predicates, common guards, scalar operands, shared remainders; no generic relation calls or range/operand objects. Executed 24-locale/operator corpus and a Russian fraction test with bigint construction disabled. |
| GEN-A2 | Fixed | `codegen-js-formatters-ru.js`: `roundGeneratedDecimal(decimal, 3, "1")`; `number_helpers.jinja`, `formatters/number.jinja` | Fraction precision and quantum are known at build time, but each call validates/parses them, computes a remainder modulo one, and may construct a large power of ten for discarded digits. Emit specialized rounding from the validated request. Verify ties, carries, currency increments, scientific/significant rounding, huge exact values, and negative zero. |
| GEN-A3 | Fixed | Same currency fixture: `trimOptionalFractionDigits(rounded.fraction, 2)` with exactly two generated digits | Omit trimming when minimum and maximum fraction lengths coincide. Select the helper only when another actual request needs optional digits. Preserve significant-digit padding. |
| GEN-A4 | Fixed | Same date fixture: unused `padNumber`; `date_helpers.jinja`, `formatter_codegen.rs` | Derive padding/digit helper demand from the compiled date fields, including mixed requests. The medium Russian date needs no padding helper; short numeric dates still do. |
| GEN-A5 | Fixed | Numeric plans with scale; `number_helpers.jinja`: `scale === 1 ? ... : scale === 100 ? ...` | Fold the validated scale into its decimal shift during generation; omit runtime selection among unused scales. Preserve expansion limits and exact input semantics. |
| GEN-A6 | Open | `js-baseline/locales/ru/email_input.js` imports every global, though it only uses the Fruit form; root `ru.js` imports unused `DeliveryNote` | Derive global imports from the actual module dependency closure. Verify functions, forms, captured variables, and sparse fallback; a bundler removing unused imports does not prove generator ownership is exact. |
| GEN-A7 | Open | `js-svelte-control/{kit,plain}-{none,storage}` allocates unread `options`; `js-svelte-effects/*-none` emits unused capability reader, undefined link-effects object, and disposal scaffolding | Select options, imports, capability reads, and effect bodies from enabled features. Preserve public APIs, storage/cookie failure handling, navigation behavior, and HMR cleanup where effects exist. |
| GEN-A8 | Open | `js-web-server/*/web/switch-route.js`: each request finds and slices the literal `{locale}` marker in a static route | Compile the route prefix/suffix before emission. Keep project base paths dynamic where the host supplies them. Verify percent encoding, suffix matching, external return rejection, and redirect/cookie behavior. |
| GEN-A9 | Open | `js-web/empty/web.js` still exposes cookie, URL, and normalization machinery; other variants retain runtime policy branches | Compile only selected module capabilities when configuration is static. Preserve deliberately dynamic public factory contracts until their replacement is implemented. Complete the typed module-configuration migration rather than silently changing those contracts. |
| GEN-A10 | Open | `js-complete-project/kit/javascript/index.js` eagerly imports base-locale messages; `locale.js` contains a locale-resolution table; shared runtime exports general dispatch/argument helpers | Emit only the exact composed-page/locale closure, including folded fallback and selected resolution/helper operations. Lookup tables of resolved aliases are not a CLDR-pattern parser, but their initial delivery must still be justified. Verify actual server/client graphs and requests, not only individual exports. |
| GEN-A11 | Open | `js-baseline/locales/ru.js`: delivery repeats plural normalization through `Delivered`, `SizeAdj`, and the form, and wraps known string results in `String`; globals have dense nested form formatting | Reuse semantically identical pure calculations and remove proven redundant coercions without changing eager argument/capture evaluation or callable behavior. Improve formatting without extra runtime wrappers. Execute the function/form matrix and check source maps. |
| GEN-A12 | Open | `web/accept-language.js` filters and sorts preferences separately for each locale; runtime-link helpers check then reparse the same href; request context computes direction repeatedly | Replace repeated work with direct selection/shared internal results while preserving quality/specificity/order rules, safe link handling, callback evaluation semantics, and request isolation. These parse runtime inputs, not CLDR patterns. |
| GEN-A13 | Open | `formatter_codegen.rs::date_parts` currently substitutes fixed `AM`/`PM`, `AD`/`BC`, conflates format/standalone month context, and uses a default UTC expression for unhandled fields | Compile the correct CLDR calendar/context/day-period/era/zone/skeleton semantics. Output must use the selected data directly, with no runtime CLDR field interpreter. Add locale-specific expected results; do not preserve an incorrect expectation merely because it is already in a fixture. |
| GEN-A14 | Open | `normalizeMessageArgs` copies rest arguments on the positional path; callers allocate a static keys array each call; numeric grouping allocates/reverses a group array | Remove avoidable work using the known call signature/group sizes. Preserve named and positional calls, exact unknown-key errors, inherited/hostile-key behavior, and grouping semantics. Measure performance before claiming a speed improvement. |

Canonical number/date fixtures already contain build-selected symbols and direct date getter
expressions rather than raw CLDR patterns. Rounding/scale helper specialization and date-helper
selection have since been implemented and tested. Date-semantic substitutions and the other
open findings still need work. Historical TypeScript and oldest JavaScript comparison fixtures retain `Intl`, generic formatter forwarding, branch maps, and the retired
plural matcher. Their existence must not be mistaken for current generator acceptance.

## Queued syntax follow-up

After the current generator fixes, verify quotes and escapes in message text, the end-of-line
boundary for unquoted text, and multiline forms both preserving indentation and trimming a
common indent. Document the actual supported syntax with executed parser examples, including
failure cases. Do not claim quote handling or Kotlin-like indentation behavior is fixed without
those checks.
