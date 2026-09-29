import { describe, expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import { persistLocaleCookie as javascriptCookie } from "./web/server-cookie.js";
import { persistLocaleCookie as typescriptCookie } from "./typescript/web/server-cookie.ts";
import { withRuntimeHost } from "../../runtime-host.ts";

describe("server cookie and switch-route target parity", () => {
  test("cookie delegation preserves target, locale and optional input identity", () => {
    for (const persist of [javascriptCookie, typescriptCookie]) {
      const calls: unknown[][] = [];
      const target = new Response(); const input = { origin: "https://app.example", httpOnly: true };
      const web = { setLocaleCookie(...args: unknown[]) { calls.push(args); } } as any;
      expect(persist(web, target, "fr", input)).toBeUndefined();
      expect(calls[0]).toEqual([target, "fr", input]);
      expect(calls[0][0]).toBe(target); expect(calls[0][2]).toBe(input);
      persist(web, target, "en");
      expect(calls[1]).toEqual([target, "en", {}]);
      const failing = { setLocaleCookie() { throw new Error("immutable headers"); } } as any;
      expect(() => persist(failing, target, "en")).toThrow("immutable headers");
    }
  });

  for (const variant of ["cookie", "path-only"] as const) {
    for (const base of ["", "/app"]) {
      for (const writesPath of [false, true]) {
        for (const writesCookie of [false, true]) {
          test(`${variant}: base=${base}, path=${writesPath}, cookie=${writesCookie}`, async () => {
            const run = async (target: "javascript" | "typescript" | "legacy") => {
              const suffix = variant === "cookie" ? "/set" : "";
              const query = variant === "cookie" ? "next" : "return";
              const status = variant === "cookie" ? 303 : 307;
              let source = await readFile(new URL(target === "javascript" ? `${variant}/web/switch-route.js` : target === "typescript" ? `${variant}/typescript/web/switch-route.ts` : "legacy/web.switch-route.runtime.ts", import.meta.url), "utf8");
              if (target === "legacy") {
                source = source.replace("{{SERVER_COOKIE_IMPORT}}", variant === "cookie" ? 'import { persistLocaleCookie } from "./server-cookie.js";' : "")
                  .replace("{{SWITCH_ROUTE_PATH}}", `/locale/{locale}${suffix}`).replace("{{SWITCH_ROUTE_RETURN_QUERY}}", query).replace("{{SWITCH_ROUTE_STATUS}}", String(status))
                  .replace("{{PERSIST_SWITCH_COOKIE}}", variant === "cookie" ? '  if (web.options.locale.switch.writesCookie) { persistLocaleCookie(web, response, locale, { origin: event.url.origin }); }' : "");
              }
              return withRuntimeHost(source, target === "javascript" ? "javascript" : "typescript", { modules: { "./server-cookie.js": { persistLocaleCookie: javascriptCookie } } }, (switchRoute) => {
                let persisted = 0;
                const web = {
                  options: { environment: { base }, locale: { switch: { writesPath, writesCookie } } },
                  matchLocale: (locale: unknown) => locale === "en" || locale === "fr" ? locale : undefined,
                  localizeHref: (href: string, locale: string) => `/${locale}${href}`,
                  setLocaleCookie(response: Response, locale: string, input: { origin: string }) { persisted++; expect(input.origin).toBe("https://app.example"); response.headers.append("set-cookie", `locale=${locale}`); },
                };
                const event = (path: string, target?: string, referer?: string) => {
                  const url = new URL(`https://app.example${base}${path}`);
                  if (target !== undefined) url.searchParams.set(query, target);
                  return { url, request: new Request(url, { headers: referer ? { referer } : {} }) };
                };
                const result: unknown[] = [];
                for (const path of ["/other", `/locale/de${suffix}`, `/locale/%zz${suffix}`, `/locale/fr/extra${suffix}`, `/locale/${suffix}`]) {
                  expect(switchRoute.handleLocaleSwitchRoute(web, event(path))).toBeUndefined();
                }
                const cases = [
                  ["/account?tab=1#top", "/account?tab=1#top"], ["https://app.example/account", "/account"],
                  ["https://evil.example/account", undefined], ["//evil.example/account", undefined],
                  ["\\\\evil.example/account", undefined], ["/%2f%2fevil.example", undefined],
                  ["%2f%2fevil.example", undefined], ["%252f%252fevil.example", undefined],
                  ["/%5cevil.example", undefined], ["%zz", undefined], ["javascript:alert(1)", undefined],
                ] as const;
                for (const [returnTarget, safe] of cases) {
                  const response = switchRoute.handleLocaleSwitchRoute(web, event(`/locale/fr${suffix}`, returnTarget)) as Response;
                  const fallback = base || "/";
                  const expected = writesPath ? `/fr${safe ?? fallback}` : safe ?? fallback;
                  expect(response.status).toBe(status); expect(response.headers.get("location")).toBe(expected);
                  expect(response.headers.get("set-cookie")).toBe(variant === "cookie" && writesCookie ? "locale=fr" : null);
                  result.push([response.status, response.headers.get("location"), response.headers.get("set-cookie")]);
                }
                const referer = switchRoute.handleLocaleSwitchRoute(web, event(`/locale/en${suffix}`, "//evil.example", "https://app.example/previous")) as Response;
                expect(referer.headers.get("location")).toBe(writesPath ? "/en/previous" : "/previous");
                const foreign = switchRoute.handleLocaleSwitchRoute(web, event(`/locale/en${suffix}`, undefined, "https://evil.example/previous")) as Response;
                expect(foreign.headers.get("location")).toBe(writesPath ? `/en${base || "/"}` : base || "/");
                expect(persisted).toBe(variant === "cookie" && writesCookie ? cases.length + 2 : 0);
                return result;
              });
            };
            const legacy = await run("legacy");
            expect(await run("typescript")).toEqual(legacy);
            expect(await run("javascript")).toEqual(legacy);
          });
        }
      }
    }
  }
});
