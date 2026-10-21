import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

import * as oniguruma from "vscode-oniguruma";
import {
  INITIAL,
  parseRawGrammar,
  Registry,
  type IGrammar,
} from "vscode-textmate";

type LexerToken = {
  kind: string;
  start: number;
  end: number;
  text: string;
  line: number;
  column: number;
};

type ScopeToken = {
  start: number;
  end: number;
  scopes: string[];
};

type GrammarContract = {
  languageConfiguration: unknown;
  manifest: {
    languages: unknown;
    grammars: unknown;
  };
  schema: unknown[];
  locale: unknown[];
};

const extensionRoot = path.resolve(__dirname, "..");
const repositoryRoot = path.resolve(extensionRoot, "../..");
const goldenPath = path.join(
  extensionRoot,
  "test/golden/grammar-contract.json",
);

const fixtures = {
  schema: {
    source: path.join(
      repositoryRoot,
      "tests/fixtures/golden/syntax/all.lgs",
    ),
    tokens: path.join(
      repositoryRoot,
      "tests/fixtures/golden/snapshots/vscode-schema.tokens",
    ),
    scopeName: "source.linguini.schema",
    grammar: "syntaxes/linguini-schema.tmLanguage.json",
  },
  locale: {
    source: path.join(
      repositoryRoot,
      "tests/fixtures/golden/syntax/all.lgl",
    ),
    tokens: path.join(
      repositoryRoot,
      "tests/fixtures/golden/snapshots/vscode-locale.tokens",
    ),
    scopeName: "source.linguini.locale",
    grammar: "syntaxes/linguini-locale.tmLanguage.json",
  },
} as const;

let onigurumaReady: Promise<void> | undefined;

function loadOniguruma() {
  if (!onigurumaReady) {
    const wasm = fs.readFileSync(
      require.resolve("vscode-oniguruma/release/onig.wasm"),
    );
    onigurumaReady = oniguruma.loadWASM(wasm);
  }
  return onigurumaReady;
}

async function loadGrammar(scopeName: string, grammarPath: string) {
  await loadOniguruma();
  const absoluteGrammarPath = path.join(extensionRoot, grammarPath);
  const registry = new Registry({
    onigLib: Promise.resolve({
      createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
      createOnigString: (value) => new oniguruma.OnigString(value),
    }),
    loadGrammar: async (requestedScope) => {
      if (requestedScope !== scopeName) return null;
      return parseRawGrammar(
        fs.readFileSync(absoluteGrammarPath, "utf8"),
        absoluteGrammarPath,
      );
    },
  });
  const grammar = await registry.loadGrammar(scopeName);
  assert.ok(grammar, `grammar ${scopeName} must load`);
  return grammar;
}

function parseLexerSnapshot(source: string, snapshotPath: string) {
  const bytes = Buffer.from(source);
  return fs
    .readFileSync(snapshotPath, "utf8")
    .trimEnd()
    .split("\n")
    .map((line): LexerToken => {
      const match = /^(.*) @ (\d+)\.\.(\d+)$/.exec(line);
      assert.ok(match, `invalid lexer snapshot line: ${line}`);
      const start = Number(match[2]);
      const end = Number(match[3]);
      const prefix = bytes.subarray(0, start).toString("utf8");
      const positionLines = prefix.split(/\r\n|\r|\n/);
      return {
        kind: match[1].replace(/\(.*/, ""),
        start,
        end,
        text: bytes.subarray(start, end).toString("utf8"),
        line: positionLines.length - 1,
        column: positionLines.at(-1)?.length ?? 0,
      };
    });
}

function tokenize(grammar: IGrammar, source: string) {
  const lines = source.split(/\r\n|\r|\n/);
  const result: ScopeToken[][] = [];
  let stack = INITIAL;
  for (const line of lines) {
    const tokenized = grammar.tokenizeLine(line, stack);
    result.push(
      tokenized.tokens.map((token) => ({
        start: token.startIndex,
        end: token.endIndex,
        scopes: token.scopes,
      })),
    );
    stack = tokenized.ruleStack;
  }
  return result;
}

function scopeContract(
  scopeName: string,
  lexerTokens: LexerToken[],
  scopeLines: ScopeToken[][],
) {
  return lexerTokens
    .filter((token) => !["Whitespace", "Newline"].includes(token.kind))
    .map((token) => {
      // TextMate omits the newline token itself. For a semantic block newline,
      // prove its state carries into the first token of the following line.
      if (token.kind === "RawText" && /^(?:\r\n|\r|\n)$/.test(token.text)) {
        const nextScopes = scopeLines[token.line + 1]?.[0]?.scopes ?? [];
        const blockScope = "meta.value.multiline.linguini.locale";
        assert.ok(nextScopes.includes(blockScope), `block newline on line ${token.line + 1} lost its multiline state`);
        return {
          kind: token.kind,
          text: token.text,
          line: token.line + 1,
          column: token.column + 1,
          scopes: [blockScope],
        };
      }
      const endColumn = token.column + token.text.length;
      const overlaps = scopeLines[token.line].filter(
        (scope) => scope.end > token.column && scope.start < endColumn,
      );
      assert.ok(
        overlaps.length > 0,
        `${token.kind} ${JSON.stringify(token.text)} has no TextMate token`,
      );
      const scopes = [
        ...new Set(
          overlaps.map((scope) => scope.scopes.at(-1) ?? scopeName),
        ),
      ];
      assert.ok(
        scopes.some((scope) => scope !== scopeName),
        `${token.kind} ${JSON.stringify(token.text)} has only the root scope`,
      );
      return {
        kind: token.kind,
        text: token.text,
        line: token.line + 1,
        column: token.column + 1,
        scopes,
      };
    });
}

function assertLanguageConfigurationMatchesLexer(
  configuration: {
    comments?: { lineComment?: string };
    brackets?: string[][];
    autoClosingPairs?: Array<{ open: string; close: string }>;
    surroundingPairs?: Array<{ open: string; close: string }>;
    wordPattern?: string;
  },
  lexerTokens: LexerToken[],
) {
  const lexemes = new Set(lexerTokens.map((token) => token.text));
  const tokenKinds = new Set(lexerTokens.map((token) => token.kind));

  assert.equal(configuration.comments?.lineComment, "//");
  assert.ok(
    lexerTokens.some(
      (token) =>
        ["Comment", "DocComment"].includes(token.kind) &&
        token.text.startsWith(configuration.comments?.lineComment ?? ""),
    ),
    "configured line comment must be emitted by the lexer",
  );

  for (const pair of [
    ...(configuration.brackets ?? []).map(([open, close]) => ({
      open,
      close,
    })),
    ...(configuration.autoClosingPairs ?? []),
    ...(configuration.surroundingPairs ?? []),
  ]) {
    if (pair.open === "\"" && pair.close === "\"") {
      assert.ok(tokenKinds.has("String"));
      continue;
    }
    assert.ok(lexemes.has(pair.open), `lexer does not emit ${pair.open}`);
    assert.ok(lexemes.has(pair.close), `lexer does not emit ${pair.close}`);
  }

  const { wordPattern } = configuration;
  assert.ok(typeof wordPattern === "string");
  assert.doesNotThrow(() => new RegExp(wordPattern));
}

async function buildContract(): Promise<GrammarContract> {
  const manifest = JSON.parse(
    fs.readFileSync(path.join(extensionRoot, "package.json"), "utf8"),
  );
  const languageConfiguration = JSON.parse(
    fs.readFileSync(
      path.join(extensionRoot, "language-configuration.json"),
      "utf8",
    ),
  );

  const contracts = await Promise.all(
    Object.values(fixtures).map(async (fixture) => {
      const source = fs.readFileSync(fixture.source, "utf8");
      const lexerTokens = parseLexerSnapshot(source, fixture.tokens);
      const grammar = await loadGrammar(fixture.scopeName, fixture.grammar);
      const scopes = scopeContract(
        fixture.scopeName,
        lexerTokens,
        tokenize(grammar, source),
      );
      return { lexerTokens, scopes };
    }),
  );

  assertLanguageConfigurationMatchesLexer(languageConfiguration, [
    ...contracts[0].lexerTokens,
    ...contracts[1].lexerTokens,
  ]);

  for (const grammar of manifest.contributes.grammars) {
    const grammarPath = path.join(extensionRoot, grammar.path);
    assert.ok(fs.existsSync(grammarPath), `${grammar.path} must exist`);
    const grammarJson = JSON.parse(fs.readFileSync(grammarPath, "utf8"));
    assert.equal(grammarJson.scopeName, grammar.scopeName);
  }
  for (const language of manifest.contributes.languages) {
    assert.ok(
      fs.existsSync(path.join(extensionRoot, language.configuration)),
      `${language.configuration} must exist`,
    );
  }

  return {
    languageConfiguration,
    manifest: {
      languages: manifest.contributes.languages,
      grammars: manifest.contributes.grammars,
    },
    schema: contracts[0].scopes,
    locale: contracts[1].scopes,
  };
}

test("TextMate grammars and editor configuration match lexer goldens", async () => {
  const actual = await buildContract();
  if (process.env.LINGUINI_UPDATE_VSCODE_GOLDENS) {
    fs.mkdirSync(path.dirname(goldenPath), { recursive: true });
    fs.writeFileSync(goldenPath, `${JSON.stringify(actual, null, 2)}\n`);
  }

  const expected = JSON.parse(fs.readFileSync(goldenPath, "utf8"));
  assert.deepEqual(actual, expected);
});

test("locale text scopes select quoting at the value start and preserve multiline context", async () => {
  const grammar = await loadGrammar(fixtures.locale.scopeName, fixtures.locale.grammar);
  const source = [
    'plain = He said "hello" and measured 6"',
    'next = Still plain',
    'joined = Hello {name}, "friend".',
    'quoted = "\\\"hello\\\"\\n\\0"',
    'block = """',
    '  "hello" {name}',
    '  """',
    'wire = raw"""  "hello"',
    '  {name}  """',
    'last = Plain again',
    'inline = {fn() { _ => The screen is 6"}}!',
    'unicode_space = \u00a0"kept"',
    'unterminated = "broken',
    'after_error = Plain after an invalid literal',
  ].join("\n");
  const lines = source.split("\n");
  const scopes = tokenize(grammar, source);
  function at(line: number, text: string) {
    const column = lines[line].indexOf(text);
    assert.ok(column >= 0, `${line}: missing ${text}`);
    return scopes[line].find(token => token.start <= column && token.end > column)?.scopes ?? [];
  }
  for (const [line, text] of [[0, '"hello"'], [0, '6"'], [1, 'Still'], [2, '"friend"'], [9, 'Plain'], [10, '6"']] as const) {
    assert.ok(at(line, text).includes("string.unquoted.raw-text.linguini.locale"), `${line}: ${text} is raw text`);
    assert.ok(!at(line, text).includes("string.quoted.double.linguini.locale"), `${line}: ${text} must not start a quoted literal`);
  }
  assert.ok(at(2, "name").includes("meta.interpolation.linguini.locale"));
  assert.ok(at(3, "\\0").includes("constant.character.escape.linguini.locale"));
  for (const [line, text] of [[5, '"hello"'], [7, '"hello"']] as const) {
    assert.ok(at(line, text).includes("string.quoted.multi.linguini.locale"));
    assert.ok(!at(line, text).includes("string.quoted.double.linguini.locale"));
  }
  assert.ok(at(5, "name").includes("meta.interpolation.linguini.locale"));
  assert.ok(at(8, "name").includes("meta.interpolation.linguini.locale"));
  assert.ok(at(11, "kept").includes("string.quoted.double.linguini.locale"));
  assert.ok(at(13, "Plain").includes("string.unquoted.raw-text.linguini.locale"));
  assert.ok(!at(13, "Plain").includes("string.quoted.double.linguini.locale"));
});
