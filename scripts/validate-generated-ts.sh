#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmpdir="$(mktemp -d /tmp/linguini-ts-run-XXXXXX)"
trap 'rm -rf "$tmpdir"' EXIT

cp "$repo_root/tests/fixtures/golden/snapshots/codegen-ts-plural-ru.ts" "$tmpdir/plurals.ts"
cp "$repo_root/tests/fixtures/golden/snapshots/codegen-ts-module-ru.ts" "$tmpdir/ru.ts"

tsc --strict --target ES2020 --module commonjs --outDir "$tmpdir/out" \
  "$tmpdir/plurals.ts" \
  "$tmpdir/ru.ts"

node - "$tmpdir/out/ru.js" <<'JS'
const m = require(process.argv[2]);

const expectations = [
  [m.counted(1, "apple"), "В корзине 1 яблока"],
  [m.counted(5, "orange"), "В корзине 5 апельсинов"],
  [m.delivery("apple", "small", 1), "Доставлено маленькое яблоко"],
  [m.email_input.label, "Email"],
  [m.createLinguini("ru").email_input.aria, "Адрес электронной почты"],
];

for (const [actual, expected] of expectations) {
  if (actual !== expected) {
    throw new Error(`expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}
JS
