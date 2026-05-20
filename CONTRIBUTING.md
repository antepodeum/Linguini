# Contributing

Keep the normal developer loop boring:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For the website:

```sh
cd site
pnpm install
pnpm check
```

Bug fixes should include a focused regression test at the layer where the bug
appears: parser, analyzer, codegen, CLI, or site.

## CLDR data

`linguini-cldr` is generated from pinned Unicode CLDR JSON data. Preview builds
can fetch that source during compilation when the local checkout is missing, so
building from a source archive may require `git` and network access. Release
builds should prefer checked-in generated CLDR data so downstream installs are
fully offline and deterministic.
