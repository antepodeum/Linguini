# @antepodeum/linguini-sveltekit

TypeScript runtime hooks, locale serialization, and request context for SvelteKit.
Configure the web runtime in application code, then create hooks with `createHandle` and
`createReroute`. Routing and persistence policy does not enter native codegen.

The Vite adapter compiles one initial payload per composed page and locale. Pass
`getPagePayload` to `createHandle` and the root `createLoad`; use `createClientLoad` from
`@antepodeum/linguini-sveltekit/load` in the universal root layout. It prepares incoming
messages before page publication. The hook preloads only the selected payload for
script-enabled clients. Server rendering does not fetch browser message files.

See [the integration guide](../../docs/web-sveltekit.md) for complete application setup
and the remediation checklist for deferred boundaries and remaining verification.
