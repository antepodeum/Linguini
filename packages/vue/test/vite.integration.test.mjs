import assert from "node:assert/strict";
import { writeFile } from "node:fs/promises";
import path from "node:path";
import { before, test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import vue from "@vitejs/plugin-vue";
import { createSSRApp } from "vue";
import { renderToString } from "vue/server-renderer";
import { build } from "vite";
import { linguini } from "../dist/vite.js";
import { buildNativeCompiler, nativeFrameworkProject, assertExactPage } from "../../../tests/support/native-vite.mjs";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
before(buildNativeCompiler);

for (const mode of ["setup", "script", "attribute", "attribute-partial"]) {
  test(`Vue ${mode} preserves exact native message leaves in client and SSR builds`, async (context) => {
    const components = {
      Page: `<!-- 😀 script offsets must remain UTF-8 safe -->
<script setup lang="ts">
import { lgl } from '../generated/index.js';
import Child from './Child.vue';
</script>
<template><main><h1>{{ lgl.main.title }}</h1><Child /></main></template>
`,
      Child: `<script setup lang="ts">
import { lgl } from '../generated/index.js';
const greeting: string = lgl.main.hello('Ada');
</script>
<template><p>{{ greeting }}</p></template>
`,
      Other: `<script setup lang="ts">
import { lgl } from '../generated/index.js';
</script>
<template><h1>{{ lgl.main.unused }}</h1></template>
`,
    };
    if (mode === "script") {
      components.Page = `<script lang="ts">
import { defineComponent } from 'vue';
import { lgl } from '../generated/index.js';
import Child from './Child.vue';
export default defineComponent({components:{Child},setup(){return {title:lgl.main.title};}});
</script>
<template><main><h1>{{ title }}</h1><Child /></main></template>
`;
    }
    if (mode.startsWith("attribute")) {
      components.Page = `<script setup lang="ts">
import { lgl } from '../generated/index.js';
import Child from './Child.vue';
${mode === "attribute-partial" ? "const title = lgl.main.title;" : ""}
</script>
<template><main><h1 :title="lgl.main.title">{{ ${mode === "attribute-partial" ? "title" : "lgl.main.title"} }}</h1><Child /></main></template>
`;
    }
    const project = await nativeFrameworkProject(context, {
      label: "vue", packageRoot, extension: "vue", components,
      references: mode.startsWith("attribute") ? ["main.title", "main.title"] : ["main.title"],
    });
    const { root, entry, messages } = project;
    for (const ssr of [false, true]) {
      const result = await build({
        root, configFile: false, logLevel: "silent",
        plugins: [linguini({ root, buildOnStart: false }), vue()],
        build: {
          ssr, write: false, minify: false, target: "es2022",
          rollupOptions: {
            input: entry, preserveEntrySignatures: "strict",
            external: (id) => /^(?:vue)(?:\/|$)/.test(id),
          },
        },
      });
      const code = assertExactPage(result, messages);
      const output = path.join(root, `output-${ssr}.js`);
      await writeFile(output, code);
      const { default: Page } = await import(pathToFileURL(output).href);
      const html = await renderToString(createSSRApp(Page));
      assert.equal(html, mode.startsWith("attribute")
        ? '<main><h1 title="PAGE_TITLE_SENTINEL">PAGE_TITLE_SENTINEL</h1><p>Hello Ada</p></main>'
        : "<main><h1>PAGE_TITLE_SENTINEL</h1><p>Hello Ada</p></main>");
    }
  });
}
