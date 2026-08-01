export type Fruit = "apple" | "pear" | "orange";

export type Size = "small" | "big";

export type Money = number | bigint | string;

export type ShortDate = Date | number | string;

export function selectBranch<T>(
  key: string,
  branches: Record<string, T>,
): T {
  if (Object.prototype.hasOwnProperty.call(branches, key)) {
    return branches[key];
  }
  if (Object.prototype.hasOwnProperty.call(branches, "_")) {
    return branches._;
  }
  if (Object.prototype.hasOwnProperty.call(branches, "other")) {
    return branches.other;
  }
  throw new Error(`Linguini dispatch has no branch for key ${JSON.stringify(key)}`);
}
