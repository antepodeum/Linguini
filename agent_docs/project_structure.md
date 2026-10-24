# Code generation

- `crates/linguini-codegen`: backend-independent type models, formatter/plural/inline-function
  requirement analysis, and transitive message dependency slicing with source provenance.
- `crates/linguini-codegen-ecmascript`: canonical JavaScript runtime syntax, adjacent
  declarations, identifier escaping, module artifacts, and source maps. It consumes shared
  semantic plans without depending on web or framework integrations.
- `crates/linguini-codegen-svelte`: thin project-specific Svelte import/export and type bindings.
  It references reusable npm factories and accepts an optional host module; it contains no
  generic state, controls, effects, web, or SvelteKit runtime implementation.
- `crates/linguini-codegen-vite`: framework-independent physical message/semantic leaves and
  manifest planning, accepting explicit runtime-helper descriptors from the host compiler.
- `packages/vite`: Vite lifecycle and application usage integration, with Svelte-specific syntax
  handling isolated behind the `/svelte` adapter.

- `packages/project`: shared JavaScript project-configuration reading and path resolution, with
  an ECMAScript layout entry used by the Vite and TypeScript adapters. Owns synchronous and
  asynchronous host parity and its packaged runtime/declaration contract.

- `crates/linguini-schema/src/highlight.rs`: source-span classification over the shared lexer
  tokens and semantic occurrences. LSP owns UTF-16/protocol conversion; the CLI and Markdown
  host expose that same pipeline. Site builds render packaged native highlight tokens.

# npm packages

All reusable npm packages live in `packages/`; there is no separate `plugins/` tree.
`packages/vite` and `packages/tsc` are build integrations, `packages/project` owns common
frontend project loading, and `packages/cli` with its platform packages provides the native
launcher. `packages/web`, `packages/svelte`, and `packages/sveltekit` contain TypeScript runtime adapter sources;
their JavaScript, declarations, and maps are emitted only into ignored `dist/` directories.
The site remains an application under `site/`, and the editor extension stays under `editors/`.

`packages/web` owns locale-source capabilities, URLs, persistence, and framework-independent
client controls. `packages/svelte` owns rune state and message providers. `packages/sveltekit`
composes those runtimes with request hooks, metadata serialization, and navigation, and exposes
`/vite` by composing the shared Svelte/Vite integration. Runtime policy is ordinary application
TypeScript; the native CLI reads neutral TOML and never evaluates an npm policy configuration.
