export function platformPackage(platform, architecture) {
  const supported = new Map([
    ["darwin-arm64", "@antepodeum/linguini-darwin-arm64"],
    ["darwin-x64", "@antepodeum/linguini-darwin-x64"],
    ["linux-arm64", "@antepodeum/linguini-linux-arm64"],
    ["linux-x64", "@antepodeum/linguini-linux-x64"],
    ["win32-arm64", "@antepodeum/linguini-win32-arm64"],
    ["win32-x64", "@antepodeum/linguini-win32-x64"]
  ]);
  const key = `${platform}-${architecture}`;
  const selected = supported.get(key);
  if (!selected) {
    throw new Error(`Linguini has no native package for ${key}`);
  }
  return selected;
}
