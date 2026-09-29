import { startRuntimeLinkLocalization } from "./web/runtime-links.js";

/** @type {import("./web.js").LinguiniWebLocale<"en" | "de">} */
const web = {
  shouldLocalizeLink: (_href, attributes) => !attributes?.ignored,
  localizeHref: (href, locale) => `/${locale}${href}`,
};
const effects = startRuntimeLinkLocalization(web, () => "en");
effects?.refresh();
effects?.destroy();

if (false) {
  // @ts-expect-error required web methods must exist
  startRuntimeLinkLocalization({}, () => "en");
  // @ts-expect-error locale must be a string
  startRuntimeLinkLocalization(web, () => 4);
  // @ts-expect-error effects expose only owned lifecycle methods
  effects?.restart();
}
