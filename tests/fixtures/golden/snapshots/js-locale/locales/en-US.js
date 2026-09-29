import { account } from "./en-US/account.js";
import { normalizeMessageArgs } from "../shared.js";

export { account };

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
  const [name] = normalizeMessageArgs(__lgl_args, ["name"]);
  return "Howdy " + String(name);
}

const lgl = {
  root,
  account,
};

export default lgl;
//# sourceMappingURL=en-US.js.map
