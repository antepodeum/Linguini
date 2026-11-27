import type { Date, Input } from "./shared.js";

export type LinguiniMessages = {
  readonly root: {
    /**
     * @param {Input} value
     * @param {globalThis.Date | number | string} direct
     * @returns {string}
     */
    (value: Input, direct: globalThis.Date | number | string): string;
    /**
     * @param {{ value: Input; direct: globalThis.Date | number | string }} args
     * @returns {string}
     */
    (args: { value: Input; direct: globalThis.Date | number | string }): string;
  };
};
