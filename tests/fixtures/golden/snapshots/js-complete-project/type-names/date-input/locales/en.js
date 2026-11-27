import { normalizeMessageArgs } from "../shared.js";
import { Forward } from "./en/_globals.js";

/** @typedef {import("../shared.js").Date} Date */
/** @typedef {import("../shared.js").globalThis} globalThis */
/** @typedef {import("../shared.js").Input} Input */

const __lgl_keys_root = ["value", "direct"];
/**
 * @overload
 * @param {Input} value
 * @param {globalThis.Date | number | string} direct
 * @returns {string}
 */
/**
 * @overload
 * @param {{ value: Input; direct: globalThis.Date | number | string }} args
 * @returns {string}
 */
/**
 * @param {...*} __lgl_args
 * @returns {string}
 */
export function root(...__lgl_args) {
  const [value, direct] = /** @type {[value: Input, direct: globalThis.Date | number | string]} */ (normalizeMessageArgs(__lgl_args, __lgl_keys_root));
  return Forward(value, direct);
}

const lgl = {
  root,
};

export default lgl;
