# @antepodeum/linguini-vite

Vite plugin for Linguini projects. Use `@antepodeum/linguini-vite/svelte` for Svelte and
SvelteKit applications. It composes the framework-neutral core with Svelte script scopes,
component cleanup, and static-link transforms. SvelteKit request and navigation behavior
remains in the native SvelteKit codegen module.

The main package entry exports the Vite core without framework imports or syntax. Its
`adapter` option selects generated entry/export names, runtime-helper descriptors, additional
source extensions, and optional scope/link/lifecycle hooks. Other integrations can supply a
matching native manifest through the build callback. The default uses `index.js`, the `lgl`
export, and the `locale` helper descriptor. The current native Svelte manifest uses the Svelte
adapter; the core does not infer an adapter from TOML.

It watches `linguini.toml`, module configuration and its imported files, and the `.lgs`/`.lgl`
files below the configured source roots. Changes are debounced and serialized, including changes that arrive
during a build. Additions and deletions refresh the watch set. Build failures use
Vite's error overlay.

```js
import { defineConfig } from "vite";
import linguini from "@antepodeum/linguini-vite/svelte";

export default defineConfig({
  plugins: [linguini()]
});
```

Options:

- `root`: project root. Defaults to Vite root.
- `configFile`: config path relative to root. Defaults to `linguini.toml`.
- `command`: Linguini executable. Defaults to `linguini`.
- `args`: build command arguments. Defaults to `["build"]`.
- `buildOnStart`: run codegen during Vite startup. Defaults to `true`.
- `debounceMs`: source-change debounce. Defaults to `40`.
- `generatedModulePatterns`: optional extra module substrings to invalidate. The
  configured `targets.ts.out` directory and Linguini virtual modules are detected
  automatically.

The published entry point is native Node ESM and supports Vite 5–8. The package
ships its source entry intentionally; it does not require a transpilation step.

Dynamic locale loading uses top-level await to prepare the initial locale before
message calls. When `locale_loading = "dynamic"` and `build.target` is unset, the
plugin selects `es2022`, including on Vite 5 and 6. Explicit targets are preserved;
they must support top-level await. Eager locale loading keeps Vite's default target.

Use `@antepodeum/linguini-sveltekit/vite` for SvelteKit and this package’s `/svelte` entry for
standalone Svelte. Runtime routing and persistence are configured through their npm adapters.
An optional `linkModule` selects a project helper for static-link transforms.
