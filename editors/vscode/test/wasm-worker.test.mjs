import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import test from 'node:test';
import esbuild from 'esbuild';
import { chromium } from 'playwright';
import { nodeEndpoint } from '../out/wasm/node-endpoint.js';

const extension = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const wasm = path.join(extension, 'dist/wasm');
const harness = path.join(extension, 'out/wasm-harness.mjs');
await esbuild.build({ entryPoints: [path.join(extension, 'test/wasm-protocol.ts')], bundle: true,
  platform: 'browser', format: 'esm', target: 'es2022', outfile: harness });
const { exerciseWorker } = await import(pathToFileURL(harness));

test('built WASM serves the full shared protocol in Node and a browser worker', { timeout: 90_000 }, async () => {
  const manifest = JSON.parse(await readFile(path.join(wasm, 'manifest.json'), 'utf8'));
  const pkg = JSON.parse(await readFile(path.join(extension, 'package.json'), 'utf8'));
  assert.equal(manifest.version, pkg.version);
  assert.equal(manifest.protocolVersion, '1');
  for (const [file, digest] of Object.entries(manifest.sha256)) {
    assert.equal(createHash('sha256').update(await readFile(path.join(wasm, file))).digest('hex'), digest, file);
  }
  const bytes = await readFile(path.join(wasm, 'linguini_lsp_bg.wasm'));
  const node = await exerciseWorker(nodeEndpoint(path.join(wasm, 'worker-node.cjs')),
    pathToFileURL(path.join(wasm, 'linguini_lsp.js')).href, bytes);
  const server = createServer(async (request, response) => {
    try {
      const pathname = new URL(request.url, 'http://localhost').pathname;
      const file = path.resolve(extension, `.${decodeURIComponent(pathname)}`);
      if (!file.startsWith(extension + path.sep) || !(await stat(file)).isFile()) {
        response.writeHead(404).end(); return;
      }
      response.setHeader('content-type', file.endsWith('.wasm') ? 'application/wasm' : 'text/javascript');
      response.end(await readFile(file));
    } catch { response.writeHead(404).end(); }
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const origin = `http://127.0.0.1:${server.address().port}`;
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    await page.goto(`${origin}/package.json`);
    const web = await page.evaluate(async (origin) => {
      const { exerciseWorker } = await import(`${origin}/out/wasm-harness.mjs`);
      // The host supplies bytes; the worker performs no executable downloads.
      const bytes = new Uint8Array(await (await fetch(`${origin}/dist/wasm/linguini_lsp_bg.wasm`)).arrayBuffer());
      const worker = new Worker(`${origin}/dist/wasm/worker-browser.js`, { type: 'module' });
      return exerciseWorker({
        post: (message) => worker.postMessage(message),
        onMessage(listener) {
          const receive = (event) => listener(event.data);
          worker.addEventListener('message', receive);
          return { dispose: () => worker.removeEventListener('message', receive) };
        },
        onError(listener) {
          const receive = (event) => listener(new Error(event.message));
          worker.addEventListener('error', receive);
          return { dispose: () => worker.removeEventListener('error', receive) };
        },
        terminate: () => worker.terminate()
      }, `${origin}/dist/wasm/linguini_lsp.js`, bytes);
    }, origin);
    assert.deepEqual(web, node, 'native JS host and browser use identical compiled WASM semantics');
  } finally { await browser.close(); await new Promise((resolve) => server.close(resolve)); }
});
