import { createWebI18n, createWebLocaleI18n } from "./all/web.js";
/** @type {import("./all/web.js").LinguiniRuntime<"en" | "fr", {title: string}>} */
const runtime = { locales: ["en", "fr"], baseLocale: "en", createLinguini: locale => ({ title: locale }) };
const web = createWebI18n(runtime);
/** @type {string} */
const title = web.createRequestContext("fr").messages.title;
web.localizeUrl("/shop", "en");
web.shouldLocalizeLink("/shop", { rel: "external", download: true });
createWebLocaleI18n(runtime, { locale: { sources: ["cookie"] }, cookie: { sameSite: "strict" } });
// @ts-expect-error Unknown locale.
web.localizeHref("/shop", "de");
// @ts-expect-error Messages retain their declared shape.
web.createRequestContext("fr").messages.missing;
// @ts-expect-error Source names are closed.
createWebLocaleI18n(runtime, { locale: { sources: ["query"] } });
// @ts-expect-error Flat obsolete policy is rejected.
createWebLocaleI18n(runtime, { prefixDefaultLocale: true });
void title;
