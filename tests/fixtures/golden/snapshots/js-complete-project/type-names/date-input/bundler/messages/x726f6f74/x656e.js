import { normalizeMessageArgs } from "../../../shared.js";
import { Forward } from "../../semantic/en/function/x466f7277617264.js";

/** @typedef {import("../../../shared.js").Date} Date */
/** @typedef {import("../../../shared.js").Input} Input */
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
export function message(...__lgl_args) {
  const [value, direct] = /** @type {[value: Input, direct: globalThis.Date | number | string]} */ (normalizeMessageArgs(__lgl_args, __lgl_keys_root));
  return Forward(value, direct);
}
//# sourceMappingURL=x656e.js.map
