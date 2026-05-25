export type Fruit = "apple" | "pear" | "orange";

export type Size = "small" | "big";

export type Money = number;

export type ShortDate = Date | number | string;

export declare function selectBranch(
  key: string,
  branches: Record<string, string>,
): string;
