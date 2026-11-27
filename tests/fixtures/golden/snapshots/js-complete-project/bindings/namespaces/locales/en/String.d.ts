import type { __lgl_name_String$Choice } from "../../shared.js";
export type { __lgl_name_String$Choice } from "../../shared.js";

export type Choice = __lgl_name_String$Choice;

declare const __lgl_name_$String: {
  readonly number: {
    /**
     * @param {__lgl_name_String$Choice} value
     * @returns {string}
     */
    (value: __lgl_name_String$Choice): string;
    /**
     * @param {{ value: __lgl_name_String$Choice }} args
     * @returns {string}
     */
    (args: { value: __lgl_name_String$Choice }): string;
  };
};
export { __lgl_name_$String as String };

declare const lgl: {
  readonly String: typeof __lgl_name_$String;
};

export default lgl;
