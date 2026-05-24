export type Fruit = "apple" | "pear" | "orange";

export type Size = "small" | "big";

export type Money = number;

export type ShortDate = string;

export function selectBranch(
  key: string,
  branches: Record<string, string>,
): string {
  return branches[key] ?? branches._ ?? branches.other ?? "";
}
