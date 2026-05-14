#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmpdir="$(mktemp -d /tmp/linguini-ts-run-XXXXXX)"
trap 'rm -rf "$tmpdir"' EXIT

cp "$repo_root/tests/fixtures/golden/snapshots/ts/shared.ts" "$tmpdir/shared.ts"
cp "$repo_root/tests/fixtures/golden/snapshots/ts/index.ts" "$tmpdir/index.ts"
mkdir -p "$tmpdir/locales"
cp "$repo_root/tests/fixtures/golden/snapshots/ts/locales/ru.ts" "$tmpdir/locales/ru.ts"

tsc --strict --target ES2020 --module commonjs --outDir "$tmpdir/out" \
  "$tmpdir/shared.ts" \
  "$tmpdir/locales/ru.ts" \
  "$tmpdir/index.ts"

node - "$tmpdir/out/index.js" "$tmpdir/out/locales/ru.js" <<'JS'
const m = require(process.argv[2]);
const ru = require(process.argv[3]);

const expectations = [
  [ru.counted(0, "apple"), "В корзине 0 яблок"],
  [ru.counted(1, "apple"), "В корзине 1 яблоко"],
  [ru.counted(2, "pear"), "В корзине 2 груши"],
  [ru.counted(5, "orange"), "В корзине 5 апельсинов"],
  [ru.delivery("apple", "small", 1), "Доставлено маленькое яблоко"],
  [ru.delivery("apple", "small", 5), "Доставлены маленьких яблок"],
  [ru.delivery("pear", "big", 2), "Доставлены больших груши"],
  [ru.email_input.label, "Email"],
  [m.createLinguini("ru").email_input.aria, "Адрес электронной почты"],
  [m.createLinguiniProvider({ resolveLanguage: () => "ru" }).price(12, "13.05.2026"), "Цена 12,00 ₽ на 13.05.2026"],
  [m.lgl.price(12, "13.05.2026"), "Цена 12,00 ₽ на 13.05.2026"],
];

for (const [actual, expected] of expectations) {
  if (actual !== expected) {
    throw new Error(`expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}
JS

mkdir -p "$tmpdir/types/locales"
cp "$repo_root/tests/fixtures/golden/snapshots/ts/shared.d.ts" "$tmpdir/types/shared.d.ts"
cp "$repo_root/tests/fixtures/golden/snapshots/ts/index.d.ts" "$tmpdir/types/index.d.ts"
cp "$repo_root/tests/fixtures/golden/snapshots/ts/locales/ru.d.ts" "$tmpdir/types/locales/ru.d.ts"
cat > "$tmpdir/types/consumer.ts" <<'TS'
import { createLinguini, createLinguiniProvider, lgl, type Linguini } from "./index";

const direct: Linguini = createLinguini("ru");
const configured = createLinguiniProvider({ resolveLanguage: () => "ru" });

direct.delivery("apple", "small", 1);
configured.price(12, "13.05.2026");
lgl.delivery("apple", "small", 1);
TS

tsc --strict --target ES2020 --module commonjs --noEmit \
  "$tmpdir/types/consumer.ts"
