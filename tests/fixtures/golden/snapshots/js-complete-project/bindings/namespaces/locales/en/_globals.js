import { pluralEn } from "./_runtime.js";

/** @typedef {import("../../shared.js").__lgl_name_String$Choice} __lgl_name_String$Choice */

/**
 * @param {number | bigint | string} plural
 * @returns {string}
 */
function __lgl_name_pluralEn$Count(plural) {
  if (pluralEn(plural) === "one") return "One";
  return "Many";
}

export { __lgl_name_pluralEn$Count };
//# sourceMappingURL=_globals.js.map
