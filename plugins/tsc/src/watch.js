import path from "node:path";
import { readdirSync } from "node:fs";
import ts from "typescript";
import { declarationFingerprint, prepareNative } from "./native.js";

/** Own every compiler/native watcher and timer, including solution-builder resources. */
export function watchSystem(cwd) {
  const watchers = new Set();
  const timers = new Set();
  const system = { ...ts.sys, getCurrentDirectory: () => cwd };
  for (const name of ["watchFile", "watchDirectory"]) {
    system[name] = (...args) => {
      const watcher = ts.sys[name](...args);
      const owned = { close() { watchers.delete(owned); watcher.close(); } };
      watchers.add(owned);
      return owned;
    };
  }
  system.setTimeout = (callback, delay, ...args) => {
    const timer = setTimeout(() => { timers.delete(timer); callback(...args); }, delay);
    timers.add(timer);
    return timer;
  };
  system.clearTimeout = (timer) => { timers.delete(timer); clearTimeout(timer); };
  return {
    system,
    close() {
      for (const watcher of [...watchers]) watcher.close();
      for (const timer of timers) clearTimeout(timer);
      timers.clear();
    },
  };
}

export function watchNative(graph, builds, changed, system) {
  const watchers = [];
  const files = new Set([...graph.configFiles, ...builds.map((build) => build.configFile)]);
  for (const file of files) watchers.push(system.watchFile(file, changed, 250));
  const directories = new Set(builds.flatMap((build) => build.sources));
  let count = 0;
  function visit(directory, depth = 0) {
    if (depth > 64) throw new Error("Linguini watch source discovery exceeded its bounds");
    let entries;
    try { entries = readdirSync(directory, { withFileTypes: true }); }
    catch (error) { if (error.code === "ENOENT") return; throw error; }
    for (const entry of entries) {
      if (++count > 100_000) throw new Error("Linguini watch source discovery exceeded its bounds");
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) visit(file, depth + 1);
      else if (entry.isFile() && /\.(?:lgs|lgl)$/.test(file)) files.add(file);
    }
  }
  for (const directory of directories) visit(directory);
  // Directory watches discover additions/removals; content edits need file
  // watches because TypeScript can intentionally filter directory change events.
  for (const file of files) {
    if (/\.(?:lgs|lgl)$/.test(file)) watchers.push(system.watchFile(file, changed, 250));
  }
  for (let directory of directories) {
    while (!system.directoryExists(directory) && path.dirname(directory) !== directory) directory = path.dirname(directory);
    watchers.push(system.watchDirectory(directory, (file) => {
      const absolute = path.isAbsolute(file) ? file : path.resolve(directory, file);
      if ([...directories].some((source) => absolute === source || absolute.startsWith(source + path.sep) || source.startsWith(absolute + path.sep))) changed();
    }, true));
  }
  return () => { for (const watcher of watchers) watcher.close(); };
}

export function watchTypeScript(context, graph, builds) {
  const { cwd, buildMode, command, direct, root, roots, compilerOptions, report, readGraph, prepareTypes, deliver, options } = context;
  const resources = watchSystem(cwd);
  let closeNative = () => {};
  let timer;
  let checkTimer;
  let closed = false;
  let successful = false;
  let nativeValid = true;
  const failure = (error) => { nativeValid = false; (options.reportError ?? ((error) => console.error(`[linguini-tsc] ${error.message}`)))(error); };
  function emitRuntime() {
    if (nativeValid) deliver(graph);
  }
  function refreshSources() {
    graph = readGraph();
    builds = prepareNative(graph, options);
    graph = readGraph();
    prepareTypes(graph);
    nativeValid = graph.errors.length === 0;
    graph.errors.forEach(report);
    closeNative();
    closeNative = watchNative(graph, builds, schedule, resources.system);
  }
  function schedule() {
    clearTimeout(timer);
    timer = setTimeout(() => {
      if (closed) return;
      try {
        const before = declarationFingerprint(builds);
        refreshSources();
        // Message-only changes need runtime emission even when TS considers
        // every declaration/project reference up to date.
        if (successful && before === declarationFingerprint(builds)) emitRuntime();
      } catch (error) { failure(error); }
    }, options.debounceMs ?? 40);
  }
  const watchStatus = (diagnostic, _newline, _compilerOptions, errors) => {
    report(diagnostic);
    if (buildMode) {
      // Solution watch status does not supply a public error count. Query the
      // builder after its current turn completes, including up-to-date projects;
      // never infer success from localized diagnostic text or one project alone.
      resources.system.clearTimeout(checkTimer);
      checkTimer = resources.system.setTimeout(() => {
        if (closed) return;
        successful = compiler.build() === 0;
        if (successful) {
          try { emitRuntime(); } catch (error) { failure(error); }
        }
      }, 0);
    } else if (typeof errors === "number") successful = errors === 0;
  };
  const host = buildMode
    ? ts.createSolutionBuilderWithWatchHost(resources.system, undefined, report, report, watchStatus)
    : direct
      ? ts.createWatchCompilerHost(direct.fileNames, direct.options, resources.system, undefined, report, watchStatus)
      : ts.createWatchCompilerHost(root, compilerOptions, resources.system, undefined, report, watchStatus);
  const create = host.createProgram;
  host.createProgram = (...values) => {
    try { refreshSources(); } catch (error) { failure(error); }
    return create(...values);
  };
  const after = (builder) => {
    successful = !ts.getPreEmitDiagnostics(builder.getProgram()).some((item) => item.category === ts.DiagnosticCategory.Error);
    if (successful) {
      try { emitRuntime(); } catch (error) { failure(error); }
    }
  };
  let compiler;
  if (buildMode) {
    compiler = ts.createSolutionBuilderWithWatch(host, roots, command.options);
    successful = compiler.build() === 0;
    resources.system.clearTimeout(checkTimer);
    if (successful) emitRuntime(); // Up-to-date projects can skip the program callback.
  } else {
    const original = host.afterProgramCreate;
    host.afterProgramCreate = (builder) => { original?.(builder); after(builder); };
    compiler = ts.createWatchProgram(host);
  }
  closeNative();
  closeNative = watchNative(graph, builds, schedule, resources.system);
  return {
    status: successful && nativeValid ? 0 : 1,
    close() {
      closed = true;
      clearTimeout(timer);
      closeNative();
      if (!buildMode) compiler.close();
      resources.close();
    },
  };
}
