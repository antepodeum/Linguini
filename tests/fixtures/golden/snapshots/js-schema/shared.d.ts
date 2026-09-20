/** Tone docs */
export type Tone = "warm" | "cool";

export type Amount = number | bigint | string;

export declare function selectBranch<T>(
  key: string,
  branches: Record<string, T>,
): T;

export declare function normalizeMessageArgs(
  values: readonly unknown[],
  keys: readonly string[],
): unknown[];
