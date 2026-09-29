import { selectBranch } from "../../shared.js";
import { pluralEnUs } from "./_runtime.js";

/** @typedef {"male" | "other"} Gender */
const prefix = "Total";

const __lgl_form_4672756974 = {
  apple: { Gender: "male", label: /** @type {(value: number | bigint | string) => string} */ ((value) => selectBranch(pluralEnUs(value), { one: "One " + String(prefix), _: "Many " + String(prefix) })) },
};

/**
 * @param {Gender} __lgl_p0
 * @param {number | bigint | string} __lgl_p1
 * @returns {string}
 */
function Render(__lgl_p0, __lgl_p1) {
  return selectBranch(String(__lgl_p0), { male: /** @type {() => string} */ (() => selectBranch(pluralEnUs(__lgl_p1), { one: /** @type {() => string} */ (() => "One"), _: /** @type {() => string} */ (() => "Many") })()), other: /** @type {() => string} */ (() => selectBranch(pluralEnUs(__lgl_p1), { one: /** @type {() => string} */ (() => "Other one"), _: /** @type {() => string} */ (() => "Other many") })()) })();
}

export { prefix, __lgl_form_4672756974, Render };
//# sourceMappingURL=_globals.js.map
