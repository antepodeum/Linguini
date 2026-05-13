# Contributing

Every implementation slice must keep the repository gates passing:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-file-size.sh
./scripts/check-unit-test-structure.sh
./scripts/check-syntax-fixtures.sh
./scripts/review-snapshots.sh --check
./scripts/validate-generated-js.sh
./scripts/validate-generated-rust.sh
./scripts/check-spec-gates.sh
```

Bug fixes require a focused regression test committed with the fix. The test must fail against the
buggy behavior and pass after the fix. Prefer the narrowest layer that proves the behavior: unit
test for isolated logic, fixture or snapshot for DSL behavior, CLI integration test for command
behavior, and generated-output validation for target-language output.

Do not mark checklist work complete until the relevant test command is recorded as evidence in
`docs/checklist.md`.
