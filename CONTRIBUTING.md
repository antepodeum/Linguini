# Contributing

Keep the normal developer loop boring:

```sh
pnpm test:quick
```

The root runner is dependency-free and keeps every local project in one
ordered, offline-friendly command. It never installs packages or accesses the
network. Use the broader profiles when you need the corresponding gates:

```sh
pnpm test:full                 # Rust, Vite, CLI, VS Code, and site
pnpm test:site                 # generate, test, check, build, then inspect site graph
node scripts/test-all.mjs --profile full --project vite --project cli
node scripts/test-all.mjs --profile site --task site:check --task site:build --task site:graph
```

Repeated `--project` and `--task` options are supported. The runner performs a
preflight for required tools and existing `node_modules` directories, reports
an exact offline installation command when a dependency tree is missing, then
runs selected tasks sequentially and continues after individual failures.

For the website:

```sh
cd site
pnpm install --offline
pnpm check
```

Bug fixes should include a focused regression test at the layer where the bug
appears: parser, analyzer, codegen, CLI, or site.

## Rust toolchain

Development and normal CI jobs follow `stable`; run `rustup update stable` to update your local
installation. `[workspace.package].rust-version` declares the tested minimum rather than
selecting the development compiler. CI reads that value instead of repeating a version literal.

Install `cargo-msrv` with `cargo install cargo-msrv --locked`, then run `cargo msrv-find` to
rediscover the minimum. The alias runs `cargo msrv find -- cargo check --locked --workspace
--all-targets --ignore-rust-version`: discovery must ignore the existing declaration, or Cargo
will reject older compilers before checking whether the code actually works. Copy the measured
result to the workspace declaration and verify it with `cargo msrv verify --rust-version
<version> -- cargo check --locked --workspace --all-targets`. CI performs this verification too.
Do not use `--write-toolchain-file`; development should continue to follow stable.

## CLDR data

`linguini-cldr` consumes checked-in data generated from pinned Unicode CLDR JSON sources.
Normal compilation does not fetch CLDR or require a source checkout. Updating the pinned data
is an explicit offline-generation workflow; source archives include the compiled artifact.
