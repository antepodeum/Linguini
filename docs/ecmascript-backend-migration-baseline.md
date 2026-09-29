# ECMAScript backend migration baseline

This inventory freezes the production ownership boundary before the remaining direct TypeScript
assembly is replaced. Path families include every option-gated instance of that family. The
migration must preserve these paths and contracts unless a separately reviewed public change says
otherwise.

## Generated artifact ownership

| Stage | Implementation artifacts | Declaration companions |
| --- | --- | --- |
| Schema | `shared.ts`, `messages.ts` | `shared.d.ts`, `messages.d.ts` |
| Locale runtime | `locales/<locale>/_runtime.ts` | none |
| Locale semantics | optional aggregate `locales/<locale>/_globals.ts`; bundler mode also owns one encoded leaf per required enum, variable, form, or function | types are re-exported through locale declarations |
| Locale messages | `locales/<locale>/<namespace>.ts`, `locales/<locale>.ts` | matching namespace and locale `.d.ts` files |
| Project runtime | `locale.ts`, `index.ts` | `locale.d.ts`, `index.d.ts` |
| Web leaves | optional `web.ts`, `web/{path,cookie,local-storage,accept-language,routes,link-transform,runtime-links,server-cookie,switch-route}.ts`; `web/path`, `web/cookie`, and `web/routes` now have checked-JavaScript compilers | matching `.d.ts` files |
| Svelte | optional `svelte-locale.svelte.ts`, `svelte-effects.svelte.ts`, `svelte-control.ts`, `svelte.ts` | matching `.d.ts` files |
| SvelteKit | optional `sveltekit-control.ts`, `sveltekit.ts` | `sveltekit-control.d.ts`, ambient `linguini-app.d.ts` |
| Bundler leaves | `bundler/messages/<encoded-message>/<encoded-locale>.ts`, `bundler/semantic/<locale>/<kind>/<encoded-name>.ts`, and one shared `locales/<locale>/_runtime.ts` | JavaScript/JSDoc compilation changes leaf extensions to `.js`; no separate declaration companion yet |
| Metadata | optional `.gitignore`, `bundler/manifest.json`, `.linguini-generated-manifest` | none |

## Import-edge baseline

- `shared` and `messages` are schema roots. `messages` has one type-only edge to `shared`.
- A locale runtime is self-contained. Locale globals import exact schema types/helpers from
  `shared` and demand-selected formatter/plural helpers from their locale runtime.
- Namespace modules import exact schema types/helpers from `shared`, exact runtime helpers from
  `_runtime`, and exact global semantic bindings from `_globals`. Locale barrels additionally
  import their namespace modules and export one default locale object.
- `index` imports the base locale eagerly, owns finite loaders for the remaining locale barrels,
  imports locale metadata from `locale`, and re-exports schema/message types.
- Web source leaves are self-contained except for link/server/switch leaves, which import the
  public web facade and, where required, Svelte locale/effect state. The web facade owns the
  selected source/route edges only.
- Svelte modules import project `locale`/`index` state plus only enabled web controls. SvelteKit
  modules additionally import the selected `$app/*` and `@sveltejs/kit` capabilities.
- A physical bundler message imports only its direct semantic leaves, exact shared types/helpers,
  and exact locale-runtime helpers. A semantic leaf imports only its direct semantic dependencies,
  exact shared types/helpers, and exact locale-runtime helpers. JavaScript leaves use `.js`
  specifiers and omit type-only edges.

## Paths, maps, and manifests

- Project paths are validated for case-folded uniqueness and portable components before output.
  Bundler message and semantic paths use deterministic encoded components and a 240-byte bound.
- Every physical message and semantic leaf owns an adjacent `<module>.map`. Public locale-runtime,
  locale-global, namespace, locale-barrel, and project-entry artifact compilers also return
  target-owned maps. Trailers name the map basename. Locale artifact `sources`/`sourcesContent`
  come from ordered source records and mappings point at semantic source spans; generated-only
  `locale`/`index` entry maps have no source records. The existing project-file generator still
  emits those entries without maps; output ownership moves in ESM-M9. The generated-only
  `web/path`, `web/cookie`, and `web/routes` artifact compilers also return target-owned maps.
- `bundler/manifest.json` version 1 owns `base_locale`, `configured_locales`,
  `effective_locales`, `sources`, and `messages`; bundler mode also owns `locale_loading`,
  `applications`, `message_runtimes`, `message_semantics`, and `runtime_helpers`.
- `.linguini-generated-manifest` is the atomic output ownership list. Migration stages may not
  bypass its collision, stale-file cleanup, or rollback rules.

## Public Rust boundary

- `ValidatedTypeScriptProject::try_new` is the production validation gate.
- `generate_typescript_project_files` owns project artifact enumeration.
- `message_artifacts`, `semantic_artifacts`, `locale_runtime_artifacts`,
  `locale_globals_artifacts`, `locale_artifacts`, and `project_artifacts` own deterministic physical
  metadata.
- The TypeScript and JavaScript message/semantic compile functions own exact leaf rendering.
- The TypeScript and JavaScript locale-runtime and locale-global artifact compilers own shared
  target-aware runtime and aggregate-global rendering.
- The TypeScript and JavaScript locale artifact compilers own fallback-composed namespace and
  barrel rendering, including recursive namespace/default exports.
- The TypeScript and JavaScript project artifact compilers own `locale` metadata, eager/dynamic
  loaders, and public configure/prepare runtime rendering.
- `compile_typescript_web_routes_module` and `compile_javascript_web_routes_module` render the
  selected route-exclusion helper from one target-aware body.
- `compile_typescript_web_path_module` and `compile_javascript_web_path_module` render the
  selected path-locale resolver from one target-aware body.
- `compile_typescript_web_cookie_module` and `compile_javascript_web_cookie_module` render the
  selected cookie-locale resolver from one target-aware body.
- `EcmaModule`, its import/statement/source records, `TypeModel`, and the TypeScript/JSDoc type
  renderers are the common backend surface to extend; target-specific assembly must not create a
  second production runtime.

## Frozen evidence

- Byte snapshots: `tests/fixtures/golden/snapshots/ts`, `ts-runtime`, and `js`.
- Message/semantic JavaScript, JSDoc, dependency, and source-map fixtures:
  `crates/linguini-codegen-ts/src/module/message.rs` and `semantic.rs` tests.
- Locale-global TypeScript/JavaScript byte snapshots, strict JSDoc checks, source maps, and
  executable form/function parity: `tests/fixtures/golden/snapshots/js-globals`.
- Locale namespace/barrel TypeScript/JavaScript byte snapshots, strict JSDoc checks, fallback
  source maps, recursive namespaces, and executable parity:
  `tests/fixtures/golden/snapshots/js-locale`.
- Project-entry TypeScript/JavaScript byte snapshots, strict JSDoc positive/negative checks,
  target-owned source maps, and executable loader/provider parity:
  `tests/fixtures/golden/snapshots/js-project`.
- Web route TypeScript/JavaScript byte snapshots, strict JSDoc, target-owned maps, and exact,
  recursive, predicate, and regex parity: `tests/fixtures/golden/snapshots/js-web-routes`.
- Web path TypeScript/JavaScript byte snapshots, strict JSDoc, target-owned maps, and base-path,
  boundary, invalid-URL, and locale-casing parity: `tests/fixtures/golden/snapshots/js-web-path`.
- Web cookie TypeScript/JavaScript byte snapshots, strict JSDoc, target-owned maps, and cookie
  boundary, decoding, and malformed-value parity: `tests/fixtures/golden/snapshots/js-web-cookie`.
- Runtime behavior: Rust project-codegen tests plus `crates/linguini-codegen-ts/tests/*.test.ts`.
- Manifest, atomic output, and cleanup behavior: `crates/linguini-cli/src/tests.rs` and
  `crates/linguini-cli/src/project/output.rs` tests.
- Consumer integration: Vite plugin tests, real-site builds, npm launcher tests, and VSIX tests.

Each migration stage updates this baseline only when it intentionally changes ownership. Otherwise
the relevant snapshot, typecheck, executable runtime, source-map, manifest, and consumer checks
must remain green before the displaced assembly code is deleted.
