// Test-only policy selection: exercise closed generated modules with the existing runtime corpus.
import * as defaultWeb from "../../../tests/fixtures/golden/snapshots/js-web/default/typescript/web";
import * as allWeb from "../../../tests/fixtures/golden/snapshots/js-web/all/typescript/web";
import * as pathWeb from "../../../tests/fixtures/golden/snapshots/js-web/path/typescript/web";
import * as cookieWeb from "../../../tests/fixtures/golden/snapshots/js-web/cookie/typescript/web";
import * as storageWeb from "../../../tests/fixtures/golden/snapshots/js-web/storage/typescript/web";
import * as languageWeb from "../../../tests/fixtures/golden/snapshots/js-web/language/typescript/web";
import * as languageCookieWeb from "../../../tests/fixtures/golden/snapshots/js-web/language-cookie/typescript/web";
import * as emptyWeb from "../../../tests/fixtures/golden/snapshots/js-web/empty/typescript/web";
export type { LinguiniRuntime, LinguiniWebOptions } from "../../../tests/fixtures/golden/snapshots/js-web/default/typescript/web";

function select(options?: defaultWeb.LinguiniWebOptions) {
  if (options?.routes?.exclude?.length) return allWeb;
  const sources = options?.locale?.sources?.join(",");
  switch (sources) {
    case "": return emptyWeb;
    case "path": return pathWeb;
    case "cookie": return cookieWeb;
    case "local-storage": return storageWeb;
    case "accept-language": return languageWeb;
    case "accept-language,cookie": return languageCookieWeb;
    default: return defaultWeb;
  }
}

export const createWebI18n: typeof defaultWeb.createWebI18n = (runtime, options, environment) =>
  select(options).createWebI18n(runtime, options, environment);
export const createWebLocaleI18n: typeof defaultWeb.createWebLocaleI18n = (runtime, options, environment) =>
  select(options).createWebLocaleI18n(runtime, options, environment);
