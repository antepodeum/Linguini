export type LinguiniMessages = {
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
