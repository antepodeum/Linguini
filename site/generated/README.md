# Packaged site output

`linguini-output.br` is the deterministic, checksummed Linguini output used by normal site builds.
It lets a clean source checkout build and typecheck the site with Node.js only; Cargo, a Rust
toolchain, network access, and `git` are not used by `pnpm linguini:build`.

After changing the site's Linguini schema, locales, or generator, run `pnpm linguini:refresh` from
`site/`. That command runs the Rust generator once and replaces this bundle. CI independently
regenerates it and rejects stale bytes.
