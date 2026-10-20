# TypeScript build integration

Build Linguini messages with an ordinary TypeScript application, without enabling `allowJs`.
Install the native Linguini CLI separately and make `linguini` available on `PATH`.

```sh
pnpm add -D @antepod/linguini-tsc typescript
pnpm exec linguini-tsc --project tsconfig.json
pnpm exec linguini-tsc --watch
pnpm exec linguini-tsc --build
pnpm exec linguini-tsc --build --watch
```

The adapter uses TypeScript's public compiler, incremental, solution-builder, and watch APIs.
The supported compiler range is TypeScript 5.8 through 6.x.
It generates source-adjacent JavaScript and declarations before type checking. A successful
application build also generates the same runtime into the corresponding `outDir` location,
using the native compiler's `--emit-dir` support and rebased source maps. There is one JavaScript
implementation with adjacent `.d.ts` files; TypeScript source generation is unnecessary.

Use an ESM TypeScript project. For example:

```json
{
  "compilerOptions": {
    "strict": true,
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "target": "ES2022",
    "rootDir": "src",
    "outDir": "dist"
  },
  "include": ["src/**/*.ts"]
}
```

With Linguini output configured as `src/generated/linguini`, runtime output is written to
`dist/generated/linguini`. Keep declarations enabled in the Linguini target. Import generated
modules with `.js` extensions in application source. The adapter reads `extends` and project
references through TypeScript and finds each project's nearest `linguini.toml`.

`--noEmit` prepares source types without writing application output. `--emitDeclarationOnly`
also delivers generated declarations and declaration maps beside the application declarations.
A separate `declarationDir` receives generated types without JavaScript. These modes do not
deliver a new runtime to `outDir`. Without an `outDir`, the generated source runtime is already adjacent to the emitted
application. Output remains inside the native Linguini project root and follows its ownership
rules. `--build --clean` and `--dry` delegate to TypeScript without generating messages; TypeScript
cleaning removes its own outputs. Native owned output can be rebuilt independently.

Watch mode observes schema/locale files and configuration separately from TypeScript inputs.
Message text edits refresh runtime output even when declarations stay unchanged and incremental
TypeScript projects remain up to date. Native failures retain the last valid generated output;
later edits can recover without restarting watch mode.

Set `LINGUINI_COMMAND` to select a native executable for the command-line adapter. Programmatic
integrations can import `runTypeScript`, supply `command`, `nativeArgs`, or diagnostic reporters,
and call the returned `close()` to dispose watch resources.
