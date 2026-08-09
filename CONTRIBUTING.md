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

## CLDR data

`linguini-cldr` is generated from pinned Unicode CLDR JSON data. Preview builds
can fetch that source during compilation when the local checkout is missing, so
building from a source archive may require `git` and network access. Release
builds should prefer checked-in generated CLDR data so downstream installs are
fully offline and deterministic.
