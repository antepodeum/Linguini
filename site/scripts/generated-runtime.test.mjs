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

test('executes generated JavaScript messages, lazy forms, exact numbers, currency, and plurals', async (context) => {
  const directory = mkdtempSync(join(tmpdir(), 'linguini-runtime-'));
  const localeDirectory = join(directory, 'locales');
  mkdirSync(join(localeDirectory, 'en'), { recursive: true });
  mkdirSync(join(localeDirectory, 'ru'), { recursive: true });

  writeFileSync(
    join(directory, 'shared.js'),
    readFileSync(join(generatedRoot, 'shared.js'), 'utf8')
  );
  for (const locale of ['en', 'ru']) {
    for (const module of ['_runtime.js', '_globals.js']) {
      writeFileSync(
        join(localeDirectory, locale, module),
        readFileSync(join(generatedRoot, 'locales', locale, module), 'utf8')
      );
    }
  }

  for (const locale of ['en', 'ru']) {
    writeFileSync(
      join(localeDirectory, locale, 'main.js'),
      readFileSync(join(generatedRoot, 'locales', locale, 'main.js'), 'utf8')
    );
  }
  // Poison one form branch: the unselected result must remain lazy after lowering.
  const globalsPath = join(localeDirectory, 'en/_globals.js');
  const globals = readFileSync(globalsPath, 'utf8');
  const poisoned = globals.replace('return "full-size";', 'throw new Error("unselected form branch ran");');
  assert.notEqual(poisoned, globals, 'the selected form branch must be instrumented');
  writeFileSync(globalsPath, poisoned);

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

  const en = await server.ssrLoadModule('/locales/en/main.js');
  const ru = await server.ssrLoadModule('/locales/ru/main.js');
  const enRuntime = await server.ssrLoadModule('/locales/en/_runtime.js');
  const ruRuntime = await server.ssrLoadModule('/locales/ru/_runtime.js');
  const shared = await server.ssrLoadModule('/shared.js');

  assert.equal(
    en.main.playground.cart_summary({ count: '1', fruit: 'apple' }),
    'Cart has 1 apple'
  );
  assert.equal(en.main.playground.cart_summary('2', 'pear'), 'Cart has 2 pears');
  assert.equal(ru.main.playground.cart_summary('2', 'pear'), 'В корзине 2 груши');
  assert.throws(
    () => en.main.playground.cart_summary({ count: '1', fruit: 'apple', extra: true }),
    /unknown keys: extra/
  );
  assert.equal(
    en.main.playground.date_format({ date: '2026-05-19' }),
    en.main.playground.date_format('2026-05-19')
  );
  assert.equal(
    en.main.playground.override_format({ date: '2026-05-19', amount: '1299.5' }),
    en.main.playground.override_format('1299.5', '2026-05-19')
  );
  assert.equal(en.main.playground.size_line('small'), 'Size form: compact');
  assert.throws(() => en.main.playground.size_line('big'), /unselected form branch ran/);
  const protoBranches = { ['__proto__']: () => 'safe', _: () => 'fallback' };
  assert.equal(shared.selectBranch('__proto__', protoBranches)(), 'safe');
  assert.equal(shared.selectBranch('missing', protoBranches)(), 'fallback');

  assert.equal(
    enRuntime.formatNumber(900719925474099312345n),
    '900,719,925,474,099,312,345'
  );
  assert.equal(
    enRuntime.formatNumber('900719925474099312345.6789'),
    '900,719,925,474,099,312,345.679'
  );
  assert.equal(enRuntime.formatNumber('999.9999'), '1,000');
  assert.equal(enRuntime.formatNumber('-0'), '-0');
  assert.equal(enRuntime.formatNumber('1.2345e3'), '1,234.5');
  assert.throws(() => enRuntime.formatNumber('not-a-number'), RangeError);
  assert.throws(
    () => enRuntime.formatNumber(`1e${'0'.repeat(8192)}1`),
    RangeError
  );

  assert.equal(numericPart(enRuntime.formatCurrency_code_USD('1.005')), '1.01');
  assert.ok(enRuntime.formatCurrency_code_USD('1.005').startsWith('$'));
  // Other currencies and rounding increments are compiled as distinct requests in
  // the backend's executed formatter corpus, rather than runtime options here.

  assert.equal(enRuntime.pluralEn(1n), 'one');
  assert.equal(ruRuntime.pluralRu('10000000000000000000000000000000000000001'), 'one');
  assert.equal(ruRuntime.pluralRu('10000000000000000000000000000000000000011'), 'many');
  assert.equal(ruRuntime.pluralRu('1.0'), 'other');
  assert.equal(ruRuntime.pluralRu('1c0'), 'one');
  assert.throws(() => ruRuntime.pluralRu('x'), RangeError);
  assert.throws(() => ruRuntime.pluralRu('1'.repeat(8193)), RangeError);
});

test('loads non-base locales once before synchronous access', async (context) => {
  const server = await createServer({
    root: generatedRoot,
    configFile: false,
    logLevel: 'silent',
    appType: 'custom',
    server: { middlewareMode: true }
  });
  context.after(() => server.close());

  const runtime = await server.ssrLoadModule('/index.js');
  assert.equal(runtime.createLinguini('en'), runtime.localeModules.en);
  assert.equal(runtime.localeModules.ru, undefined);
  assert.throws(
    () => runtime.createLinguini('ru'),
    /locale "ru" is not prepared; call prepareLinguini\(locale\) first/
  );

  const [first, second] = await Promise.all([
    runtime.prepareLinguini('ru'),
    runtime.prepareLinguini('ru')
  ]);
  assert.equal(first, second);
  assert.equal(runtime.createLinguini('ru'), first);
  assert.equal(runtime.localeModules.ru, first);
});
