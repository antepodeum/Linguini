import {createServer} from 'node:http';
import {readFile,stat} from 'node:fs/promises';
import {extname,join,resolve,sep} from 'node:path';
import {fileURLToPath} from 'node:url';
import {Readable} from 'node:stream';

const site = fileURLToPath(new URL('../', import.meta.url));
const output = join(site, 'build');

export async function serve(context, {preload,ssr} = {}) {
  let releaseScripts;
  const scriptsReady = new Promise(resolve => {releaseScripts=resolve;});
  if (!preload) releaseScripts();
  const types = {'.html':'text/html', '.js':'text/javascript', '.json':'application/json', '.css':'text/css', '.svg':'image/svg+xml', '.woff2':'font/woff2'};
  const server = createServer(async (request, response) => {
    try {
      const pathname = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
      if (pathname === preload) releaseScripts();
      else if (pathname.includes('/immutable/entry/') && pathname.endsWith('.js')) await scriptsReady;
      if (ssr && (pathname.endsWith('/') || pathname.endsWith('/__data.json'))) {
        const headers = new Headers();
        for (const [name,value] of Object.entries(request.headers)) {
          if (value !== undefined) headers.set(name,Array.isArray(value) ? value.join(', ') : value);
        }
        const rendered = await ssr.respond(new Request(`http://${request.headers.host}${request.url}`,{headers}),{getClientAddress:()=> '127.0.0.1'});
        response.writeHead(rendered.status,Object.fromEntries(rendered.headers));
        if (rendered.body) Readable.fromWeb(rendered.body).pipe(response);
        else response.end();
        return;
      }
      let file = resolve(output, '.' + pathname);
      if (file !== output && !file.startsWith(output + sep)) throw Error('outside output');
      if ((await stat(file)).isDirectory()) file = join(file, 'index.html');
      const body = await readFile(file);
      response.writeHead(200, {'content-type':types[extname(file)] ?? 'application/octet-stream'});
      response.end(body);
    } catch {
      response.writeHead(404);
      response.end('Not found');
    }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  context.after(() => new Promise((resolve, reject) => {
    releaseScripts();
    server.close(error => error ? reject(error) : resolve());
    server.closeAllConnections();
  }));
  return `http://127.0.0.1:${server.address().port}`;
}

export async function createSsrServer() {
  const {Server} = await import('../.svelte-kit/output/server/index.js');
  const {manifest} = await import('../.svelte-kit/output/server/manifest-full.js');
  const server = new Server(manifest);
  await server.init({env:process.env});
  return server;
}
