# Code generation

- `crates/linguini-codegen`: backend-independent type/numeric plans, formatter/plural/inline-function
  requirement analysis, project locale matching/fallback composition, and transitive message
  dependency slicing with source provenance. Namespace projection and the shared message tree
  retain declaration metadata without depending on a target renderer. Form/function dispatch
  planning owns IR traversal and result merging; backends project selectors/text and render syntax.
  Dependency slices and semantic artifacts share source-provenance traversal, including nested
  inline inputs, call arguments, dispatch branches, and form values. Scoped expression-value
  inference distinguishes explicit formatted strings from unformatted forwarded values.
- `crates/linguini-cli/src/project/test_data`: bounded native sample matrices and previews.
  Each locale is prepared through common default/inline lowering once before its cases;
  the evaluator executes the planned formatter operations rather than deriving display defaults.
- `crates/linguini-codegen-ecmascript`: canonical JavaScript runtime syntax, adjacent
  declarations, identifier escaping, module artifacts, and source maps. It consumes shared
  semantic plans without depending on web or framework integrations.
- `crates/linguini-codegen-svelte`: thin project-specific Svelte import/export and type bindings.
  It references reusable npm factories and accepts an optional host module; it contains no
  generic state, controls, effects, web, or SvelteKit runtime implementation.
- `crates/linguini-codegen-vite`: framework-independent physical message/semantic leaves and
  manifest planning, accepting explicit runtime-helper descriptors from the host compiler.
- `packages/vite`: framework-neutral Vite lifecycle, exact linking, and dependency planning.
- `packages/svelte`: Svelte runtime and the `/vite` adapter's script, link, and cleanup behavior.
- `packages/vue`: TypeScript `/vite` adapter using Vue's compiler for SFC script boundaries.

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
client controls. `packages/svelte` owns rune state, message providers, and the Svelte `/vite` adapter.
`packages/vue` owns the Vue `/vite` adapter; generic React consumers use `packages/vite`. `packages/sveltekit`
composes those runtimes with request hooks, metadata serialization, and navigation, and exposes
`/vite` by composing the shared Svelte/Vite integration. Runtime policy is ordinary application
TypeScript; the native CLI reads neutral TOML and never evaluates an npm policy configuration.
