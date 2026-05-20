# @antepod/linguini-vite

Vite plugin for Linguini projects.

It watches `linguini.toml`, `.lgs`, and `.lgl` files, runs `linguini build`
after changes, invalidates generated Linguini modules, and emits a
`linguini:update` HMR event.

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
- `generatedModulePatterns`: substrings used to invalidate generated modules.
