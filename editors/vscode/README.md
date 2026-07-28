# Linguini VS Code Extension

VS Code support for Linguini schema (`.lgs`) and locale (`.lgl`) files.

Published desktop VSIX packages contain one matching native Linguini server.
Development resolution is deterministic: explicit override, optional
workspace-pinned CLI, bundled binary, then a compatible CLI on `PATH`. Every
server must pass the protocol and compiler-version handshake.

## Features

- Language contributions for `.lgs` and `.lgl`.
- TextMate grammars for declarations, selectors, interpolations, formatters, raw locale text, strings, comments, and punctuation.
- Semantic token scope mappings for schema and locale files.
- Language client activation through `vscode-languageclient/node`.
- Document formatting is provided by the Linguini LSP.
- Config changes and manual restarts are debounced and serialized.
- Multi-root workspaces are passed through the LSP workspace-folders protocol;
  no arbitrary “first folder” is selected as the process root.
