# Linguini VS Code Extension

VS Code support for Linguini schema (`.lgs`) and locale (`.lgl`) files.

## Features

- Language contributions for `.lgs` and `.lgl`.
- TextMate grammars for declarations, selectors, interpolations, formatters, raw locale text, strings, comments, and punctuation.
- Semantic token scope mappings for schema and locale files.
- Language client activation through `vscode-languageclient/node`.
- LSP starts with `linguini lsp` by default.
- Document formatting runs `linguini formatting` by default, sends the document text to stdin, and expects the formatted document on stdout.

## Local Development

1. Build or install the Linguini binary so VS Code can run it:

   ```sh
   cargo build -p linguini-cli
   ```

2. Install extension dependencies:

   ```sh
   cd editors/vscode
   npm install
   ```

3. Compile the extension:

   ```sh
   npm run compile
   ```

4. Launch the Extension Development Host in one of two ways:

   - Open this extension folder in VS Code and press `F5`. The launch config opens `sample-workspace` automatically.
   - Or run:

     ```sh
     npm run open:dev
     ```

5. Open `sample-workspace/example.lgs` or `sample-workspace/en.lgl` in the Extension Development Host.

6. Check LSP behavior through diagnostics/completion/hover provided by `linguini lsp`.

7. Check formatting with `Format Document` / `Shift+Alt+F`. The extension runs:

   ```sh
   linguini formatting
   ```

   The current document is passed on stdin. The formatted document must be printed to stdout.

If the binary is not available as `linguini` on `PATH`, set explicit paths in VS Code settings. Use your actual binary path, for example:

```json
{
  "linguini.server.path": "/absolute/path/to/target/debug/linguini",
  "linguini.server.args": ["lsp"],
  "linguini.formatter.path": "/absolute/path/to/target/debug/linguini",
  "linguini.formatter.args": ["formatting"]
}
```

`${workspaceFolder}` is supported in these settings when that is more convenient.

`linguini.formatter.args` supports `${file}`, `${workspaceFolder}`, and `${languageId}` placeholders if the CLI later needs file-aware arguments, for example:

```json
{
  "linguini.formatter.args": ["formatting", "--stdin-filepath", "${file}"]
}
```

## Dependency audit

`@vscode/test-cli` was removed because its `mocha` dependency chain currently pulls audited vulnerable packages. Use the checked-in `.vscode/launch.json` or `npm run open:dev` for manual Extension Host testing.

Useful checks:

```sh
npm run compile
npm audit
npm run audit:prod
```
