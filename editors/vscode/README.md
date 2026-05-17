# Linguini VS Code Extension

VS Code support for Linguini schema (`.lgs`) and locale (`.lgl`) files.

## Features

- Language contributions for `.lgs` and `.lgl`.
- TextMate grammars for declarations, selectors, interpolations, formatters, raw locale text, strings, comments, and punctuation.
- Semantic token scope mappings for schema and locale files.
- Language client activation through `vscode-languageclient/node`.
- Document formatting is provided by the Linguini LSP.

The extension is platform-independent. It does not bundle a native server binary; it starts `linguini lsp` from the command line instead. Users must install the Linguini CLI separately and make sure `linguini` is on `PATH`, or configure `linguini.server.path` to an absolute binary path.
