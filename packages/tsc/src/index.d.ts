import type { Diagnostic } from "typescript";

export interface LinguiniTypeScriptOptions {
  cwd?: string;
  command?: string;
  nativeArgs?: string[];
  env?: Record<string, string | undefined>;
  debounceMs?: number;
  reportError?: (error: Error) => void;
  reportDiagnostic?: (diagnostic: Diagnostic) => void;
}

export interface LinguiniTypeScriptResult {
  status: number;
  close?: () => void;
}

export declare function runTypeScript(args?: string[], options?: LinguiniTypeScriptOptions): LinguiniTypeScriptResult;
