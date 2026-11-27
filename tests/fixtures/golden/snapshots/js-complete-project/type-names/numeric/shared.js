/** @typedef {"first"} GeneratedDecimal */
/** @typedef {"second"} GeneratedNumeric */

/**
 * @overload
 * @param {unknown[]} values
 * @param {readonly string[]} keys
 * @returns {unknown[]}
 */
/**
 * @overload
 * @param {readonly unknown[]} values
 * @param {readonly string[]} keys
 * @returns {readonly unknown[]}
 */
/**
 * @param {readonly unknown[]} values
 * @param {readonly string[]} keys
 * @returns {readonly unknown[]}
 */
export function normalizeMessageArgs(values, keys) {
  if (values.length !== 1) {
    return values;
  }
  const candidate = values[0];
  if (typeof candidate !== "object" || candidate === null) {
    return values;
  }
  const record = /** @type { { [key: string]: unknown } } */ (candidate);
  if (!keys.every((key) => Object.prototype.hasOwnProperty.call(record, key))) {
    return values;
  }
  /** @type {(string | symbol)[] | undefined} */
  let unknown;
  Reflect.ownKeys(record).forEach((key) => {
    if (typeof key !== "string" || !keys.includes(key)) {
      (unknown ??= []).push(key);
    }
  });
  if (unknown) {
    throw new TypeError(
      `Linguini message arguments contain unknown keys: ${unknown.map(String).sort().join(", ")}`,
    );
  }
  return keys.map((key) => record[key]);
}
