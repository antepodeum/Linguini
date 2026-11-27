import { formatDate_skeleton_yw, formatDate_style_short } from "./_runtime.js";

/** @typedef {import("../../shared.js").Date} Date */
/** @typedef {import("../../shared.js").globalThis} globalThis */
/** @typedef {import("../../shared.js").Input} Input */

/**
 * @param {Input} value
 * @param {globalThis.Date | number | string} direct
 * @returns {string}
 */
function Forward(value, direct) {
  return formatDate_style_short(value) + "|" + formatDate_skeleton_yw(direct);
}

export { Forward };
