import { localizeTransformedHref } from "./web/link-transform.js";

if (localizeTransformedHref("/shop") !== "/de/shop") {
  throw new Error("transformed link was not localized");
}
if (localizeTransformedHref("/shop", { ignored: true }) !== "/shop") {
  throw new Error("ignored transformed link was localized");
}

if (false) {
  // @ts-expect-error href must be a string
  localizeTransformedHref(4);
  // @ts-expect-error ignored must be a boolean
  localizeTransformedHref("/shop", { ignored: "yes" });
  // @ts-expect-error input must be a record
  localizeTransformedHref("/shop", {}, "input");
}
