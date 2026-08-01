import assert from 'node:assert/strict';
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync
} from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { createServer } from 'vite';

const siteRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
const generatedRoot = join(siteRoot, 'src/lib/generated/linguini');

function numericPart(value) {
  const match = /-?[\d,]+(?:\.\d+)?/.exec(value);
  assert.ok(match, `expected numeric output in ${JSON.stringify(value)}`);
  return match[0];
}

test('executes generated inline, exact numeric, currency, and plural runtime', async (context) => {
  const directory = mkdtempSync(join(tmpdir(), 'linguini-runtime-'));
  const localeDirectory = join(directory, 'locales');
  mkdirSync(join(localeDirectory, 'en'), { recursive: true });
  mkdirSync(join(localeDirectory, 'ru'), { recursive: true });

  writeFileSync(
    join(directory, 'shared.ts'),
    readFileSync(join(generatedRoot, 'shared.ts'), 'utf8')
  );

  const enSourcePath = join(generatedRoot, 'locales/en/main.ts');
  const originalEnSource = readFileSync(enSourcePath, 'utf8');
  const lazyBranchPattern =
    /(size_line:[^\n]*?, _: \(\)(?:: string)? => )[^\n]+?( \}\)\(\)\)\(size,)/;
  const enSource = originalEnSource.replace(
    lazyBranchPattern,
    '$1(() => { throw new Error("unselected inline branch ran"); })()$2'
  );
  assert.notEqual(enSource, originalEnSource, 'generated inline branch shape changed');
  writeFileSync(
    join(localeDirectory, 'en/main.ts'),
    `${enSource}\nexport { formatNumber as __testFormatNumber, formatCurrency as __testFormatCurrency, pluralEn as __testPlural };\nexport const __testProto = (key: string) => selectBranch(key, { ["__proto__"]: () => "safe", _: () => "fallback" })();\n`
  );

  const ruSource = readFileSync(
    join(generatedRoot, 'locales/ru/main.ts'),
    'utf8'
  );
  writeFileSync(
    join(localeDirectory, 'ru/main.ts'),
    `${ruSource}\nexport { pluralRu as __testPlural };\n`
  );

  const server = await createServer({
    root: directory,
    configFile: false,
    logLevel: 'silent',
    appType: 'custom',
    server: { middlewareMode: true }
  });
  context.after(async () => {
    await server.close();
    rmSync(directory, { recursive: true, force: true });
  });

  const en = await server.ssrLoadModule('/locales/en/main.ts');
  const ru = await server.ssrLoadModule('/locales/ru/main.ts');

  assert.equal(en.main.playground.cart_summary('1', 'apple'), 'Cart has 1 apple');
  assert.equal(en.main.playground.cart_summary('2', 'pear'), 'Cart has 2 pears');
  assert.equal(en.main.playground.size_line('small'), 'Size form: compact');
  assert.equal(en.__testProto('__proto__'), 'safe');
  assert.equal(en.__testProto('missing'), 'fallback');

  assert.equal(
    en.__testFormatNumber(900719925474099312345n),
    '900,719,925,474,099,312,345'
  );
  assert.equal(
    en.__testFormatNumber('900719925474099312345.6789'),
    '900,719,925,474,099,312,345.679'
  );
  assert.equal(en.__testFormatNumber('999.9999'), '1,000');
  assert.equal(en.__testFormatNumber('-0'), '-0');
  assert.equal(en.__testFormatNumber('1.2345e3'), '1,234.5');
  assert.throws(() => en.__testFormatNumber('not-a-number'), RangeError);
  assert.throws(
    () => en.__testFormatNumber(`1e${'0'.repeat(8192)}1`),
    RangeError
  );

  assert.equal(
    numericPart(en.__testFormatCurrency('1.005', 2, 0, { code: 'USD' })),
    '1.01'
  );
  assert.equal(
    numericPart(en.__testFormatCurrency('1.25', 0, 0, { code: 'JPY' })),
    '1'
  );
  assert.ok(
    en.__testFormatCurrency('1.25', 0, 0, { code: 'jpy' }).includes('¥')
  );
  assert.equal(
    numericPart(en.__testFormatCurrency('1.2345', 3, 0, { code: 'KWD' })),
    '1.235'
  );
  assert.equal(
    numericPart(en.__testFormatCurrency('1.23456', 4, 0, { code: 'CLF' })),
    '1.2346'
  );
  assert.equal(
    numericPart(en.__testFormatCurrency('1.23', 2, 5, { code: 'CHF' })),
    '1.25'
  );

  assert.equal(en.__testPlural(1n), 'one');
  assert.equal(ru.__testPlural('10000000000000000000000000000000000000001'), 'one');
  assert.equal(ru.__testPlural('10000000000000000000000000000000000000011'), 'many');
  assert.equal(ru.__testPlural('1.0'), 'other');
  assert.equal(ru.__testPlural('1c0'), 'one');
  assert.throws(() => ru.__testPlural('x'), RangeError);
  assert.throws(() => ru.__testPlural('1'.repeat(8193)), RangeError);
});
