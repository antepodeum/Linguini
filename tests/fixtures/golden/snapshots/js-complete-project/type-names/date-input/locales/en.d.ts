import type { Date, globalThis, Input } from "../shared.js";
export type { Date, globalThis, Input } from "../shared.js";

/**
 * @param {Input} value
 * @param {globalThis.Date | number | string} direct
 * @returns {string}
 */
export declare function root(value: Input, direct: globalThis.Date | number | string): string;
/**
 * @param {{ value: Input; direct: globalThis.Date | number | string }} args
 * @returns {string}
 */
export declare function root(args: { value: Input; direct: globalThis.Date | number | string }): string;

declare const lgl: {
  readonly root: typeof root;
};

export default lgl;
