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
- Updated the remediation checklist through `CALL-A1`–`CALL-A6`, `P2-7`, `ESM-A9`, `API-D4`,
  `DOC-R3`, `DOC-G3`, `DOC-F2`, #200, #202, #204, and `BUNDLE-A13`; #199, #201, and #208 now
  record their verified bundler/legacy split honestly.
- Next work starts from the first dependency-ready unfinished remediation package after the
  bundler-native message API sequence.
