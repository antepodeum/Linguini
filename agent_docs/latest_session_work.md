# Latest session work

- Removed all unreleased bundler manifest revisions and retained the complete active contract as
  manifest v1 only.
- Moved reusable formatter/runtime and semantic declarations out of physical message leaves;
  generated leaves now import exact shared dependencies and contain one message body.
- Completed positional-compatible named-object calls in `76059bc` using one signature model and
  one shared boundary normalizer. Parameterless messages remain values.
- Added generated overload snapshots, CLI assertions, real-site positive/negative TypeScript
  fixtures, positional/named runtime parity, unknown-property rejection, and cross-locale checks.
- Verified codegen (99 unit + 3 integration), CLI (78 unit + 24 integration), Vite (2 suites), and
  all five site gates. The site build contains 585 physical message leaves and no duplicated helper
  implementations.
- Added one reserved `_globals.ts` module per effective locale in `e0a82a3`. Legacy root and
  namespace modules import locale variables, forms, and functions from it instead of duplicating
  their definitions. The real site generates nine such modules and passes all five gates.
- Cleared the strict codegen Clippy blocker in `bd9c67e`; 100 codegen unit tests, three web safety
  tests, and `cargo clippy -p linguini-codegen-ts --all-targets -- -D warnings` pass.
- Closed #200 in `8b2bf7f`: the validated direct project boundary now requires complete effective
  locale message coverage after fallback composition while respecting tree-shaken visibility.
  Codegen now has 103 unit tests; the full CLI regression and strict codegen Clippy pass.
- Fixed browser startup locale initialization in `6542ce2`: guarded navigator language negotiation
  supplies the browser equivalent of server `Accept-Language` selection when headers are absent.
- Coalesced bundler message loading in `cd630fa`: 58 site message facades now share nine virtual
  locale entries, with zero physical message client entries and 21 total client JavaScript files
  instead of roughly 549. SSR remains synchronous and locale-granular HMR remains exact.
- Bundler parameterless leaves emit raw values; parameterized leaves remain functions and the public
  standalone single-message compiler preserves callable compatibility.
- Passed the full 11-task root profile plus focused post-refinement Vite tests. Independent
  verification reproduced browser runtime, production graph, SSR, and generated-leaf assertions
  without defects.
- Updated the remediation checklist through `CALL-A1`–`CALL-A6`, `P2-7`, `ESM-A9`, `API-D4`,
  `DOC-R3`, `DOC-G3`, `DOC-F2`, #200, #202, #204, and `BUNDLE-A13`; #199, #201, and #208 now
  record their verified bundler/legacy split honestly.
- Added `BUNDLE-A14`; refined #201 and BUNDLE-A10 with one-entry-per-locale evidence; marked
  WEB-A14 and P2-9 partial for browser negotiation parity while retaining their unfinished shared
  transition-plan scope.
- Completed the shared locale transition plan in `5e81c9a`: validated config emits one generated
  plan, browser mutation gates path/cookie/local-storage transports through it, and both SvelteKit
  adapters gate server cookie persistence through the same plan.
- Fixed source/declaration required-type parity in `193c83f` after the full site gate exposed five
  optional-plan diagnostics. Runtime fallback plans now derive from custom source arrays while an
  explicit generated plan remains authoritative.
- Passed the final full 11-task root profile. Focused evidence includes 103 codegen tests, four web
  template safety tests, 30 Bun runtime tests, 24 CLI integration tests, strict Clippy, zero Svelte
  errors/warnings, site production build, and production graph assertions.
- Closed WEB-A14 and P2-9 in the remediation checklist.
- Next work starts from the first dependency-ready unfinished remediation package after the
  shared locale transition plan.
