import { section } from "./en/section.js";
import { normalizeMessageArgs } from "../shared.js";

export { section };

/**
 * Greeting
 *
 * @overload
 * @param {string} name
 * @returns {string}
 */
/**
 * Greeting
 *
 * @overload
 * @param {{ name: string }} args
 * @returns {string}
 */
/**
 * @param {...*} __lgl_args
 * @returns {string}
 */
export function hello(...__lgl_args) {
  const [name] = /** @type {[name: string]} */ (normalizeMessageArgs(__lgl_args, ["name"]));
  return "Hello " + String(name);
}

const lgl = {
  hello,
  section,
};

export default lgl;
//# sourceMappingURL=en.js.map
