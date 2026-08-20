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

## Language example

The editor understands recursive groups, attached API docs, parameterless value
leaves, and callable messages:

```lgs
account {
  /// Heading shown on the account page.
  title

  /// Greets the signed-in customer.
  greeting(name: String)

  receipt(order: String)
}
```

Locale implementations can use dedented multiline text while preserving hover,
definition, formatting, and semantic-token behavior:

```lgl
account {
  title = Account
  greeting = Hello, {name}!
  receipt = """
    Order {order}
    Thank you.
  """
}
```

Generated TypeScript exposes the value and both equivalent callable forms:

```ts
l.account.title;
l.account.greeting("Artemy");
l.account.greeting({ name: "Artemy" });
```
