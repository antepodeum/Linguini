# Code generation

- `crates/linguini-codegen`: backend-independent type models, formatter/plural/inline-function
  requirement analysis, and transitive message dependency slicing with source provenance.
- `crates/linguini-codegen-ecmascript`: JavaScript and TypeScript target syntax, identifier
  escaping, runtime templates, framework adapters, module artifacts, and source maps. This
  backend consumes the shared code-generation plans.
