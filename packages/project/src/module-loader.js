import { fileURLToPath } from "node:url";

let port;
const files = new Set();

export function initialize(data) {
  port = data.port;
  port.on("message", () => port.postMessage([...files]));
  port.unref();
}

export async function resolve(specifier, context, nextResolve) {
  if (specifier.startsWith("file:")) files.add(fileURLToPath(specifier));
  else if (specifier.startsWith(".") && context.parentURL?.startsWith("file:")) {
    files.add(fileURLToPath(new URL(specifier, context.parentURL)));
  }
  const result = await nextResolve(specifier, context);
  if (result.url.startsWith("file:")) files.add(fileURLToPath(result.url));
  return result;
}
