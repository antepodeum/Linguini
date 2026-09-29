import { resolvePathLocale } from "./web/path.js";

const locales = /** @type {const} */ (["en", "de", "fr"]);
const options = { environment: { base: "/shop" } };
if (resolvePathLocale({ url: "https://example.test/shop/de/orders" }, options, locales) !== "de") {
  throw new Error("path locale did not resolve");
}

if (false) {
  // @ts-expect-error input must be a record
  resolvePathLocale("/shop/de", options, locales);
  // @ts-expect-error base must be a string
  resolvePathLocale({ url: "/shop/de" }, { environment: { base: 4 } }, locales);
  // @ts-expect-error locales must be strings
  resolvePathLocale({ url: "/shop/de" }, options, [4]);
}
