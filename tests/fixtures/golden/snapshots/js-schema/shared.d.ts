/** Tone docs */
export type Tone = "warm" | "cool";

export type Amount = number | bigint | string;

export declare function normalizeMessageArgs(
  values: unknown[],
  keys: readonly string[],
): unknown[];

export declare function normalizeMessageArgs(
  values: readonly unknown[],
  keys: readonly string[],
): readonly unknown[];
