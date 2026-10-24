import { account } from "./en-US/account.js";
import { normalizeMessageArgs } from "../shared.js";

export { account };

const __lgl_root_keys = ["name"];
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
export function root(...__lgl_args) {
  const [name] = /** @type {[name: string]} */ (normalizeMessageArgs(__lgl_args, __lgl_root_keys));
  return "Howdy " + String(name);
}

const lgl = {
  root,
  account,
};

export default lgl;
//# sourceMappingURL=en-US.js.map
