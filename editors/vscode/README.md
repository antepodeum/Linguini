# Linguini VS Code Extension

VS Code support for Linguini schema (`.lgs`) and locale (`.lgl`) files.

## Features

- Language contributions for `.lgs` and `.lgl`.
- TextMate grammars for declarations, selectors, interpolations, formatters, raw locale text, strings, comments, and punctuation.
- Semantic token scope mappings for schema and locale files.
- Language client activation through `vscode-languageclient/node`.
- LSP starts from the bundled native `linguini` binary when the extension package includes one for the current VS Code platform.
- Document formatting is provided by the LSP formatting request.

Packaged users do not need to install the Linguini CLI separately. The extension falls back to `linguini` on `PATH` only when no bundled server binary exists or when `linguini.server.path` is overridden.

## Local Development

1. Install extension dependencies:

   ```sh
   cd editors/vscode
   npm install
   ```

2. Compile the extension:

   ```sh
   npm run compile
   ```

3. Build a bundled native server for the current host:

   ```sh
   npm run build:server
   ```

   To build a specific VS Code target:

   ```sh
   npm run build:server:target -- linux-x64
   npm run build:server:target -- darwin-arm64
   npm run build:server:target -- win32-x64
   ```

   Supported target names are `darwin-arm64`, `darwin-x64`, `linux-arm64`, `linux-armhf`, `linux-x64`, `alpine-arm64`, `alpine-x64`, `win32-arm64`, `win32-ia32`, and `win32-x64`.

4. Launch the Extension Development Host in one of two ways:

   - Open this extension folder in VS Code and press `F5`. The launch config opens `sample-workspace` automatically.
   - Or run:

     ```sh
     npm run open:dev
     ```

5. Open `sample-workspace/example.lgs` or `sample-workspace/en.lgl` in the Extension Development Host.

6. Check LSP behavior through diagnostics/completion/hover provided by `linguini lsp`.

7. Check formatting with `Format Document` / `Shift+Alt+F`. VS Code sends a formatting request to the running LSP.

If you want to test an external binary instead of the bundled one, set an explicit path in VS Code settings:

```json
{
  "linguini.server.path": "/absolute/path/to/target/debug/linguini",
  "linguini.server.args": ["lsp"]
}
```

`${workspaceFolder}` is supported in these settings when that is more convenient.

## Dependency audit

`@vscode/test-cli` was removed because its `mocha` dependency chain currently pulls audited vulnerable packages. Use the checked-in `.vscode/launch.json` or `npm run open:dev` for manual Extension Host testing.

Useful checks:

```sh
npm run compile
npm audit
npm run audit:prod
```
