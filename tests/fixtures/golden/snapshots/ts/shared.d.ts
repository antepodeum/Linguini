export type Fruit = "apple" | "pear" | "orange";

export type Size = "small" | "big";

export type Money = number | bigint | string;

export type ShortDate = Date | number | string;

export declare function selectBranch<T>(
  key: string,
  branches: Record<string, T>,
): T;
