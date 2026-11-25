# @antepodeum/linguini-vite

Vite plugin for Linguini projects. Use `@antepodeum/linguini-sveltekit/vite` for SvelteKit
and `@antepodeum/linguini-svelte/vite` for standalone Svelte. These adapters compose the
framework-neutral core with Svelte script scopes, component cleanup, and static-link
transforms. Maintained TypeScript npm packages own request and navigation behavior.

Vue uses `@antepodeum/linguini-vue/vite`; React uses this package directly alongside
its official Vite plugin. Framework syntax and lifecycle implementations live in their
owning packages.

The main package entry exports the Vite core without framework imports or syntax. Its
`adapter` option selects generated entry/export names, runtime-helper descriptors, additional
source extensions, and optional scope/link/lifecycle hooks. Other integrations can supply a
matching native manifest through the build callback. The default uses `index.js` and the
`lgl` export, which is bound to the project’s base locale. It links directly to compiled
message leaves without locale lookup, wrappers, loaders, or top-level await; the re-export
facades disappear during bundling. A stateful adapter selects a `localeHelper` descriptor
for its runtime. The native Svelte manifest uses the Svelte adapter; the core does not
infer an adapter from TOML.

Namespace imports use the same exact selection: `catalog.lgl.main.title` for the base-locale
entry, or `catalog.l.main.title` and `catalog.messages.main.title` for Svelte. Quoted members,
callable leaves, and finite computed access follow the same rules as named imports. Separate
metadata/control exports such as `catalog.setLocale` remain usable. Escaping the namespace,
unknown computed roots, and provider/catalog exports are rejected instead of retaining all
messages. Use matching native compiler and Vite adapter versions when rebuilding.

An adapter may declare `entrySideEffects: false` when its generated entry has no required
initialization effects. This allows direct re-exports to bypass unused entry code. The
Svelte adapter uses this contract so a setter-only import drops the message provider and
all catalog loaders. Custom host factories must keep construction free of required effects;
register application effects explicitly in their owning lifecycle.

It watches `linguini.toml`, `.lgs`/`.lgl` files below the configured source roots, and
configured application sources. Changes are debounced and serialized, including changes
that arrive during a build. Additions and deletions refresh the watch set. Build failures use
Vite's error overlay.

```js
import { defineConfig } from "vite";
import linguini from "@antepodeum/linguini-vite";

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

Use `@antepodeum/linguini-sveltekit/vite` for SvelteKit and `@antepodeum/linguini-svelte/vite` for
standalone Svelte. Runtime routing and persistence are configured through their npm adapters.
An optional `linkModule` selects a project helper for static-link transforms.

Framework adapters can select `pageMessages` and call `plugin.api.compilePages` once their
transformed graph is available. The compiler re-shakes each supplied page independently, accepts selected
nested export paths, and bundles only the surviving messages and their transitive compiled
dependencies. The host adapter owns route discovery, request locale selection, and page
publication. Normal, hidden, inline, and disabled source maps follow the Vite build setting.

Adapters implementing deferred preparation can instead call `plugin.api.compilePageBundles`.
It returns initial `pages`, additional `deferred` closures keyed by lazy module and page, build-only
`deferredImports` targets per page (including transit-only modules), and the optional generated
`localeModule` import specifier. Equal closures share emitted assets; additional payloads exclude
messages guaranteed in the initial page or on every parent import path. Shared entries intersect
their parent availability; cycles cannot justify preparation from future imports. Transit-only
modules emit no locale files. Hosts must prepare boundaries in application import order, since
child record sets can rely on prepared ancestors. `compilePages` continues
to reject new deferred messages for hosts without this protocol. Distinct additional export
demands from the same lazy module on one page are rejected until call-site preparation exists.

The optional `pageMessages.resolve(canonical)` asynchronously selects the client getter module
and export for a message. SSR retains direct compiled message bindings. Build-only integration
capabilities can provide an `analysisModule(id)` source model while their real implementation
awaits demand planning. Models must preserve application message demand; the Kit adapter models
only its own pending capability modules. The host must prepare each additional closure before
executing its application import and must reject boundaries it cannot lower exactly.
