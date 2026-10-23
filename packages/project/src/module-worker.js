import { writeFileSync } from "node:fs";
import { createRequire, register } from "node:module";
import { pathToFileURL } from "node:url";
import { MessageChannel } from "node:worker_threads";
import { normalizeModulePlan } from "./modules.js";

// Both host modes reload the full config graph in Node. Native code receives data only.
const [source, output] = process.argv.slice(2);
const { port1, port2 } = new MessageChannel();
async function dependencies() {
  const files = await new Promise(resolve => {
    port1.once("message", resolve);
    port1.postMessage("dependencies");
  });
  return [...new Set([...files, ...Object.keys(createRequire(import.meta.url).cache)])].sort();
}
let registered = false;
try {
  register(new URL("./module-loader.js", import.meta.url), { data: { port: port2 }, transferList: [port2] });
  registered = true;
  const config = await import(pathToFileURL(source).href);
  const plan = normalizeModulePlan(config.default);
  writeFileSync(output, JSON.stringify({ plan, dependencies: await dependencies() }), { flag: "wx", mode: 0o600 });
} catch (error) {
  writeFileSync(output, JSON.stringify({ error: error?.message ?? String(error), dependencies: registered ? await dependencies() : [source] }), { flag: "wx", mode: 0o600 });
  process.exitCode = 1;
} finally {
  port1.close();
}
