import { section } from "./en/section";
import { normalizeMessageArgs } from "../shared";

export { section };

/**
 * Greeting
 *
 * @param {string} name
 * @returns {string}
 */
export function hello(name: string): string;
/**
 * Greeting
 *
 * @param {{ name: string }} args
 * @returns {string}
 */
export function hello(args: { name: string }): string;
export function hello(...__lgl_args: [name: string] | [args: { name: string }]): string {
  const [name] = normalizeMessageArgs(__lgl_args, ["name"]) as [string];
  return "Hello " + String(name);
}

const lgl = {
  hello,
  section,
} as const;

export default lgl;
//# sourceMappingURL=en.ts.map
