export async function resolve(specifier, context, nextResolve) {
  if (specifier === "@antepod/linguini-web") {
    return {
      url: new URL("../../linguini-web/src/index.js", import.meta.url).href,
      shortCircuit: true,
    };
  }
  return nextResolve(specifier, context);
}
