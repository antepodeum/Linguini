import assert from "node:assert/strict";
import { writeFile } from "node:fs/promises";
import path from "node:path";
import { before, test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import react from "@vitejs/plugin-react";
import { createElement } from "react";
import { renderToString } from "react-dom/server";
import { build } from "vite";
import { linguini } from "../src/index.js";
import { buildNativeCompiler, nativeFrameworkProject, assertExactPage } from "../../../tests/support/native-vite.mjs";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
before(buildNativeCompiler);

test("React TSX preserves exact native message leaves in client and SSR builds", async (context) => {
  const components = {
    Page: `import { lgl } from '../generated/index.js';
import Child from './Child';
export default function Page() { return <main><h1>{lgl.main.title}</h1><Child /></main>; }
`,
    Child: `import { lgl } from '../generated/index.js';
export default function Child() { return <p>{lgl.main.hello(1, 'Ada')}</p>; }
`,
    Other: `import { lgl } from '../generated/index.js';
export default function Other() { return <h1>{lgl.main.unused}</h1>; }
`,
  };
  const { root, entry, messages } = await nativeFrameworkProject(context, {
    label: "react", packageRoot, extension: "tsx", components,
  });
  for (const ssr of [false, true]) {
    const result = await build({
      root, configFile: false, logLevel: "silent",
      plugins: [linguini({ root, buildOnStart: false }), react()],
      build: {
        ssr, write: false, minify: false, target: "es2022",
        rollupOptions: {
          input: entry, preserveEntrySignatures: "strict",
          external: (id) => /^(?:react|react-dom)(?:\/|$)/.test(id),
        },
      },
    });
    const code = assertExactPage(result, messages);
    const output = path.join(root, `output-${ssr}.js`);
    await writeFile(output, code);
    const { default: Page } = await import(pathToFileURL(output).href);
    assert.equal(renderToString(createElement(Page)), "<main><h1>PAGE_TITLE_SENTINEL</h1><p>Hello Ada</p></main>");
  }
});
