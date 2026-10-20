import { spawnSync } from "node:child_process";
import { realpathSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import ts from "typescript";
import { configPath, readProjects } from "./projects.js";
import { prepareNative } from "./native.js";
import { watchTypeScript } from "./watch.js";

/** Run native generation and TypeScript through public compiler/build/watch APIs. */
export function runTypeScript(args = [], options = {}) {
  const cwd = realpathSync(options.cwd ?? process.cwd());
  const buildMode = ts.isBuildCommand(args);
  const parsedCommand = buildMode ? ts.parseBuildCommand(args.slice(1)) : ts.parseCommandLine(args);
  const command = buildMode ? { ...parsedCommand, options: parsedCommand.buildOptions } : parsedCommand;
  const formatHost = { getCurrentDirectory: () => cwd, getCanonicalFileName: (file) => file, getNewLine: () => ts.sys.newLine };
  const report = options.reportDiagnostic ?? ((diagnostic) => ts.sys.write(ts.formatDiagnostic(diagnostic, formatHost)));
  if (command.errors.length) { command.errors.forEach(report); return { status: 1 }; }
  if (command.options.help || command.options.version || command.options.showConfig) {
    const require = createRequire(import.meta.url);
    const result = spawnSync(process.execPath, [require.resolve("typescript/bin/tsc"), ...args], { cwd, stdio: "inherit" });
    if (result.error) throw result.error;
    return { status: result.status ?? 1 };
  }
  const compilerOptions = { ...command.options };
  for (const name of ["outDir", "rootDir", "declarationDir", "tsBuildInfoFile", "outFile"]) {
    if (typeof compilerOptions[name] === "string") compilerOptions[name] = path.resolve(cwd, compilerOptions[name]);
  }
  if (!buildMode && command.fileNames.length && compilerOptions.project) throw new Error("Option 'project' cannot be mixed with source files");
  const root = buildMode ? undefined : compilerOptions.project
    ? configPath(compilerOptions.project, cwd)
    : ts.findConfigFile(cwd, ts.sys.fileExists);
  const roots = buildMode ? command.projects.map((project) => configPath(project, cwd)) : root ? [root] : [];
  const direct = !buildMode && command.fileNames.length ? {
    ...command,
    fileNames: command.fileNames.map((file) => path.resolve(cwd, file)),
    options: { ...compilerOptions },
  } : undefined;
  if (direct) roots.length = 0;
  if (!roots.length && !direct) throw new Error("Cannot find a TypeScript project; pass --project or source files");
  const readGraph = () => direct
    ? { projects: new Map([[path.join(cwd, "tsconfig.json"), direct]]), configFiles: new Set(), errors: [] }
    : readProjects(roots, compilerOptions, cwd, buildMode);
  let graph = readGraph();
  if (command.options.clean || command.options.dry) {
    const host = ts.createSolutionBuilderHost({ ...ts.sys, getCurrentDirectory: () => cwd }, undefined, report, report);
    const builder = ts.createSolutionBuilder(host, roots, command.options);
    return { status: command.options.clean ? builder.clean() : builder.build() };
  }
  const builds = prepareNative(graph, options);
  graph = readGraph(); // Include declarations/files that native generation just created.
  if (graph.errors.length) { graph.errors.forEach(report); return { status: 1 }; }
  const outputGraph = (current) => buildMode ? current : { ...current, projects: new Map([...current.projects].slice(0, 1)) };
  const prepareTypes = (current) => prepareNative(outputGraph(current), options, "declarations");
  const deliver = (current) => prepareNative(outputGraph(current), options, "runtime");
  prepareTypes(graph);
  if (command.options.watch) return watchTypeScript({ cwd, buildMode, command, direct, root, roots, compilerOptions, report, readGraph, prepareTypes, deliver, options }, graph, builds);
  let status;
  if (buildMode) {
    const host = ts.createSolutionBuilderHost({ ...ts.sys, getCurrentDirectory: () => cwd }, undefined, report, report);
    status = ts.createSolutionBuilder(host, roots, command.options).build();
  } else {
    const parsed = [...graph.projects.values()][0];
    const programOptions = { rootNames: parsed.fileNames, options: parsed.options, projectReferences: parsed.projectReferences, configFileParsingDiagnostics: parsed.errors };
    const compiler = parsed.options.incremental || parsed.options.composite
      ? ts.createIncrementalProgram({ ...programOptions, host: ts.createIncrementalCompilerHost(parsed.options) })
      : ts.createProgram(programOptions);
    const program = compiler.getProgram ? compiler.getProgram() : compiler;
    const diagnostics = ts.getPreEmitDiagnostics(program);
    const emitted = compiler.emit();
    [...diagnostics, ...emitted.diagnostics].forEach(report);
    status = [...diagnostics, ...emitted.diagnostics].some((item) => item.category === ts.DiagnosticCategory.Error) ? 1 : 0;
  }
  if (status === 0) deliver(graph);
  return { status };
}
