import { matchesRoute } from "./web/routes.js";

const url = new URL("https://example.test/api/orders");
if (!matchesRoute("/api/**", url)) throw new Error("recursive route did not match");
if (matchesRoute("/apiculture/**", url)) throw new Error("unexpected recursive match");
if (!matchesRoute((candidate) => candidate.pathname === "/api/orders", url)) {
  throw new Error("predicate route did not match");
}

if (false) {
  // @ts-expect-error route pattern must be a string, regex, or predicate
  matchesRoute(42, url);
  // @ts-expect-error route input must be a URL
  matchesRoute("/api/**", "/api/orders");
  // @ts-expect-error predicate receives a URL
  matchesRoute((candidate) => candidate.unknown, url);
}
