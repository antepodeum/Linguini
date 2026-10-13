export type LinguiniMessages = {
  readonly hello: {
    /**
     * Greeting
     *
     * @param {string} name
     * @returns {string}
     */
    (name: string): string;
    /**
     * Greeting
     *
     * @param {{ name: string }} args
     * @returns {string}
     */
    (args: { name: string }): string;
  };
  readonly section: {
    readonly title: string;
  };
};
//# sourceMappingURL=messages.ts.map
