import type { __lgl_name_String$Choice } from "../../shared.js";
export type { __lgl_name_String$Choice } from "../../shared.js";

export type Choice = __lgl_name_String$Choice;

declare const __lgl_name_$formatNumber: {
  readonly number: {
    /**
     * @param {number | bigint | string} value
     * @returns {string}
     */
    (value: number | bigint | string): string;
    /**
     * @param {{ value: number | bigint | string }} args
     * @returns {string}
     */
    (args: { value: number | bigint | string }): string;
  };
};
export { __lgl_name_$formatNumber as formatNumber };

declare const lgl: {
  readonly formatNumber: typeof __lgl_name_$formatNumber;
};

export default lgl;
