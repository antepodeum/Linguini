# @linguini/cli

Install the Linguini compiler, CLI, and native language server without Cargo:

```sh
npm install --save-dev @linguini/cli
npx linguini check
```

The launcher selects one exact optional platform package, verifies its SHA-256
manifest, and executes the bundled native binary. It never downloads code during
installation or first launch.
