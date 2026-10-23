# @antepodeum/linguini

Install the Linguini compiler, CLI, and native language server without Cargo:

```sh
npm install --save-dev @antepodeum/linguini
npx linguini check
```

The launcher selects one exact optional platform package, verifies its SHA-256
manifest, and executes the bundled native binary. It never downloads code during
installation or first launch.

With Node 20.6 or newer, `linguini build` loads project-root `linguini.config.mjs`. Import typed
`defineModules`, `web`, `svelte`, and `svelteKit` factories from `@antepodeum/linguini/modules`.
The launcher passes JSON to native codegen; Rust never evaluates the configuration as code.
Explicit `--module-plan FILE` and `--module-plan-stdin` inputs bypass configuration loading.
