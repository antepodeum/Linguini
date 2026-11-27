# Linguini CLDR generator

Maintainer tool for updating Linguini’s CLDR data. Requires a local checkout of
CLDR JSON 48.2.0.

Regenerate the data:

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
