import { createWebLocaleI18n } from "./web.js";
import { persistLocaleCookie } from "./web/server-cookie.js";
import { handleLocaleSwitchRoute } from "./cookie/web/switch-route.js";

const web = createWebLocaleI18n(/** @type {const} */ ({ locales: ["en", "fr"], baseLocale: "en" }));
persistLocaleCookie(web, new Response(), "fr", { origin: "https://app.example" });
const response = handleLocaleSwitchRoute(web, { url: new URL("https://app.example/locale/fr/set"), request: new Request("https://app.example/locale/fr/set") });
response?.headers.get("location");
if (false) {
  // @ts-expect-error cookie locale remains narrowed to generated locales
  persistLocaleCookie(web, {}, "de");
  // @ts-expect-error switch events require a real URL
  handleLocaleSwitchRoute(web, { url: "/locale/fr/set", request: new Request("https://app.example") });
}
