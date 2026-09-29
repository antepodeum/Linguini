import { account } from "./en-US/account";
import { normalizeMessageArgs } from "../shared";

export { account };

/**
 * @param {string} name
 * @returns {string}
 */
export function root(name: string): string;
/**
 * @param {{ name: string }} args
 * @returns {string}
 */
export function root(args: { name: string }): string;
export function root(...__lgl_args: [name: string] | [args: { name: string }]): string {
  const [name] = normalizeMessageArgs(__lgl_args, ["name"]) as [string];
  return "Howdy " + String(name);
}

const lgl = {
  root,
  account,
} as const;

export default lgl;
