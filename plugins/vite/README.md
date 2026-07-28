# @antepod/linguini-vite

Vite plugin for Linguini projects.

It watches `linguini.toml` and only the `.lgs`/`.lgl` files below the configured
source roots. Changes are debounced and serialized, including changes that arrive
during a build. Additions and deletions refresh the watch set. Build failures use
Vite's error overlay.

```js
import { defineConfig } from "vite";
import linguini from "@antepod/linguini-vite";

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
