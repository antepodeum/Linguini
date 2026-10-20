import { goto } from "$app/navigation";

/** @param {string} href @param {Record<string, unknown>} options @returns {Promise<void>} */
export function navigate(href, options) {
  return goto(href, {
    replaceState: Boolean(options.replaceState),
    invalidateAll: Boolean(options.invalidateAll),
    keepFocus: /** @type {boolean | undefined} */ (options.keepFocus),
    noScroll: /** @type {boolean | undefined} */ (options.noScroll),
    state: /** @type {App.PageState | undefined} */ (options.state),
  });
}
//# sourceMappingURL=sveltekit-navigation.js.map
