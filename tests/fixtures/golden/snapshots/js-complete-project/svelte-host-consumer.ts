import { setLocale as setKitLocale } from "./kit/javascript/svelte-control.js";

declare global {
  namespace App {
    interface PageState { tab?: string }
  }
}

setKitLocale("fr", { state: { tab: "account" }, noScroll: true });
// @ts-expect-error The framework's navigation-state type remains exact.
setKitLocale("fr", { state: { tab: 42 } });
// @ts-expect-error Unknown navigation-state properties are rejected.
setKitLocale("fr", { state: { missing: true } });
// @ts-expect-error Locale-switch options retain their types.
setKitLocale("fr", { noScroll: "yes" });
