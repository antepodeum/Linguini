/**
 * @template {string} Locale
 * @param {Record<string, unknown>} input
 * @param {{localStorage: {key: string}}} options
 * @param {(value: unknown) => Locale | undefined} matchLocale
 * @returns {Locale | undefined}
 */
export function resolveLocalStorageLocale(input, options, matchLocale) {
  try {
    return matchLocale(
      (/** @type {Storage | undefined} */ (input.localStorage))?.getItem(options.localStorage.key) ?? undefined,
    );
  } catch {
    return undefined;
  }
}
//# sourceMappingURL=local-storage.js.map
