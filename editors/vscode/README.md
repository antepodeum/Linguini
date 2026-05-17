# Linguini VS Code Extension

VS Code support for Linguini schema (`.lgs`) and locale (`.lgl`) files.

## Features

- Language contributions for `.lgs` and `.lgl`.
- TextMate grammars for declarations, selectors, interpolations, formatters, raw locale text, strings, comments, and punctuation.
- Semantic token scope mappings for schema and locale files.
- Language client activation through `vscode-languageclient/node`.
- Document formatting is provided by the Linguini LSP.

The extension is platform-independent. It does not bundle a native server binary; it starts `linguini lsp` from the command line instead. Users must install the Linguini CLI separately and make sure `linguini` is on `PATH`, or configure `linguini.server.path` to an absolute binary path.

## Local Development

1. Install extension dependencies:

   ```sh
   cd editors/vscode
   npm install
   ```

2. Install or build the Linguini CLI so that the `linguini` command is available:

   ```sh
   cargo install --path ../../crates/linguini-cli
   ```

   Alternatively, point the extension at a local binary in VS Code settings:

   ```json
   {
     "linguini.server.path": "/absolute/path/to/target/debug/linguini",
     "linguini.server.args": ["lsp"]
   }
   ```

3. Compile the extension:

   ```sh
   npm run compile
   ```

4. Produce the universal VSIX package:

   ```sh
   npm run package:vsix
   ```

   The generated package is written to `dist/linguini-vscode.vsix` and contains no native binaries.

5. Launch the Extension Development Host in one of two ways:

   - Open this extension folder in VS Code and press `F5`. The launch config opens `sample-workspace` automatically.
   - Or run:

     ```sh
     npm run open:dev
     ```

6. Open `sample-workspace/example.lgs` or `sample-workspace/en.lgl` in the Extension Development Host.

7. Check LSP behavior through diagnostics/completion/hover provided by `linguini lsp`.

8. Check formatting with `Format Document` / `Shift+Alt+F`. VS Code sends a formatting request to the running LSP.

`${workspaceFolder}` is supported in `linguini.server.path` and `linguini.server.args` when that is more convenient.

## Dependency audit

`@vscode/test-cli` was removed because its `mocha` dependency chain currently pulls audited vulnerable packages. Use the checked-in `.vscode/launch.json` or `npm run open:dev` for manual Extension Host testing.

Useful checks:

```sh
npm run compile
npm audit
npm run audit:prod
```
