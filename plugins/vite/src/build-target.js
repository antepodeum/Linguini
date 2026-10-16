/** Preserve explicit application targets; dynamic registries require top-level await. */
export function localeBuildConfig(localeLoading, build) {
  if (localeLoading !== "dynamic" || build?.target !== undefined) {
    return {};
  }
  return { build: { target: "es2022" } };
}
