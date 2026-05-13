#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmpdir="$(mktemp -d /tmp/linguini-js-run-XXXXXX)"
trap 'rm -rf "$tmpdir"' EXIT

cp "$repo_root/tests/fixtures/golden/snapshots/js/shared.js" "$tmpdir/shared.js"
cp "$repo_root/tests/fixtures/golden/snapshots/js/index.js" "$tmpdir/index.js"
mkdir -p "$tmpdir/locales"
cp "$repo_root/tests/fixtures/golden/snapshots/js/locales/ru.js" "$tmpdir/locales/ru.js"

cat > "$tmpdir/package.json" <<'JSON'
{ "type": "module" }
JSON

node --input-type=module - "$tmpdir/index.js" "$tmpdir/locales/ru.js" <<'JS'
const m = await import(process.argv[2]);
const ru = await import(process.argv[3]);

const expectations = [
  [ru.counted(1, "apple"), "В корзине 1 яблока"],
  [ru.counted(5, "orange"), "В корзине 5 апельсинов"],
  [ru.delivery("apple", "small", 1), "Доставлено маленькое яблоко"],
  [ru.email_input.label, "Email"],
  [m.createLinguini("ru").email_input.aria, "Адрес электронной почты"],
  [m.configureLinguini({ language: "ru" }).price(12, "13.05.2026"), "Цена 12,00 ₽ на 13.05.2026"],
  [m.lgl.price(12, "13.05.2026"), "Цена 12,00 ₽ на 13.05.2026"],
];

for (const [actual, expected] of expectations) {
  if (actual !== expected) {
    throw new Error(`expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}
JS
