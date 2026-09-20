import type { Tone } from "./shared.js";

export type LinguiniMessages = {
  readonly hello: {
    /**
     * @param {Tone} tone
     * @returns {string}
     */
    (tone: Tone): string;
    /**
     * @param {{ tone: Tone }} args
     * @returns {string}
     */
    (args: { tone: Tone }): string;
  };
};
