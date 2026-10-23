# @antepodeum/linguini-tsc

Native Linguini generation for ordinary TypeScript builds, incremental project references,
and watch mode. Generated JavaScript has adjacent declarations; `allowJs` is unnecessary.

```sh
pnpm add -D @antepodeum/linguini-tsc typescript
pnpm exec linguini-tsc --project tsconfig.json
pnpm exec linguini-tsc --build --watch
```

Install the native Linguini CLI separately. See the [TypeScript integration guide](https://github.com/antepodeum/Linguini/blob/main/docs/typescript.md)
for output layout, declaration-only builds, configuration, and the programmatic API.

Node 20.6 or newer is required. Configure web/framework capabilities in `linguini.config.mjs`
with typed factories from `@antepodeum/linguini-tsc/modules`. Both watch modes track imported
configuration files and preserve the last successful runtime after configuration errors.
