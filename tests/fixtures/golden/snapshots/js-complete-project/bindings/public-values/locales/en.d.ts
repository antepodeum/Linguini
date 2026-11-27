import type { Choice } from "../shared.js";
export type { Choice } from "../shared.js";

/**
 * @param {number | bigint | string} value
 * @returns {string}
 */
declare function __lgl_name_$formatNumber(value: number | bigint | string): string;
/**
 * @param {{ value: number | bigint | string }} args
 * @returns {string}
 */
declare function __lgl_name_$formatNumber(args: { value: number | bigint | string }): string;
export { __lgl_name_$formatNumber as formatNumber };

/**
 * @param {number | bigint | string} value
 * @returns {string}
 */
declare function __lgl_name_$normalizeMessageArgs(value: number | bigint | string): string;
/**
 * @param {{ value: number | bigint | string }} args
 * @returns {string}
 */
declare function __lgl_name_$normalizeMessageArgs(args: { value: number | bigint | string }): string;
export { __lgl_name_$normalizeMessageArgs as normalizeMessageArgs };

/**
 * @param {number | bigint | string} value
 * @returns {string}
 */
export declare function message(value: number | bigint | string): string;
/**
 * @param {{ value: number | bigint | string }} args
 * @returns {string}
 */
export declare function message(args: { value: number | bigint | string }): string;

/**
 * @param {Choice} value
 * @returns {string}
 */
declare function __lgl_name_$String(value: Choice): string;
/**
 * @param {{ value: Choice }} args
 * @returns {string}
 */
declare function __lgl_name_$String(args: { value: Choice }): string;
export { __lgl_name_$String as String };

declare const __lgl_name_$lgl: string;
export { __lgl_name_$lgl as lgl };

declare const lgl: {
  readonly formatNumber: typeof __lgl_name_$formatNumber;
  readonly normalizeMessageArgs: typeof __lgl_name_$normalizeMessageArgs;
  readonly message: typeof message;
  readonly String: typeof __lgl_name_$String;
  readonly lgl: typeof __lgl_name_$lgl;
};

export default lgl;
