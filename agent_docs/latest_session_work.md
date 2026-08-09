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
- Updated the remediation checklist through `CALL-A1`–`CALL-A6`, `P2-7`, `ESM-A9`, `API-D4`,
  `DOC-R3`, `DOC-G3`, and `DOC-F2`.
- Next work starts from the first dependency-ready unfinished remediation package after the
  bundler-native message API sequence.
