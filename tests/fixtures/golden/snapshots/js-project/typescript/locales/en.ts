import { normalizeMessageArgs } from "../shared";

/**
 * @param {string} name
 * @returns {string}
 */
export function hello(name: string): string;
/**
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
} as const;

export default lgl;
