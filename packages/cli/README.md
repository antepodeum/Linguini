# @antepodeum/linguini

Install the Linguini compiler, CLI, and native language server without Cargo:

```sh
npm install --save-dev @antepodeum/linguini
npx linguini check
```

The launcher selects one exact optional platform package, verifies its SHA-256
manifest, and executes the bundled native binary. It never downloads code during
installation or first launch.

Web and framework runtime policy is configured in application code. `linguini build` compiles
translation sources; build integrations select thin Svelte project bindings with `--svelte`
and an optional `--svelte-host-module`.
