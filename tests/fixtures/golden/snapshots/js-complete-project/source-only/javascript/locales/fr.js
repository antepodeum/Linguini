import { section } from "./fr/section.js";
import { normalizeMessageArgs } from "../shared.js";

export { section };

const __lgl_hello_keys = ["name"];
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
  const [name] = /** @type {[name: string]} */ (normalizeMessageArgs(__lgl_args, __lgl_hello_keys));
  return "Bonjour " + String(name);
}

const lgl = {
  hello,
  section,
};

export default lgl;
//# sourceMappingURL=fr.js.map
