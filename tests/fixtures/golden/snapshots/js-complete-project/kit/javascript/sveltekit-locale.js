import { page } from "$app/state";

/** @returns {unknown} */
export function getRequestLocale() {
  return page.data?.linguini?.locale;
}
//# sourceMappingURL=sveltekit-locale.js.map
