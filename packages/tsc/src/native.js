import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { findLinguiniProject, runtimeOutput } from "./projects.js";

export function prepareNative(graph, options, emission = "source") {
  const builds = new Map();
  const layouts = new Map();
  for (const [config, parsed] of graph.projects) {
    const layout = findLinguiniProject(config, layouts);
    if (!layout) continue;
    let build = builds.get(layout.root);
    if (!build) {
      build = { ...layout, outputs: new Set(), declarations: new Set() };
      builds.set(layout.root, build);
    }
    if (emission !== "source") {
      const types = runtimeOutput(parsed, layout, true);
      if (types) build.declarations.add(types);
    }
    if (emission === "runtime") {
      const output = runtimeOutput(parsed, layout);
      if (output) build.outputs.add(output);
    }
  }
  for (const build of builds.values()) {
    if (emission === "declarations" && !build.declarations.size) continue;
    if (emission === "runtime" && !build.outputs.size) continue;
    const args = [...(options.nativeArgs ?? ["build"]), ...[...build.outputs].sort().flatMap((output) => ["--emit-dir", output]), ...[...build.declarations].sort().flatMap((output) => ["--emit-declaration-dir", output])];
    const input = build.moduleConfigExists ? JSON.stringify(build.modulePlan) : undefined;
    if (input !== undefined) args.push("--module-plan-stdin");
    const result = spawnSync(options.command ?? "linguini", args, { cwd: build.root, input,
      stdio: input === undefined ? "inherit" : ["pipe", "inherit", "inherit"], env: options.env ?? process.env });
    if (result.error || result.status !== 0) {
      const error = result.error ?? new Error(`Native Linguini build failed (${result.signal ?? result.status}) in ${build.root}`);
      error.moduleDependencies = build.moduleFiles;
      throw error;
    }
  }
  return [...builds.values()];
}

export function declarationFingerprint(builds) {
  const hash = createHash("sha256");
  function visit(directory) {
    let entries;
    try { entries = readdirSync(directory, { withFileTypes: true }); }
    catch (error) { if (error.code === "ENOENT") return; throw error; }
    for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) visit(file);
      else if (entry.isFile() && file.endsWith(".d.ts")) hash.update(file).update("\0").update(readFileSync(file)).update("\0");
    }
  }
  for (const build of builds) visit(build.generated);
  return hash.digest("hex");
}
