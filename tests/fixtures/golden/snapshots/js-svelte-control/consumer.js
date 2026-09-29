import * as controls from "./kit-all/svelte-control.js";
/** @type {"en" | "fr"} */ const locale = controls.linguini.locale;
controls.setLocale(locale, { navigate: false, cookie: true });
controls.localizeHref("/account", "fr");
controls.localizeUrl(new URL("https://app.example/account"));
controls.shouldLocalizeLink("/file", { download: true });
controls.alternateLinks("/account");
controls.destroy();
if (false) {
  // @ts-expect-error locale switching requires a string
  controls.setLocale(42);
  // @ts-expect-error forwarded locale remains narrowed
  controls.localizeHref("/account", "de");
  // @ts-expect-error link attributes enforce boolean download
  controls.shouldLocalizeLink("/file", { download: "yes" });
  // @ts-expect-error destroy takes no arguments
  controls.destroy("fr");
}
