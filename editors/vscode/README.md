# Linguini VS Code Extension

VS Code support for Linguini schema (`.lgs`) and locale (`.lgl`) files.

Platform VSIX packages contain one matching native Linguini server and a bundled WASM worker.
The universal VSIX uses the WASM worker on desktop and in the browser. Both runtimes share the
Rust parser, analyzer, language features, and protocol/compiler-version handshake. Activation
uses the packaged files without downloading executables.

`linguini.server.runtime` selects `auto` (default), `native`, or `wasm`. On desktop file workspaces,
automatic native resolution checks the explicit override, optional workspace CLI, bundled binary,
then `PATH`. An unusable automatic candidate falls back to WASM. Explicit and workspace launchers
retain the LSP handshake, including command/argument setups such as `cargo run`; their startup
errors are reported. Browser and virtual workspaces use WASM in auto mode.

## Features

- Language contributions for `.lgs` and `.lgl`.
- Highlighting from the shared Rust lexer and semantic walk, including incomplete edits,
  unquoted text, interpolation, comments, and multiline strings.
- Markdown Linguini fences use the same Rust highlighter in either runtime.
- Semantic token scope mappings for schema and locale files.
- Native and worker language clients for desktop and browser extension hosts.
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

Highlighting requires the matching language server and semantic highlighting to be enabled.
`linguini.semanticHighlighting.enabled` controls both Linguini files and Markdown fences.
Theme fallback scopes provide colors without a separate language grammar. Quote auto-closing
is disabled because quotes can be ordinary characters in unquoted locale text; explicit quote
surrounding remains available.

## Build and verification

Use the moving stable Rust toolchain and add `wasm32-unknown-unknown` with `rustup target add`.
After `pnpm install --frozen-lockfile` in this directory, `pnpm bundle` builds both extension
entries, the shared WASM server, and its Node/browser workers. The build installs a matching
`wasm-bindgen-cli` into the repository's ignored `target` directory when needed; `WASM_BINDGEN`
can select an already installed matching tool. Generated bindings and binaries stay in `dist`.

`pnpm package:vsix` produces the universal VSIX. Set `VSCE_TARGET` to a native platform and
`LINGUINI_SERVER_BINARY` to its matching release CLI to package a native VSIX instead.
Each archive gets a SHA-256 sidecar. `LINGUINI_WASM_DIR` can supply a prebuilt runtime; bundling
verifies its version and file hashes before reuse.
`LINGUINI_VSIX_OUT_DIR` selects the archive/sidecar destination; release jobs collect native npm
and VSIX outputs together so artifact download preserves the flat publish layout.

`pnpm test` checks extension policy and workspace/URI handling. `pnpm test:wasm` builds and
executes the actual WASM protocol in Node and Chromium workers; install Chromium with
`pnpm exec playwright install chromium` first. `pnpm test:integration` runs VS Code's Extension
Host; `LINGUINI_TEST_VSIX` selects an archive to install into an isolated profile and
`LINGUINI_TEST_RUNTIME` chooses native, WASM, or an automatic fallback test with an empty `PATH`.
