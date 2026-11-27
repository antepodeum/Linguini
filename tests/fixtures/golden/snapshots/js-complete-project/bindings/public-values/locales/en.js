import { normalizeMessageArgs } from "../shared.js";
import { formatNumber } from "./en/_runtime.js";

/** @typedef {import("../shared.js").Choice} Choice */

const __lgl_keys_formatNumber = ["value"];
const __lgl_keys_normalizeMessageArgs = ["value"];
const __lgl_keys_message = ["value"];
const __lgl_keys_String = ["value"];
/**
 * @overload
 * @param {number | bigint | string} value
 * @returns {string}
 */
/**
 * @overload
 * @param {{ value: number | bigint | string }} args
 * @returns {string}
 */
/**
 * @param {...*} __lgl_args
 * @returns {string}
 */
function __lgl_name_$formatNumber(...__lgl_args) {
  const [value] = /** @type {[value: number | bigint | string]} */ (normalizeMessageArgs(__lgl_args, __lgl_keys_formatNumber));
  return formatNumber(value);
}
export { __lgl_name_$formatNumber as formatNumber };

/**
 * @overload
 * @param {number | bigint | string} value
 * @returns {string}
 */
/**
 * @overload
 * @param {{ value: number | bigint | string }} args
 * @returns {string}
 */
/**
 * @param {...*} __lgl_args
 * @returns {string}
 */
function __lgl_name_$normalizeMessageArgs(...__lgl_args) {
  const [value] = /** @type {[value: number | bigint | string]} */ (normalizeMessageArgs(__lgl_args, __lgl_keys_normalizeMessageArgs));
  return formatNumber(value);
}
export { __lgl_name_$normalizeMessageArgs as normalizeMessageArgs };

/**
 * @overload
 * @param {number | bigint | string} value
 * @returns {string}
 */
/**
 * @overload
 * @param {{ value: number | bigint | string }} args
 * @returns {string}
 */
/**
 * @param {...*} __lgl_args
 * @returns {string}
 */
export function message(...__lgl_args) {
  const [value] = /** @type {[value: number | bigint | string]} */ (normalizeMessageArgs(__lgl_args, __lgl_keys_message));
  return formatNumber(value);
}

/**
 * @overload
 * @param {Choice} value
 * @returns {string}
 */
/**
 * @overload
 * @param {{ value: Choice }} args
 * @returns {string}
 */
/**
 * @param {...*} __lgl_args
 * @returns {string}
 */
function __lgl_name_$String(...__lgl_args) {
  const [value] = /** @type {[value: Choice]} */ (normalizeMessageArgs(__lgl_args, __lgl_keys_String));
  return String(value);
}
export { __lgl_name_$String as String };

const __lgl_name_$lgl = "Catalog";
export { __lgl_name_$lgl as lgl };

const lgl = {
  formatNumber: __lgl_name_$formatNumber,
  normalizeMessageArgs: __lgl_name_$normalizeMessageArgs,
  message,
  String: __lgl_name_$String,
  lgl: __lgl_name_$lgl,
};

export default lgl;
