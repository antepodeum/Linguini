export declare const handle: import("@sveltejs/kit").Handle;
export declare const reroute: import("@sveltejs/kit").Reroute;
export declare const load: import("@sveltejs/kit").ServerLoad;

declare global {
  namespace App {
    interface Locals {
      linguini: import("@antepod/linguini-web").LinguiniRequestContext<import("./index").Locale, import("./index").Linguini>;
      locale: import("./index").Locale;
      direction: import("./index").TextDirection;
      l: import("./index").Linguini;
    }
    interface PageData {
      linguini?: import("@antepod/linguini-sveltekit/server").SerializedLinguiniContext<import("./index").Locale>;
    }
  }
}
