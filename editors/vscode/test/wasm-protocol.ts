import { WorkerChannel } from '../src/wasm/channel';
import type { WorkerEndpoint } from '../src/wasm/protocol';

function expect(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
const root = 'file:///work/caf%C3%A9%20space';
const schemaUri = `${root}/build/schema/shop.lgs`;
const localeUri = `${root}/locale/shop/en.lgl`;
const config = '[project]\nname = "shop"\ndefault_locale = "en"\nlocales = ["en", "fr"]\n[paths]\nschema = "build/schema"\nlocale = "locale"\n';
const schema = '/// Delivery label\ndelivery(count: Number)\n';
const locale = '// 😀 Unicode\ndelivery = 😀 {count}  \n';
const fallbackUri = `${root}/locale/fallback/fr.lgl`;
const fallbackBaseUri = `${root}/locale/fallback/en.lgl`;
const fallbackBase = 'let suffix = BASE\nfn render(count: Number) { _ => {count}|{suffix} }\nimpl Fruit { apple { label = Apple } }\nsummary = BASE\nother = OTHER\n';
const fallbackLocale = 'summary = {render(count)}|{fruit.label}|{suffix}\n';
const scopeUri = `${root}/locale/scope/en.lgl`;
const scopeLocale = 'fn helper(count: Number) { _ => {count} }\nfn Forward(helper: String, count: Number) { _ => {helper}|{helper(count)} }\nsummary = {fn(helper: value) { _ => {helper}|{helper(count)} }}\n';

/** Run the same real protocol scenario in a Node worker and a browser worker. */
export async function exerciseWorker(endpoint: WorkerEndpoint, moduleUri: string, bytes: Uint8Array): Promise<unknown[]> {
  const channel = new WorkerChannel(endpoint, moduleUri, bytes);
  const messages: any[] = [];
  const pending = new Map<number, { resolve(message: any): void; reject(error: Error): void }>();
  let id = 0;
  const send = (method: string, params?: unknown) => channel.send({ jsonrpc: '2.0', method, ...(params === undefined ? {} : { params }) });
  const request = (method: string, params?: unknown): Promise<any> => {
    const next = ++id;
    const timeout = setTimeout(() => { pending.get(next)?.reject(new Error(`request timed out: ${method}`)); pending.delete(next); }, 10_000);
    const promise = new Promise((resolve, reject) => pending.set(next, { resolve, reject }));
    channel.send({ jsonrpc: '2.0', id: next, method, ...(params === undefined ? {} : { params }) });
    return promise.finally(() => clearTimeout(timeout));
  };
  channel.onMessage((message: any) => {
    if (message.method && message.id !== undefined) {
      channel.send({ jsonrpc: '2.0', id: message.id, result: null });
    } else if (message.id !== undefined) {
      const waiter = pending.get(message.id);
      pending.delete(message.id);
      if (message.error) waiter?.reject(new Error(JSON.stringify(message.error)));
      else waiter?.resolve(message.result);
    } else { messages.push(message); }
  });
  channel.onError((error) => { for (const waiter of pending.values()) waiter.reject(error); });
  async function diagnostic(version: number, present: boolean, uri = localeUri,
    matches: (params: any) => boolean = () => true): Promise<any> {
    const started = Date.now();
    while (Date.now() - started < 10_000) {
      const event = messages.find((message) => message.method === 'textDocument/publishDiagnostics' && message.params.uri === uri && message.params.version === version && (message.params.diagnostics.length > 0) === present && matches(message.params));
      if (event) return event.params;
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
    throw new Error(`missing diagnostics for version ${version}: ${JSON.stringify(messages)}`);
  }
  try {
    const { ignoredDirectories: ignored } = await channel.ready;
    expect(ignored.includes('node_modules') && ignored.includes('build'), 'Rust discovery policy');
    const configFile = { uri: `${root}/linguini.toml`, text: config };
    await channel.replace([configFile]);
    const sourceRoots = (await channel.sourcePlan([root])).roots;
    expect(sourceRoots[0].schemaUri === `${root}/build/schema`, 'Rust configured roots');
    const files = [configFile, { uri: `${root}/broken/linguini.toml`, text: '[web]\n' }, { uri: schemaUri, text: schema }, { uri: localeUri, text: locale },
      { uri: `${root}/build/schema/account.lgs`, text: 'delivery(name: String)\n' },
      { uri: `${root}/locale/account/en.lgl`, text: 'delivery = Hello {name}\n' },
      { uri: `${root}/build/schema/fallback.lgs`, text: 'enum Fruit { apple }\nsummary(fruit: Fruit, count: Number)\nother\n' },
      { uri: fallbackBaseUri, text: fallbackBase }, { uri: fallbackUri, text: fallbackLocale },
      { uri: `${root}/build/schema/scope.lgs`, text: 'summary(value: String, count: Number)\n' },
      { uri: scopeUri, text: scopeLocale }];
    await channel.replace(files);
    const partialPlan = await channel.sourcePlan([root]);
    expect(partialPlan.roots.length === 1 && partialPlan.errors.length === 1, 'an invalid config must not hide valid projects');
    let rejected = false;
    try { await channel.replace([configFile, configFile]); } catch { rejected = true; }
    expect(rejected, 'duplicate workspace snapshot must fail atomically');
    expect(((await channel.sourcePlan([root])).roots)[0].schemaUri === sourceRoots[0].schemaUri, 'saved snapshot survives rejection');
    const initialized = await request('initialize', { capabilities: {}, workspaceFolders: [{ uri: root, name: 'shop' }] });
    expect(initialized.capabilities.renameProvider.prepareProvider === true, 'full server capabilities');
    send('initialized', {});
    const version = await request('linguini/protocolVersion');
    expect(version.protocolVersion === '1' && version.compilerVersion === '0.1.0-alpha.4', 'protocol and compiler version');
    send('textDocument/didOpen', { textDocument: { uri: localeUri, languageId: 'linguini-locale', version: 1, text: locale } });
    await diagnostic(1, false);
    const textDocument = { uri: localeUri };
    const at = { textDocument, position: { line: 1, character: 2 } };
    const hover = await request('textDocument/hover', at);
    expect(JSON.stringify(hover).includes('Delivery label'), 'cross-file schema docs');
    expect(!JSON.stringify(hover).includes('name: String'), 'namespace isolation');
    const definition = await request('textDocument/definition', at);
    expect(JSON.stringify(definition).includes(schemaUri), 'definition URI preserves encoded Unicode');
    const references = await request('textDocument/references', { ...at, context: { includeDeclaration: true } });
    expect(references.length === 2, 'only shop schema and locale references');
    const prepared = await request('textDocument/prepareRename', at);
    expect(prepared !== null, 'rename preparation');
    const rename = await request('textDocument/rename', { ...at, newName: 'shipping' });
    expect(JSON.stringify(rename).includes(schemaUri) && JSON.stringify(rename).includes(localeUri), 'workspace rename');
    expect(!JSON.stringify(rename).includes('account'), 'rename namespace isolation');
    const completion = await request('textDocument/completion', { textDocument, position: { line: 1, character: 18 } });
    expect(JSON.stringify(completion).includes('count'), 'parameter completion');
    const symbols = await request('textDocument/documentSymbol', { textDocument });
    expect(JSON.stringify(symbols).includes('delivery'), 'document symbols');
    const workspace = await request('workspace/symbol', { query: 'delivery' });
    expect(workspace.length >= 2, 'saved workspace symbols');
    const tokens = await request('textDocument/semanticTokens/full', { textDocument });
    expect(tokens.data.length > 0 && tokens.data.length % 5 === 0, 'semantic tokens');
    const edits = await request('textDocument/formatting', { textDocument, options: { tabSize: 2, insertSpaces: true, trimTrailingWhitespace: true } });
    expect(edits.length > 0 && edits[0].range.start.character === 21, `UTF-16 minimal formatting: ${JSON.stringify(edits)}`);
    const highlight = await request('linguini/highlight', { language: 'lgl', source: locale });
    expect(highlight.version === 1 && highlight.tokens.some((token: any) => token.line === 1 && token.start === 15), 'Unicode highlight coordinates');
    // Exercise error responses without killing the worker or its protocol loop.
    rejected = false;
    try { await request('linguini/highlight', { language: 'invalid', source: '' }); } catch { rejected = true; }
    expect(rejected, 'invalid protocol parameters return an error');
    send('textDocument/didChange', { textDocument: { uri: localeUri, version: 2 }, contentChanges: [{ text: 'delivery = {unknown}\n' }] });
    await diagnostic(2, true);
    const actions = await request('textDocument/codeAction', { textDocument, range: { start: { line: 0, character: 0 }, end: { line: 0, character: 20 } }, context: { diagnostics: [] } });
    expect(Array.isArray(actions), 'code action handler');
    send('textDocument/didChange', { textDocument: { uri: localeUri, version: 3 }, contentChanges: [{ text: locale }] });
    await diagnostic(3, false);
    // Saved-file changes reindex unopened schemas and preserve the open buffer version.
    await channel.update(schemaUri, '/// Updated label\ndelivery(count: Number)\n');
    send('workspace/didChangeWatchedFiles', { changes: [{ uri: schemaUri, type: 2 }] });
    const updated = await request('textDocument/hover', at);
    expect(JSON.stringify(updated).includes('Updated label'), 'saved-file index refresh');
    // A stale document update must not overwrite the recovered buffer.
    send('textDocument/didChange', { textDocument: { uri: localeUri, version: 2 }, contentChanges: [{ text: 'broken' }] });
    expect(JSON.stringify(await request('textDocument/hover', at)).includes('Updated label'), 'stale edit rejection');
    const noErrors = (params: any) => params.diagnostics.every((item: any) => item.severity !== 1);
    const errorContains = (text: string) => (params: any) => params.diagnostics.some((item: any) => item.severity === 1 && item.message.includes(text));
    send('textDocument/didOpen', { textDocument: { uri: fallbackUri, languageId: 'linguini-locale', version: 1, text: fallbackLocale } });
    const inherited = await diagnostic(1, true, fallbackUri, noErrors);
    expect(inherited.diagnostics.some((item: any) => item.message.includes('missing 1 schema message: `other`')), 'fallback keeps local translation coverage');
    const fallbackNavigation = [];
    for (const [name, line, character] of [['render', 1, 3], ['suffix', 0, 4]] as const) {
      const at = { textDocument: { uri: fallbackUri }, position: { line: 0, character: fallbackLocale.indexOf(name) } };
      const definition = await request('textDocument/definition', at);
      expect(definition.uri === fallbackBaseUri && definition.range.start.line === line
        && definition.range.start.character === character && definition.range.end.character === character + name.length,
      'inherited definition retains exact source ownership and span');
      const references = await request('textDocument/references', { ...at, context: { includeDeclaration: true } });
      expect(references.length === (name === 'suffix' ? 3 : 2) && references.every((item: any) => item.uri === fallbackBaseUri || item.uri === fallbackUri), 'inherited references include owner and consumer');
      const renamed = await request('textDocument/rename', { ...at, newName: 'renamed' });
      expect(Object.keys(renamed.changes).length === 2 && renamed.changes[fallbackBaseUri].length === (name === 'suffix' ? 2 : 1)
        && renamed.changes[fallbackUri].length === 1, 'inherited rename edits owner and consumer together');
      fallbackNavigation.push([definition, references, renamed]);
    }
    await channel.update(fallbackBaseUri, fallbackBase.replace('count: Number', 'count: String'));
    send('workspace/didChangeWatchedFiles', { changes: [{ uri: fallbackBaseUri, type: 2 }] });
    const changedFallback = await diagnostic(1, true, fallbackUri, errorContains('expects `String`, got `Number`'));
    expect(changedFallback.diagnostics.some((item: any) => item.relatedInformation?.some((related: any) => related.location.uri === fallbackBaseUri)), 'fallback errors retain related source URIs');
    const recoveredLocale = fallbackLocale.replace('render(count)', 'render(fruit.label)');
    send('textDocument/didChange', { textDocument: { uri: fallbackUri, version: 2 }, contentChanges: [{ text: recoveredLocale }] });
    const recoveredFallback = await diagnostic(2, true, fallbackUri, noErrors);
    await channel.update(fallbackBaseUri, fallbackBase);
    send('workspace/didChangeWatchedFiles', { changes: [{ uri: fallbackBaseUri, type: 2 }] });
    const restoredBase = await diagnostic(2, true, fallbackUri, errorContains('expects `Number`, got `String`'));
    send('textDocument/didChange', { textDocument: { uri: fallbackUri, version: 3 }, contentChanges: [{ text: fallbackLocale }] });
    await diagnostic(3, true, fallbackUri, noErrors);
    await channel.update(configFile.uri, config.replace('["en", "fr"]', '["en"]'));
    send('workspace/didChangeWatchedFiles', { changes: [{ uri: configFile.uri, type: 2 }] });
    const undeclared = await diagnostic(3, true, fallbackUri, errorContains('unknown function `render`'));
    await channel.update(configFile.uri, config);
    send('workspace/didChangeWatchedFiles', { changes: [{ uri: configFile.uri, type: 2 }] });
    send('textDocument/didChange', { textDocument: { uri: fallbackUri, version: 4 }, contentChanges: [{ text: fallbackLocale }] });
    const restoredFallback = await diagnostic(4, true, fallbackUri, noErrors);
    const overridden = `let suffix = LOCAL\n${fallbackLocale}`;
    send('textDocument/didChange', { textDocument: { uri: fallbackUri, version: 5 }, contentChanges: [{ text: overridden }] });
    const overriddenFallback = await diagnostic(5, true, fallbackUri, noErrors);
    send('textDocument/didOpen', { textDocument: { uri: fallbackBaseUri, languageId: 'linguini-locale', version: 1, text: fallbackBase } });
    await diagnostic(1, false, fallbackBaseUri, noErrors);
    const refusedRenames = [];
    for (const uri of [fallbackBaseUri, fallbackUri]) {
      const refused = await request('textDocument/rename', { textDocument: { uri }, position: { line: 0, character: 4 }, newName: 'renamed' }).then(
        () => { throw new Error('ambiguous inherited rename must be refused'); },
        (error: Error) => JSON.parse(error.message),
      );
      expect(refused.code === -32602, 'shared inherited body with different override bindings cannot be renamed safely');
      refusedRenames.push(refused);
    }
    const fallbackResults = [inherited, fallbackNavigation, changedFallback, recoveredFallback, restoredBase, undeclared, restoredFallback, overriddenFallback, refusedRenames];
    send('textDocument/didOpen', { textDocument: { uri: scopeUri, languageId: 'linguini-locale', version: 1, text: scopeLocale } });
    await diagnostic(1, false, scopeUri);
    const scopeResults = [];
    for (const [line, source] of scopeLocale.trimEnd().split('\n').entries()) {
      const character = source.indexOf('helper(count)');
      if (character < 0) continue;
      const at = { textDocument: { uri: scopeUri }, position: { line, character } };
      const definition = await request('textDocument/definition', at);
      expect(definition.uri === scopeUri && definition.range.start.line === 0 && definition.range.start.character === 3, 'named calls resolve to the function, preserving local bindings');
      const references = await request('textDocument/references', { ...at, context: { includeDeclaration: true } });
      expect(references.length === 3 && references.every((item: any) => item.uri === scopeUri), 'function references exclude same-named parameter and inline bindings');
      const rename = await request('textDocument/rename', { ...at, newName: 'render' });
      expect(rename.changes[scopeUri].length === 3 && Object.keys(rename.changes).length === 1, 'function rename edits only its declaration and calls');
      scopeResults.push([definition, references, rename]);
    }
    expect(scopeResults.length === 2, 'both named and inline binding scopes are exercised');
    await request('shutdown');
    send('exit');
    return [version, hover, definition, references, prepared, rename, completion, symbols, workspace, tokens, edits, highlight, actions, updated, fallbackResults, scopeResults];
  } finally { channel.dispose(); }
}
