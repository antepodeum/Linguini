# Testing Policy

Every implementation task needs committed test evidence before it can be marked complete in
`docs/checklist.md`.

Required gates:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `./scripts/check-file-size.sh`
- `./scripts/check-unit-test-structure.sh`
- `./scripts/check-syntax-fixtures.sh`

Golden Linguini projects live under `tests/fixtures/golden`. Snapshot-style tests should compare
stable output with committed expected files, and may move to `insta` once external dependencies are
introduced.

Golden `.lqs` and `.lgl` syntax fixtures must be complete valid Linguini programs. Small fragments
belong only under `tests/fixtures/invalid` when they exist to assert a precise diagnostic.
