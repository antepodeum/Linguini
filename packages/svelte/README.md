# @antepodeum/linguini-svelte

TypeScript rune state and typed message providers. Svelte compiles the built `.svelte.js`
module for the consuming application. Server locale reads use request context; mutable
client locale state is never used to select a server request locale.

Message-provider `setLocale` ignores earlier changes that finish late. The optional third
`createLinguiniRune` argument accepts `getRequestIdentity` to invalidate preparation after
a request replacement; the SvelteKit wrapper supplies this automatically.
