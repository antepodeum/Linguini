# @antepodeum/linguini-sveltekit

TypeScript runtime hooks, locale serialization, and request context for SvelteKit.
Configure the web runtime in application code, then create hooks with `createHandle` and
`createReroute`. Routing and persistence policy does not enter native codegen.

The Vite adapter compiles one initial payload per composed page and locale. Pass
`getPagePayload` to `createHandle` and the root `createLoad`; use `createClientLoad` from
`@antepodeum/linguini-sveltekit/load` in the universal root layout. It prepares incoming
messages before page publication. The hook preloads only the selected payload for
script-enabled clients. Server rendering does not fetch browser message files.

Lazy application imports prepare one additional selected-locale closure before evaluation,
then reuse cached files. Unused exports and messages already present in the initial closure
are excluded. Preparation follows the current page and locale; its runtime is linked only
through page nodes exposing these actions. Unsupported import shapes fail explicitly.
Server message calls use SvelteKit's request event, including in server loads.
Nested actions load only their additional records, reusing guaranteed parent messages and
functions. Transit-only modules add no locale requests. Shared and direct import paths keep
the records they need independently; framework route loaders do not widen ownership.
The build selects deferred navigation support only when additional lazy closures exist;
the public root-load import stays the same. SSR and client builds without that demand use
the base loader without optional callback branches.

See [the integration guide](../../docs/web-sveltekit.md) for complete application setup
and remaining import-shape and platform limitations.
