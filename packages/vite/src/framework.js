/** The Vite core accepts framework behavior through explicit, build-time callbacks. */
const ecmascript = Object.freeze({
  entry: "index.js",
  messageExports: Object.freeze(["lgl"]),
  localeHelper: "locale",
  extensions: Object.freeze([]),
});

export function frameworkAdapter(value = ecmascript) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError("Linguini adapter must be an object");
  }
  if (typeof value.entry !== "string" || !/^(?:[\w$.-]+\/)*[\w$.-]+\.js$/.test(value.entry)
    || value.entry.split("/").some((part) => part === "." || part === "..")) {
    throw new TypeError("Linguini adapter.entry must be a relative JavaScript module path");
  }
  if (!Array.isArray(value.messageExports) || value.messageExports.length === 0
    || !value.messageExports.every((name) => typeof name === "string" && name.length > 0)) {
    throw new TypeError("Linguini adapter.messageExports must be a non-empty string array");
  }
  for (const key of ["localeHelper", "effectsHelper"]) {
    if (key === "effectsHelper" && value[key] === undefined) continue;
    if (typeof value[key] !== "string" || !value[key]) {
      throw new TypeError(`Linguini adapter.${key} must be a non-empty string`);
    }
  }
  const extensions = value.extensions ?? [];
  if (!Array.isArray(extensions) || !extensions.every((extension) => typeof extension === "string" && /^\.[a-z][a-z0-9]*$/.test(extension))) {
    throw new TypeError("Linguini adapter.extensions must contain lowercase file extensions");
  }
  for (const key of ["applicationScope", "findStaticLinks", "applyStaticLinks", "injectLocaleLoaderLifecycle"]) {
    if (value[key] !== undefined && typeof value[key] !== "function") {
      throw new TypeError(`Linguini adapter.${key} must be a function`);
    }
  }
  if (Boolean(value.findStaticLinks) !== Boolean(value.applyStaticLinks)) {
    throw new TypeError("Linguini adapter static-link discovery and application must be supplied together");
  }
  return Object.freeze({
    ...value,
    messageExports: Object.freeze([...value.messageExports]),
    extensions: Object.freeze([...extensions]),
  });
}
