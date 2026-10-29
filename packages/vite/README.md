# @antepodeum/linguini-vite

Vite plugin for Linguini projects. Use `@antepodeum/linguini-sveltekit/vite` for SvelteKit
and `@antepodeum/linguini-vite/svelte` for standalone Svelte. These adapters compose the
framework-neutral core with Svelte script scopes, component cleanup, and static-link
transforms. Maintained TypeScript npm packages own request and navigation behavior.

The main package entry exports the Vite core without framework imports or syntax. Its
`adapter` option selects generated entry/export names, runtime-helper descriptors, additional
source extensions, and optional scope/link/lifecycle hooks. Other integrations can supply a
matching native manifest through the build callback. The default uses `index.js` and the
`lgl` export, which is bound to the project’s base locale. It links directly to compiled
message leaves without locale lookup, wrappers, loaders, or top-level await; the re-export
facades disappear during bundling. A stateful adapter selects a `localeHelper` descriptor
for its runtime. The native Svelte manifest uses the Svelte adapter; the core does not
infer an adapter from TOML.

It watches `linguini.toml`, `.lgs`/`.lgl` files below the configured source roots, and
configured application sources. Changes are debounced and serialized, including changes
that arrive during a build. Additions and deletions refresh the watch set. Build failures use
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

Stateful dynamic locale loading uses top-level await to prepare the initial locale before
message calls. When an adapter selects `localeHelper`, `locale_loading = "dynamic"`, and
`build.target` is unset, the plugin selects `es2022`, including on Vite 5 and 6. Explicit
targets are preserved and must support top-level await. Eager locale loading and direct
base-locale `lgl` imports retain Vite’s default target.

Use `@antepodeum/linguini-sveltekit/vite` for SvelteKit and this package’s `/svelte` entry for
standalone Svelte. Runtime routing and persistence are configured through their npm adapters.
An optional `linkModule` selects a project helper for static-link transforms.

Framework adapters can select `pageMessages` and call `plugin.api.compilePages` during
`generateBundle`. The compiler re-shakes each supplied page independently, accepts selected
nested export paths, and bundles only the surviving messages and their transitive compiled
dependencies. The host adapter owns route discovery, request locale selection, and page
publication. Normal, hidden, inline, and disabled source maps follow the Vite build setting.
