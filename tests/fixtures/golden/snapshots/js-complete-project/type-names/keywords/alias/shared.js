/** @typedef {string} __lgl_type_any */
/** @typedef {string} __lgl_type_unknown */
/** @typedef {string} __lgl_type_never */
/** @typedef {string} __lgl_type_number */
/** @typedef {string} __lgl_type_bigint */
/** @typedef {string} __lgl_type_boolean */
/** @typedef {string} __lgl_type_string */
/** @typedef {string} __lgl_name_scoped$symbol */
/** @typedef {string} __lgl_type_object */
/** @typedef {string} __lgl_type_undefined */
/** @typedef {string} __lgl_type_keyof */
/** @typedef {string} __lgl_type_unique */
/** @typedef {string} __lgl_type_readonly */
/** @typedef {string} __lgl_type_infer */
/** @typedef {string} __lgl_type_as */
/** @typedef {string} __lgl_name_766F6964 */
/** @typedef {string} intrinsic */
/** @typedef {string} type */
/** @typedef {string} abstract */
/** @typedef {string} constructor */
/** @typedef {string} __lgl_name_5F5F6C676C5F747970655F737472696E67 */

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
