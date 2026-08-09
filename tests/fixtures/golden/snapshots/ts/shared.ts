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

export function normalizeMessageArgs(
  values: readonly unknown[],
  keys: readonly string[],
): unknown[] {
  if (values.length !== 1) {
    return [...values];
  }
  const candidate = values[0];
  if (typeof candidate !== "object" || candidate === null) {
    return [...values];
  }
  const record = candidate as Record<string, unknown>;
  if (!keys.every((key) => Object.prototype.hasOwnProperty.call(record, key))) {
    return [...values];
  }
  const unknown = Reflect.ownKeys(record)
    .filter((key) => typeof key !== "string" || !keys.includes(key))
    .map(String)
    .sort();
  if (unknown.length > 0) {
    throw new TypeError(
      `Linguini message arguments contain unknown keys: ${unknown.join(", ")}`,
    );
  }
  return keys.map((key) => record[key]);
}
