import type { __lgl_name_String$Choice } from "../../shared.js";
export type { __lgl_name_String$Choice } from "../../shared.js";

export type Choice = __lgl_name_String$Choice;

declare const __lgl_name_$normalizeMessageArgs: {
  readonly hello: {
    /**
     * @param {string} name
     * @returns {string}
     */
    (name: string): string;
    /**
     * @param {{ name: string }} args
     * @returns {string}
     */
    (args: { name: string }): string;
  };
};
export { __lgl_name_$normalizeMessageArgs as normalizeMessageArgs };

declare const lgl: {
  readonly normalizeMessageArgs: typeof __lgl_name_$normalizeMessageArgs;
};

export default lgl;
