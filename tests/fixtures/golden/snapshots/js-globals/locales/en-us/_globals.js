import { pluralEnUs } from "./_runtime.js";

/** @typedef {import("../../shared.js").Fruit} Fruit */

/** @typedef {"male" | "other"} Gender */
const prefix = "Total";

const __lgl_form_Fruit = {
  apple: {
    Gender: /** @type {const} */ ("male"),
    label: /** @type {(value: number | bigint | string) => string} */ ((value) => {
      if (pluralEnUs(value) === "one") return "One " + String(prefix);
      return "Many " + String(prefix);
    }),
  },
};

/**
 * @param {Gender} gender
 * @param {number | bigint | string} plural
 * @returns {string}
 */
function Render(gender, plural) {
  if (String(gender) === "male") {
    if (pluralEnUs(plural) === "one") return "One";
    return "Many";
  }
  if (pluralEnUs(plural) === "one") return "Other one";
  return "Other many";
}

export { prefix, __lgl_form_Fruit, Render };
//# sourceMappingURL=_globals.js.map
