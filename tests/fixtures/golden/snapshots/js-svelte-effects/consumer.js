import { web, refreshLinguiniEffects, destroyLinguiniEffects } from "./plain-all/svelte-effects.svelte.js";
/** @type {"en" | "fr"} */ const locale = web.resolveLocaleSync();
web.localizeHref("/account", locale);
refreshLinguiniEffects();
destroyLinguiniEffects();
if (false) {
  // @ts-expect-error configured locales exclude German
  web.localizeHref("/account", "de");
  // @ts-expect-error lifecycle refresh takes no parameters
  refreshLinguiniEffects("fr");
}
