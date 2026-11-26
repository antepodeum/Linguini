# @antepodeum/linguini-web

TypeScript runtime adapters for locale resolution, URL routing, and cookie persistence.
The package is independent of native codegen and does not import a generated message catalog.
Applications configure resolver capabilities in their own modules and hooks. Source is maintained
in TypeScript; npm contains compiled ESM JavaScript, declarations, and source maps.

Routing/metadata objects hold normalized policy without cookie-writing methods. Import
`serializeLocaleCookie` from the package for `serializeLocaleCookie(web, locale, input)`, or
`persistLocaleCookie` from `@antepodeum/linguini-web/server` for
`persistLocaleCookie(web, target, locale, input)`. Client controls and framework adapters compose
these operations; routing-only bundles can omit their implementations.

Client controls ignore superseded changes and invalidate pending changes on destruction.
An optional `getRequestIdentity` callback also invalidates changes prepared for an older
request; the SvelteKit adapter supplies this capability automatically.
