import { section } from "./en/section.js";
import { normalizeMessageArgs } from "../shared.js";

export { section };

const __lgl_keys_hello = ["name"];
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
  const [name] = /** @type {[name: string]} */ (normalizeMessageArgs(__lgl_args, __lgl_keys_hello));
  return "Hello " + String(name);
}

const lgl = {
  hello,
  section,
};

export default lgl;
//# sourceMappingURL=en.js.map
