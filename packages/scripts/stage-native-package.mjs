import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const protocolVersion = "1";
const platform = process.argv[2];
const source = process.argv[3] && path.resolve(process.argv[3]);
const packagesRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const launcher = JSON.parse(await readFile(path.join(packagesRoot, "cli", "package.json"), "utf8"));
const version = launcher.version;
const supported = new Set([
  "darwin-arm64",
  "darwin-x64",
  "linux-arm64",
  "linux-x64",
  "win32-arm64",
  "win32-x64"
]);

if (!supported.has(platform) || !source) {
  throw new Error(
    "usage: node packages/scripts/stage-native-package.mjs <platform-arch> <binary>"
  );
}

const executable = platform.startsWith("win32") ? "linguini.exe" : "linguini";
const destinationDir = path.join(packagesRoot, `cli-${platform}`, "bin");
const destination = path.join(destinationDir, executable);
const check = spawnSync(source, ["--version"], { encoding: "utf8", shell: false });
if (check.error || check.status !== 0 || check.stdout.trim() !== `linguini ${version}`) {
  throw check.error ?? new Error(`unexpected Linguini version: ${check.stdout || check.stderr}`);
}

await mkdir(destinationDir, { recursive: true });
await mkdir(path.resolve(packagesRoot, "..", "release"), { recursive: true });
await copyFile(source, destination);
if (!platform.startsWith("win32")) {
  await chmod(destination, 0o755);
}
const sha256 = createHash("sha256").update(await readFile(destination)).digest("hex");
await writeFile(
  path.join(destinationDir, "manifest.json"),
  `${JSON.stringify({ version, protocolVersion, platform, executable, sha256 }, null, 2)}\n`
);
