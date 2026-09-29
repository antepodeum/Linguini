import { normalizeMessageArgs } from "../shared.js";

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
  const [name] = normalizeMessageArgs(__lgl_args, ["name"]);
  return "Hello " + String(name);
}

const lgl = {
  hello,
};

export default lgl;
//# sourceMappingURL=en.js.map
