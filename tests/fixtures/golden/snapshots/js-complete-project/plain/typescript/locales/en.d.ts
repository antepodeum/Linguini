import { section } from "./en/section";

export { section };

/**
 * Greeting
 *
 * @param {string} name
 * @returns {string}
 */
export declare function hello(name: string): string;
/**
 * Greeting
 *
 * @param {{ name: string }} args
 * @returns {string}
 */
export declare function hello(args: { name: string }): string;

declare const lgl: {
  readonly hello: typeof hello;
  readonly section: typeof section;
};

export default lgl;
