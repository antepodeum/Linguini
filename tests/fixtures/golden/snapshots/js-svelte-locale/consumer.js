import * as context from "./context/svelte-locale.svelte.js";
import * as standalone from "./standalone/svelte-locale.svelte.js";
import * as kit from "./sveltekit/svelte-locale.svelte.js";

for (const state of [context, standalone, kit]) {
  /** @type {"en" | "fr"} */ const locale = state.getCurrentLocale();
  state.setCurrentLocale(locale);
  state.prepareLocale("fr-CA").then((prepared) => state.setCurrentLocale(prepared));
  state.registerLocaleLoader(async (next) => { state.setCurrentLocale(next); })();
  if (false) {
    // @ts-expect-error loader argument must accept supported locale strings
    state.registerLocaleLoader((value) => value.toFixed());
    // @ts-expect-error loaders cannot return numbers
    state.registerLocaleLoader(() => 42);
  }
}
standalone.initializeCurrentLocale("en");
kit.initializeCurrentLocale("fr");
kit.clearCurrentLocaleOverride();
if (false) {
  // @ts-expect-error context-only state has no browser initialization
  context.initializeCurrentLocale("en");
  // @ts-expect-error only SvelteKit has request-data overrides
  standalone.clearCurrentLocaleOverride();
}
