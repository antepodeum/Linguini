# Linguini CLDR generator

This package is an offline maintainer tool. It is not a procedural macro and
never runs during a consumer build. `linguini-cldr` directly includes the
checked-in Rust artifact under `src/data/generated`.

Regenerate from the exact pinned CLDR checkout:

```sh
cargo run --offline -p linguini-cldr-macros --bin generate_cldr -- \
  crates/linguini-cldr-macros/vendor/cldr-json \
  crates/linguini-cldr/src/data/generated
```

Verify byte-for-byte reproducibility without writing:

```sh
cargo run --offline -p linguini-cldr-macros --bin generate_cldr -- \
  crates/linguini-cldr-macros/vendor/cldr-json \
  crates/linguini-cldr/src/data/generated \
  --check
```

Generation rejects any source whose full commit identity, package versions,
locale coverage, or deterministic consumed-input SHA-256 differs from the
pinned manifest. No fetch, checkout, lock, or deletion path exists.
