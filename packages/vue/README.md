# @antepodeum/linguini-vue

Linguini integration for Vue 3.5 projects using Vite.

```ts
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import linguini from "@antepodeum/linguini-vue/vite";

export default defineConfig({ plugins: [linguini(), vue()] });
```

Use the generated `lgl` namespace in inline `<script>`/`<script setup>` blocks and template
interpolations or bound attributes. Configure application source roots under
`targets.ts.bundler.sources`. Client and SSR builds retain
only the page's used messages and their function dependencies in one composed chunk.

Messages use the project's base locale. Request-specific locales and runtime locale switching
are not supported by this integration yet.
