# Linguini VS Code Extension

VS Code support for Linguini schema (`.lgs`) and locale (`.lgl`) files.

## Features

- Language contributions for `.lgs` and `.lgl`.
- TextMate grammars for declarations, selectors, interpolations, formatters, raw locale text, strings, comments, and punctuation.
- Semantic token scope mappings for schema and locale files.
- Language client activation through `vscode-languageclient/node`.
- Document formatting requests routed to the Linguini language server.

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

4. Open this repository in VS Code, then press `F5` from `editors/vscode` to launch an Extension Development Host.

5. If the binary is not available as `linguini` on `PATH`, set:

   ```json
   {
     "linguini.server.path": "${workspaceFolder}/target/debug/linguini"
   }
   ```

The extension starts the server with `linguini lsp` by default. Override `linguini.server.args` if your local binary uses different language-server arguments.
