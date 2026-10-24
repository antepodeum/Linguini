import { realpathSync } from "node:fs";
import path from "node:path";
import { readEcmascriptProjectSync } from "@antepodeum/linguini-project/ecmascript";
import ts from "typescript";

export function readProjects(roots, overrides, cwd, solution = false) {
  const projects = new Map();
  const configFiles = new Set();
  const errors = [];
  const host = {
    ...ts.sys,
    getCurrentDirectory: () => cwd,
    readFile(file) {
      configFiles.add(path.resolve(file));
      return ts.sys.readFile(file);
    },
    onUnRecoverableConfigFileDiagnostic: (error) => errors.push(error),
  };
  function visit(file, isRoot = false) {
    const config = realpathSync(file);
    if (projects.has(config)) return;
    const parsed = ts.getParsedCommandLineOfConfigFile(config, isRoot || solution ? overrides : undefined, host);
    if (!parsed) return;
    projects.set(config, parsed);
    errors.push(...parsed.errors);
    for (const reference of parsed.projectReferences ?? []) visit(ts.resolveProjectReferencePath(reference));
  }
  for (const root of roots) visit(root, true);
  return { projects, configFiles, errors };
}

export function configPath(value, cwd) {
  const candidate = path.resolve(cwd, value);
  return ts.sys.directoryExists(candidate) ? path.join(candidate, "tsconfig.json") : candidate;
}

export function findLinguiniProject(config, layouts = new Map()) {
  let root = path.dirname(config);
  while (true) {
    const configFile = path.join(root, "linguini.toml");
    if (ts.sys.fileExists(configFile)) {
      if (layouts.has(root)) return layouts.get(root);
      const layout = readEcmascriptProjectSync(root);
      if (!layout.targetConfigured) return undefined;
      for (const directory of [layout.generatedRoot, layout.schemaRoot, layout.localeRoot]) {
        if (directory === root) throw new Error(`Linguini path equals its project root: ${directory}`);
      }
      const project = {
        root,
        configFile,
        generated: layout.generatedRoot,
        sources: [layout.schemaRoot, layout.localeRoot],
      };
      layouts.set(root, project);
      return project;
    }
    const parent = path.dirname(root);
    if (parent === root) return undefined;
    root = parent;
  }
}

export function runtimeOutput(parsed, layout, declarations = false) {
  const options = parsed.options;
  const outputRoot = declarations ? options.declarationDir ?? options.outDir : options.outDir;
  if (options.noEmit || (!declarations && options.emitDeclarationOnly) || !outputRoot) return undefined;
  if (options.outFile) throw new Error("Linguini ESM output requires separate TypeScript output files, without outFile");
  // Use the public emitter path planner with the actual import graph. Declaration
  // inputs do not affect TypeScript's inferred common source directory.
  let files;
  if (options.rootDir || options.composite) {
    files = parsed.fileNames.filter((file) => !/\.d\.[cm]?ts$/.test(file));
  } else {
    const program = ts.createProgram({ rootNames: parsed.fileNames, options, projectReferences: parsed.projectReferences });
    files = program.getSourceFiles().filter((file) => !file.isDeclarationFile && !program.isSourceFileFromExternalLibrary(file)).map((file) => file.fileName);
  }
  const anchor = files.find((file) => /\.[cm]?[jt]sx?$/.test(file));
  if (!anchor) return undefined;
  let emissionOptions = declarations ? { ...options, declaration: true, emitDeclarationOnly: true } : options;
  if (!options.configFilePath) {
    emissionOptions = { ...emissionOptions, configFilePath: path.join(layout.root, "tsconfig.json"), rootDir: options.rootDir ?? commonDirectory(files) };
  }
  const output = ts.getOutputFileNames({ ...parsed, options: emissionOptions, fileNames: files }, anchor, !ts.sys.useCaseSensitiveFileNames)
    .find((file) => declarations ? /\.d\.[cm]?ts$/.test(file) : /\.(?:[cm]?js|jsx)$/.test(file));
  if (!output) return undefined;
  const destination = path.resolve(path.dirname(output), path.relative(path.dirname(anchor), layout.generated));
  if (!isWithin(path.resolve(outputRoot), destination)) {
    throw new Error(`Generated Linguini directory ${layout.generated} falls outside TypeScript's source root; set rootDir to include it`);
  }
  if (!isWithin(layout.root, destination)) {
    throw new Error(`Linguini output ${destination} must remain inside its project root ${layout.root}`);
  }
  return destination === layout.generated ? undefined : path.relative(layout.root, destination);
}

function isWithin(root, file) {
  const relative = path.relative(root, file);
  return relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
}

function commonDirectory(files) {
  let directory = path.dirname(files[0]);
  for (const file of files) {
    while (!isWithin(directory, file)) {
      const parent = path.dirname(directory);
      if (parent === directory) throw new Error("TypeScript source files have no common filesystem root");
      directory = parent;
    }
  }
  return directory;
}
