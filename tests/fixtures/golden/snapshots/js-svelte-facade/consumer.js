import * as context from "./context/svelte.js";
import * as web from "./web/svelte.js";

context.l.hello("Ada");
web.messages.hello({ name: "Ada" });
context.setLocale("fr");
web.setLocale("fr", { navigate: false, state: { tab: "account" } });
web.localizeHref("/account", "fr");
web.linguini.destroy();
/** @type {"en" | "fr"} */ const locale = context.linguini.locale;
if (false) {
  // @ts-expect-error messages preserve named-object overload requirements
  web.l.hello({ first: "Ada" });
  // @ts-expect-error context-only facade has no web methods
  context.localizeHref("/account");
  // @ts-expect-error generated locale union remains closed
  web.localizeHref("/account", "de");
  // @ts-expect-error context locale switching requires a string
  context.setLocale(42);
}
