# @antepodeum/linguini-vue

TypeScript Vue build integration for Linguini. Vue 3.5's compiler locates SFC script
boundaries; the shared Vite core owns native generation, message linking, and watches.

```ts
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import linguini from "@antepodeum/linguini-vue/vite";

export default defineConfig({ plugins: [linguini(), vue()] });
```

Use the generated `lgl` namespace in inline `<script>`/`<script setup>` blocks and template
interpolations or bound attributes. Configure application source roots under
`targets.ts.bundler.sources`. The native build records exact source spans; the adapter inserts
imports inside the matching script block before Vue compilation. Client and SSR builds retain
only the page's used messages and their function dependencies in one composed chunk.

This initial boundary uses the project's compiled base locale. Request-selected locale state
and reactive switching remain under remediation. React uses the shared Vite entry directly and is verified by that package's integration tests.

Builds emit JavaScript, declarations, and maps into ignored `dist/`. Published archives contain
those artifacts and their TypeScript sources; generated application messages stay in build output.
