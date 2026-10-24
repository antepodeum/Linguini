import { normalizeMessageArgs } from "../shared.js";

const __lgl_hello_keys = ["name"];
/**
 * @overload
 * @param {string} name
 * @returns {string}
 */
/**
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
  return "Hello " + String(name);
}

const lgl = {
  hello,
};

export default lgl;
//# sourceMappingURL=en.js.map
