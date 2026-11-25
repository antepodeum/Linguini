# @antepodeum/linguini-svelte

TypeScript rune state and typed message providers. Svelte compiles the built `.svelte.js`
module for the consuming application. Server locale reads use request context; mutable
client locale state is never used to select a server request locale.

Message-provider `setLocale` ignores earlier changes that finish late. The optional third
`createLinguiniRune` argument accepts `getRequestIdentity` to invalidate preparation after
a request replacement; the SvelteKit wrapper supplies this automatically.

`createLocaleSetter(state, options?)` prepares and publishes locale changes without a
message provider. Generated bindings create it once in the locale helper and re-export
it directly; the message provider receives that same function through its `setLocale`
option. With the Svelte Vite adapter, importing only this setter retains no catalogs.

The `@antepodeum/linguini-svelte/vite` entry composes the shared Vite plugin with Svelte
script scopes, static-link rewriting, and component cleanup. Install
`@antepodeum/linguini-vite` when using this entry. Vite remains an optional peer for
applications that only consume the locale runtime. SvelteKit composes this same adapter
through `@antepodeum/linguini-sveltekit/vite`.
