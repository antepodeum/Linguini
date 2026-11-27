export type Date = "entry";

export type globalThis = "namespace";

export type Input = globalThis.Date | number | string;

export declare function normalizeMessageArgs(
  values: unknown[],
  keys: readonly string[],
): unknown[];

export declare function normalizeMessageArgs(
  values: readonly unknown[],
  keys: readonly string[],
): readonly unknown[];
