# Code generation

- `crates/linguini-codegen`: backend-independent type models, formatter/plural/inline-function
  requirement analysis, and transitive message dependency slicing with source provenance.
- `crates/linguini-codegen-ecmascript`: JavaScript and TypeScript target syntax, identifier
  escaping, language runtime templates, module artifacts, and source maps. This backend consumes
  the shared code-generation plans without depending on web or framework integrations.
- `crates/linguini-codegen-ecmascript-web`: web policy validation, URL/request and locale-source
  templates, cookie/switch-route emitters, declarations, and selected web artifacts. Depends on
  ECMAScript rendering without a framework or Vite dependency.
- `crates/linguini-codegen-svelte`: Svelte locale state, controls, effects, and provider facades;
  consumes explicit host bindings and shared web capabilities.
- `crates/linguini-codegen-sveltekit`: SvelteKit request, navigation, and server adapters;
  composes Svelte and web artifacts.
- `crates/linguini-codegen-vite`: framework-independent physical message/semantic leaves and
  manifest planning, accepting explicit runtime-helper descriptors from the host compiler.
- `plugins/vite`: Vite lifecycle and application usage integration, with Svelte-specific syntax
  handling isolated behind the `/svelte` adapter.

- `packages/project`: shared JavaScript project-configuration reading and path resolution, with
  an ECMAScript layout entry used by the Vite and TypeScript adapters. Owns synchronous and
  asynchronous host parity and its packaged runtime/declaration contract.
