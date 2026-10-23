import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { linguiniFences } from './markdown';

const extensionRoot = path.resolve(__dirname, '..');

test('highlighting has one language owner and retains language configuration', () => {
  const manifest = JSON.parse(fs.readFileSync(path.join(extensionRoot, 'package.json'), 'utf8'));
  assert.equal(manifest.contributes.grammars, undefined);
  assert.equal(fs.existsSync(path.join(extensionRoot, 'syntaxes')), false);
  for (const dependency of ['vscode-textmate', 'vscode-oniguruma']) {
    assert.equal(manifest.devDependencies[dependency], undefined);
  }
  const configuration = JSON.parse(fs.readFileSync(path.join(extensionRoot, 'language-configuration.json'), 'utf8'));
  assert.equal(configuration.comments.lineComment, '//');
  assert.deepEqual(configuration.brackets, [['{', '}'], ['(', ')']]);
  // Quotes are valid inside unquoted locale text; inserting a partner corrupts that text.
  assert.ok(configuration.autoClosingPairs.every((pair: { open: string }) => pair.open !== '"'));
  assert.doesNotThrow(() => new RegExp(configuration.wordPattern));
  for (const language of ['linguini-schema', 'linguini-locale']) {
    assert.equal(manifest.contributes.configurationDefaults[`[${language}]`]['editor.semanticHighlighting.enabled'], true);
    const scopes = manifest.contributes.semanticTokenScopes.find((entry: { language: string }) => entry.language === language).scopes;
    for (const kind of ['keyword', 'variable', 'type', 'enum', 'enumMember', 'parameter', 'function', 'operator', 'string', 'comment']) {
      assert.ok(scopes[kind]?.length, `missing ${kind} theme fallback`);
    }
  }
});

test('Markdown forwards only Linguini fences, with source-to-document positions', () => {
  const source = [
    '# Example', '  ```LGL fragment=demo', '  text = 🍎 "hello" {name}', ' x = raw"""', '  body', '  """', '  ````',
    '```text', '~~~lgs', 'not_linguini', '~~~', '```',
    '~~~linguini-schema', 'title(name: String)', '~~~',
    '````lgl', '```', 'text = Unclosed fence'
  ].join('\r\n');
  const blocks = linguiniFences(source);
  assert.equal(blocks.length, 3);
  assert.equal(blocks[0].language, 'linguini-locale');
  assert.equal(blocks[0].startLine, 2);
  assert.deepEqual(blocks[0].indentation, [2, 1, 2, 2]);
  assert.equal(blocks[0].source, 'text = 🍎 "hello" {name}\nx = raw"""\nbody\n"""');
  assert.equal(blocks[1].language, 'linguini-schema');
  assert.equal(blocks[1].startLine, 13);
  assert.equal(blocks[1].source, 'title(name: String)');
  assert.equal(blocks[2].source, '```\ntext = Unclosed fence');
  assert.equal(blocks[2].startLine, 16);
  assert.deepEqual(linguiniFences('    ```lgl\ntext = Not a fence\n    ```'), []);
  assert.deepEqual(linguiniFences('```lgl`invalid\ntext = Not a fence'), []);
});
