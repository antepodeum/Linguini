# Linguini Audit: Issues Across All Crates and the Overall Concept

**Audit date:** July 28, 2026  
**Source:** `Linguini-main(1).zip`  
**Audit type:** static code, architecture, and security review plus the available JavaScript tests

## Executive summary

The audit recorded **369 distinct findings**: **22 critical**, **166 high**, **162 medium**, and **19 low** severity. This is the most complete issue inventory found during this static pass; “all” is not a mathematical guarantee that no additional defects exist, especially because the Rust test suite, fuzzing, and a generated-runtime corpus could not be executed.

The project is currently suitable as a research preview, but it is **not ready for safe production adoption**. The main blockers are destructive filesystem operations, network and `git` execution inside a procedural macro, broken form code generation, loss of multi-key branches, incorrect variable classification, semantic errors that do not block the CLI, and unsafe lexical rename in the LSP.

### Most dangerous blockers

1. The CLDR procedural macro deletes the directory supplied through `LINGUINI_CLDR_SOURCE_CHECKOUT_DIR` and then performs network and `git` operations during compilation (`crates/linguini-cldr-macros/src/source_paths.rs:87-105,191-243`).
2. The CLI may delete or replace an arbitrary output directory configured through `targets.ts.out`, including an absolute path or a path containing `..`, without canonical containment checks (`crates/linguini-cli/src/project/codegen.rs:99`; writer logic at `:198-265`).
3. The TypeScript form emitter drops branch entries but emits calls as though the generated value were a function (`module/expr.rs:14-27,122-175`).
4. Multi-key branch code generation preserves only the first selector (`module/expr.rs:78-90`).
5. The locale loader stores variables in the message map (`crates/linguini-locale/src/lib.rs:116-123`).
6. The CLI routes all file-level analyzer diagnostics to warnings and may allow semantic errors to pass (`crates/linguini-cli/src/project/check.rs:104-113`).
7. The LSP analyzes a locale against every schema and renames token text lexically across the workspace instead of resolving symbols (`crates/linguini-lsp/src/document.rs:167-187,251-317`).
8. `stripLeadingLocale` may falsely recognize a locale prefix in an ordinary route segment.
9. The test helper allows `..` in a temporary project name and later recursively removes the computed path (`crates/linguini-test-support/src/lib.rs:20-34`).
10. Alias cycles can cause stack overflow in code generation and sample generation.

## Method and limitations

- Inventoried 14 Rust crates, the Vite plugin, VS Code extension, SvelteKit site, CI, and documentation.
- Read parser, analyzer, IR, code generation, runtime, CLI, LSP, and path-handling code; searched for panic, unwrap, TODO, and security-sensitive operations.
- `plugins/vite`: **3/3 tests passed** on Node 22.
- The environment did not contain a Rust toolchain, so `cargo test`, Clippy, MSRV checks, and dynamically generated Rust code could not be run.
- `editors/vscode`: `npm ci` did not complete within the available execution window; the later compile attempt was invalid because dependency installation was incomplete. This was not counted as a repository defect.

## Severity scale

- **Critical:** possible data loss, supply-chain compromise, panic or `TypeError` on valid input, or a fundamentally incorrect result.
- **High:** violation of a documented contract, correctness, security, namespace or type semantics, or production reliability.
- **Medium:** substantial technical debt, diagnostic, performance, UX, portability, or edge-case correctness problem.
- **Low:** hygiene issue, coverage gap, or limited inconsistency.

## Cross-cutting conceptual and architectural issues

1. **[Critical]** The specification and implementation diverge: the reference promises inline `fn`, a configurable CJS/ESM target, the complete lint set, strict exhaustiveness, and type errors that are absent or only partially checked in code. CJS should be removed from the current specification and configuration rather than implemented during the SvelteKit-first phase.
2. **[Critical]** There is no enforced “validated IR → codegen” boundary: the public code generator accepts invalid IR and can emit syntactically valid TypeScript that fails at runtime.
3. **[High]** Semantics are spread across syntax, schema, locale, analyzer, IR, CLI, LSP, and codegen; equivalent AST walks, name resolution, fallback, and rendering logic are implemented repeatedly and have already diverged.
4. **[High]** The compile-time type-safety claim is overstated: there is no complete type inference, call-argument type checking, selector type checking, or validation for a large class of references.
5. **[High]** The compile-time exhaustiveness claim is overstated: a wildcard terminates checking regardless of position, multidimensional coverage is incomplete, and unknown variants are often not rejected.
6. **[High]** There is no unified source-identity model: `Span` stores only a range, so cross-file related diagnostics, renames, cycles, and IR errors lose their exact source.
7. **[High]** The namespace model is inconsistent: schema files act as namespaces, but forms, functions, and variables remain global in several pipelines, while the LSP matches one document against every schema.
8. **[High]** Locale fallback is implemented separately in config, CLI, codegen, and runtime as hyphen truncation; it is not CLDR `parentLocales` or canonicalization and produces divergent behavior across layers.
9. **[High]** CLDR formatting is limited to Latin digits and the Gregorian calendar and does not match the broad localization claim; currency-specific digits and spacing, standalone contexts, eras, time zones, and skeletons are absent.
10. **[High]** Builds are unpredictable and non-hermetic: the procedural macro performs network and `git` operations, and the site invokes Cargo during JavaScript checks.
11. **[High]** The web-strategy API is broader than the implementation: config accepts strategies ignored by the runtime, and the default strategy contains an effectively nonfunctional `preferredLanguage` entry.
12. **[Medium]** Public models expose mutable `Vec` and `BTreeMap` values without invariant-preserving constructors; downstream code must repeatedly revalidate uniqueness, order, and valid combinations.
13. **[Medium]** Diagnostics have no stable codes, categories, or source IDs, preventing reliable SARIF/JSON output, suppression, compatible quick fixes, and analytics.
14. **[Medium]** Documentation is not executed as doctests or golden projects; examples and promises have already drifted away from parser, analyzer, and codegen behavior.
15. **[Medium]** The test strategy is mostly unit and snapshot based; property/fuzz tests, multi-platform path tests, generated-TypeScript typecheck/runtime tests, and real editor/plugin integrations are absent.
16. **[Medium]** Versions of Rust crates, the Vite package, the site, and the VS Code extension differ, so component compatibility is not pinned.
17. **[Medium]** The README correctly labels the project a preview while simultaneously presenting guarantees such as “type errors” and “compile-time” as completed properties, which is misleading.

## linguini-core

18. **[Medium]** Public `CRATE_PURPOSE` is a placeholder rather than domain API and pollutes the public surface.
19. **[High]** `FormatterKind::from_name` maps any unknown name to an unnamed `Unknown`; the original name is lost and downstream code silently ignores the error.
20. **[Medium]** Primitive types are hard-coded and duplicated in analyzer, codegen, and CLI; adding a type requires synchronized changes across multiple crates.
21. **[Medium]** The CLI sample generator recognizes `Integer` and `DateTime`, which do not exist in `TypeKind`, already demonstrating model drift.
22. **[Low]** The crate has almost nothing to test beyond enum round trips; contract tests with dependent layers are absent.

## linguini-config

23. **[High]** The custom parser supports only a TOML subset even though the file is called `linguini.toml`; valid multiline strings and arrays, escapes, and complex comments may be parsed incorrectly.
24. **[High]** Two additional independent pseudo-TOML parsers exist in the Vite plugin and CLDR macros; the same format therefore has three different semantics.
25. **[High]** `targets.ts.out` is not validated against absolute paths, `..`, overlap with source roots, or symlink escapes; combined with the CLI, this becomes arbitrary-directory deletion.
26. **[High]** Discovery follows directory symlinks through `Path::is_dir` or filesystem metadata without a visited set, allowing cycles and infinite recursion.
27. **[High]** Locale-tag validation is not BCP 47: numeric regions such as `es-419` are rejected while some duplicate or invalid subtags are accepted.
28. **[Medium]** Duplicate sections and keys are not diagnosed consistently; some values are silently overwritten or ignored.
29. **[Medium]** Duplicate legacy `[paths] cache` configuration is silently ignored instead of producing a migration error.
30. **[Medium]** Empty project names, default locales, and paths are not validated; duplicate locales differing only by case and duplicate strategies are also not rejected.
31. **[Medium]** `globalVariable` does not require `global_variable_name`; config may parse successfully even though the runtime cannot execute the strategy.
32. **[Medium]** `SameSite=None` does not require `Secure`, producing cookies rejected by modern browsers.
33. **[Medium]** `base_path`, origin, cookie path/domain/name/max-age, and message patterns receive little validation.
34. **[Medium]** Non-UTF-8 path components are silently discarded during namespace derivation, creating collisions.
35. **[Medium]** For a file outside the root, `strip_prefix(...).unwrap_or(path)` derives a namespace or scope from an unrelated or absolute path instead of returning an explicit error.
36. **[Medium]** For a path outside the locale root, `locale_scope_chain` may include parent or absolute components and select the wrong files.
37. **[Medium]** Some I/O errors lose their original kind and context, making permission, symlink, or corruption failures difficult to diagnose.
38. **[Low]** Path ordering depends on platform representation; reproducible ordering across Windows and Unix is not defined.
39. **[Medium]** Config accepts strategies and parameters not implemented by the generated runtime, so a successful parse does not imply an executable configuration.

## linguini-syntax

40. **[High]** The parser does not implement the documented inline `fn` inside an interpolation.
41. **[High]** AST `Expression` represents both `foo` and `foo()` identically (`arguments = []`), so later stages cannot distinguish a reference from a zero-argument call.
42. **[High]** Zero-argument function and form declarations are not supported consistently: a bare form and `foo()` behave differently.
43. **[High]** `form` and `fn` lowering and AST both use one `FunctionDeclaration`, losing the original distinction and making the `fn_without_strings` lint impossible to compute reliably.
44. **[High]** The lexer changes mode after every `=`, making the grammar fragile under recovery and future language extensions.
45. **[High]** Triple-quoted multiline messages are only partially implemented: the lexer emits `TripleQuote` and newline tokens, but the locale parser has no `TripleQuote` production and the shared `strip_trivia` pass removes newlines before parsing. Multiline content therefore cannot retain its structure through the parser, while existing unquoted raw-text edge trimming can also remove semantically significant whitespace.
46. **[Medium]** Keywords remain ordinary identifiers; there is no reserved-name policy.
47. **[Medium]** Naming conventions such as PascalCase types and lowercase messages or variants are not checked.
48. **[Medium]** The parser accepts empty enums, duplicate enum variants, and duplicate parameters; errors are deferred or lost.
49. **[Medium]** String literals preserve the written escape sequences instead of decoded values, complicating equivalence and code generation.
50. **[Medium]** There is no unambiguous escape mechanism for literal `{` and `}` in raw message text.
51. **[Medium]** Locale-tag tokenization is inconsistent with the config validator.
52. **[Medium]** `parse_locale` and `parse_schema` use `expect` for internal invariants and may panic after an unexpected parser-recovery state.
53. **[Medium]** The public mutable AST does not preserve grammar invariants after manual construction.
54. **[Medium]** `Span` lacks a source or file ID; cross-file precision is impossible without external tables.
55. **[Medium]** The override node loses part of the metadata and span of the original `override`, degrading diagnostics and formatting.
56. **[Medium]** Recovery discards error tokens and can “stitch” a construct across invalid bytes, producing a false AST.
57. **[Medium]** Doc-comment attachment depends on whitespace and recovery and can shift to the wrong declaration.
58. **[Medium]** Deep property paths are syntactically accepted without a model of the valid shape, while downstream code checks only the first segments.
59. **[Low]** Lexer and parser fuzz/property tests are absent, especially for Unicode, recovery, nesting, and large inputs.

## linguini-schema

60. **[High]** Duplicate enum variants are collected into a `BTreeMap` and silently collapsed without a diagnostic.
61. **[High]** Alias cycles and self-aliases are not detected; an alias is treated as a known type before its validity is established.
62. **[High]** Formatter annotations on type aliases are not copied into `TypeAliasSymbol`, so the semantic layer loses part of the schema contract.
63. **[Medium]** Duplicate parameter names in a message are not checked.
64. **[Medium]** Group `messages: Vec<String>` and the global message map can diverge and contain duplicate entries.
65. **[Medium]** Naming constraints are absent.
66. **[Medium]** Unknown-type diagnostics are often attached to an entire parameter or declaration span rather than the exact type token.
67. **[Medium]** Source identity is absent, so duplicate and related diagnostics across files cannot be rendered correctly.
68. **[Medium]** The builder duplicates part of analyzer and syntax logic despite being described as an internal helper; no single semantic model exists.
69. **[High]** During project merge, schema vectors are concatenated without a mandatory cross-file duplicate-declaration pass before code generation.
70. **[Low]** Tests cover basic registration, duplicate top-level declarations, and unknown types, but not cycles, duplicate variants or parameters, or cross-file merge.

## linguini-locale

71. **[Critical]** `LocaleDeclaration::Variable` is registered as `ScopeKind::Message`; variables enter the message map and no separate variable map exists.
72. **[High]** A cross-kind override inserts the new symbol in a new kind map but does not remove the old symbol from the previous map; one name may remain simultaneously a message, function, form, and enum.
73. **[High]** A same-file `override` is accepted before the duplicate-in-same-source branch and silently replaces the declaration instead of reporting an error.
74. **[High]** AST produced through parser recovery is still merged into scope and may shadow a valid parent declaration.
75. **[Medium]** Symbols store `source_index` and path, but a diagnostic related span carries no file identity; the renderer cannot display the correct file.
76. **[Medium]** A group name is registered as a message even though IR and codegen mainly operate on member messages, creating a phantom symbol.
77. **[Medium]** The order of `sources` defines parent/child semantics, but the loader does not validate path or locale hierarchy.
78. **[Medium]** One declarations map combines all kinds while separate indexes are not synchronized during replacement.
79. **[Medium]** Duplicate members inside enums, forms, and functions and cross-source collisions are delegated to an incomplete analyzer.
80. **[Low]** Tests are missing for variable classification, cross-kind override cleanup, recovered-source shadowing, and source-order validation.

## linguini-ir

81. **[High]** Lowering removes all spans and source locations; codegen and reference errors after IR cannot identify the original file and range.
82. **[High]** Locale enum declarations are not represented or are lost in lowered locale IR even though the analyzer and examples use locale enums.
83. **[High]** Override semantics are erased instead of being handled in an explicit resolve/replace phase, so provenance and replacement errors are unavailable.
84. **[High]** The reference checker skips expressions inside forms, form attributes, maps, and nested objects.
85. **[High]** The reference checker mainly validates the root segment and does not verify member or property existence.
86. **[High]** Call arity and types are not checked.
87. **[High]** Form and function references remain conflated with calls because an empty arguments vector is ambiguous.
88. **[High]** Variables and helpers have no cycle detection; codegen can produce infinite recursion.
89. **[High]** Formatter kinds and options are not validated before code generation.
90. **[Medium]** Documentation trimming differs from schema and locale helper crates, producing inconsistent public docs and hover text.
91. **[Medium]** The hard-coded name `plural` exists at several layers instead of being defined through one symbol or builtin registry.
92. **[Medium]** Errors are source-less and have no related edge or call site.
93. **[Medium]** Public vector structures permit duplicates and invalid ordering.
94. **[Medium]** Context sets and `BTreeMap`s silently collapse duplicate names.
95. **[Medium]** One `IrModule` represents both schema and locale data, allowing many meaningless combinations of fields.
96. **[Critical]** The codegen API does not require a validated-IR token or capability, so invalid IR reaches the production emitter.
97. **[Low]** Tests do not cover nested references, cycles, duplicate collapse, form expressions, or the invalid-IR boundary.

## linguini-analyzer

98. **[Critical]** Call detection is based on `arguments.is_empty()`: `foo()` is analyzed as a reference rather than a call.
99. **[High]** Only the first one or two property-path segments are checked; deeper paths remain partially or entirely unchecked.
100. **[High]** Form and method calls receive almost no arity or type checking.
101. **[High]** Implicit plural use without numeric variables does not produce the required error; an explicit plural argument is not required to be `Number` or `Decimal`.
102. **[High]** User functions are checked at most for arity; argument types are not checked.
103. **[High]** The builtin plural function is checked for argument count but not argument types.
104. **[High]** Local enum variants and variants used in `impl` or form branches are not fully checked for existence.
105. **[High]** Every non-String parameter becomes a dispatch dimension, including Date and Boolean, which is conceptually incorrect.
106. **[High]** A function containing only String parameters has zero dispatch dimensions and is poorly represented by the current branch model.
107. **[Medium]** Duplicate map keys or branches may be overwritten or collapsed without a diagnostic.
108. **[Medium]** Message-local names may silently shadow globals.
109. **[High]** There is no complete type-inference system or type environment.
110. **[High]** A wildcard `_` in any position prematurely completes exhaustiveness checking and hides missing variants.
111. **[High]** The requirement that `_` must be last is not checked, and arms after a wildcard are not reported as unreachable.
112. **[High]** Duplicate branches are not diagnosed.
113. **[High]** Coverage checking for multi-key or nested branches focuses mainly on the first key or dimension.
114. **[High]** `other` is accepted as a universal branch even for enums that do not define such a variant.
115. **[Medium]** The missing-arm quick fix inserts text heuristically and may break indentation or grammar.
116. **[Medium]** The `impl`-variant quick fix creates an empty body that may be invalid or meaningless.
117. **[Medium]** Locale enums can overwrite schema enums in lookup tables.
118. **[High]** Unknown form targets or types are silently skipped in several paths instead of producing an error.
119. **[High]** The documented lints `param_order`, `unreachable_arm`, `collapsible_arms`, `fn_without_strings`, `incomplete_impl`, and `redundant_wildcard` are not implemented.
120. **[Medium]** The promised unused-message analysis is not possible with the current file-local reference graph and is effectively absent.
121. **[High]** Cross-file and cross-kind duplicate declarations are analyzed incompletely.
122. **[Medium]** Duplicate nodes in the reference graph use last-wins semantics and lose the earlier declaration.
123. **[Medium]** The cycle detector emits repeated diagnostics from different start nodes instead of reporting one strongly connected component with the complete cycle.
124. **[Medium]** Cycle diagnostics do not show source paths or graph edges.
125. **[Medium]** DFS does not use a shared memoized visited result and may repeatedly traverse the graph.
126. **[Medium]** Project references cover only a limited subset of locale-level relationships.
127. **[High]** Diagnostics have no stable code, category, or source ID.
128. **[Medium]** Related spans are assumed to belong to the same file; CLI and LSP renderers reuse the primary path.
129. **[Medium]** Span bounds are not validated before rendering.
130. **[Medium]** `QuickFix` may contain both a command and a replacement without clear atomic semantics.
131. **[Medium]** `without_source` leaves a fake `0..0` span instead of using a distinct source-less diagnostic type.
132. **[Low]** The CLI renderer is ASCII-oriented and does not account for Unicode display width.
133. **[Medium]** Lint names and severity policy are not attached to diagnostics, so suppression and configuration are impossible.

## linguini-format

134. **[High]** An unknown file extension defaults to Locale formatting, so an arbitrary file may be interpreted using the wrong grammar.
135. **[Medium]** `sort_enum_variants` is publicly exposed but unused.
136. **[High]** The formatter uses a token/heuristic IR rather than the canonical AST; parser recovery and formatter behavior therefore duplicate decisions independently.
137. **[High]** Private-use characters U+E000 and U+E001 are used as markers and then removed globally; legitimate user text containing them is corrupted.
138. **[Medium]** Line width counts Unicode scalar values rather than graphemes or display columns, so CJK text, combining marks, and emoji are measured incorrectly.
139. **[Medium]** Newline style is always normalized to LF, losing CRLF.
140. **[High]** An invalid span becomes an empty string through `unwrap_or_default`, hiding data loss rather than reporting an error.
141. **[Medium]** `max_line_width` is applied only partially and is not a strict limit.
142. **[Medium]** Extremely large indentation or width values may cause excessive allocations or multiplication overflow.
143. **[Medium]** Comment and doc-comment movement is based on token heuristics and breaks on malformed input.
144. **[Medium]** The formatter reparses and re-lexes the same source, increasing cost.
145. **[Medium]** `max_line_width = 0` has the special meaning “unlimited,” but this is not documented as a contract.
146. **[High]** Combined with raw-text trimming, formatting can change message meaning rather than only layout.
147. **[Low]** Property tests are absent for idempotence, comment preservation, Unicode, CRLF, and malformed input.

## linguini-cldr

148. **[High]** `PluralOperands::parse` removes all leading signs, so malformed values such as `--1` or `+-1` may be accepted as numbers.
149. **[High]** Exponent notation and compact-decimal operands `c` and `e` are unsupported; `c` and `e` are effectively always zero.
150. **[High]** Large decimal or integer operands can overflow `u64`.
151. **[High]** Conversion through `f64` loses precision for large values and long fractional parts.
152. **[High]** Casting `f64` to an integer may saturate or round in ways inconsistent with CLDR operand semantics.
153. **[Medium]** The same input is reparsed or reconverted several times while computing operands and rules.
154. **[Medium]** An empty rule is treated as true, so correctness depends on category order.
155. **[Medium]** Errors are represented as strings without a structured kind or offset.
156. **[High]** Locale fallback is simple `-subtag` truncation rather than CLDR `parentLocales`, `likelySubtags`, and canonical aliases.
157. **[High]** Number formatting supports only the `latn` numbering system.
158. **[High]** Date formatting supports only Gregorian dates and lacks standalone contexts, eras, day periods, time zones, week data, and skeleton matching.
159. **[High]** Currency formatting does not apply currency-specific symbols, fraction digits, rounding increments, or spacing.
160. **[Medium]** The plural parser is duplicated across runtime, procedural macro, and TypeScript generation.
161. **[Medium]** The documentation and builtin plural example list an incomplete category set, while generated data includes `zero` and `two`.
162. **[Low]** There are no differential tests against ICU or the CLDR conformance corpus.

## linguini-cldr-macros

163. **[Critical]** The procedural macro performs network access and launches `git` during compilation.
164. **[Critical]** `LINGUINI_CLDR_SOURCE_CHECKOUT_DIR` selects a directory that `fetch_cldr_json` may delete with `remove_dir_all` without a containment guard, allowing arbitrary data deletion.
165. **[Critical]** Compilation mutates a checkout or vendor path inside the source tree, or a path supplied through the environment.
166. **[High]** The build is neither hermetic nor offline and depends on `git`, GitHub, DNS, and network availability.
167. **[High]** Pinning validates only a short commit prefix—seven hexadecimal characters in config—and no content checksum or signature is used.
168. **[High]** The `git ref` is fetched from a remote; a ref plus short prefix is weaker than a full immutable SHA and archive checksum.
169. **[High]** `is_usable` checks only plural and misc directories, not numbers, dates, or layout payloads; a partial checkout may pass early validation.
170. **[High]** Lock acquisition may wait up to 60 seconds inside the `rustc` or procedural-macro process.
171. **[High]** The lock directory has no PID or ownership metadata and no reliable stale-lock recovery.
172. **[Medium]** The lock's `Drop` implementation blindly removes the directory without checking ownership, creating a race.
173. **[High]** There is no build-script-style `cargo:rerun-if-env-changed` or tracked-input semantics; procedural-macro cache invalidation is opaque.
174. **[High]** Large filesystem, JSON, and code-generation workloads run inside the compiler process.
175. **[High]** Malformed or missing locale subtrees are sometimes silently skipped, so generated locale coverage depends on checkout quality without manifest validation.
176. **[High]** Generated data is limited to Latin numbering and Gregorian calendars even though the source CLDR dataset is broader.
177. **[High]** Text direction is extracted by substring search rather than JSON parsing, making it fragile to formatting and key-order changes.
178. **[High]** The number-pattern parser ignores quoting, significant digits, scientific notation, padding, currency spacing, percent/per-mille scaling, and rounding increments.
179. **[Medium]** Some sizes and counts are cast to `u8` without explicit validation of model limits.
180. **[Medium]** The plural grammar parser duplicates runtime logic.
181. **[Medium]** `cldr-json.toml` is parsed by another ad hoc TOML reader.
182. **[High]** The supply-chain boundary does not cryptographically verify the downloaded archive or content tree.
183. **[Low]** Tests use minimal synthetic fixtures and do not validate the complete pinned CLDR corpus or a golden hash.

## linguini-codegen-ts — core emitter

184. **[Critical]** `form_object` discards `IrFormEntry::Branch`, while call emission invokes `TypeForms[root](...)`; the generated value is an object, so a valid form can terminate in a runtime `TypeError`.
185. **[Critical]** Multi-key branch emission uses only `branch.keys.first()`, losing every remaining selector or variant.
186. **[High]** `map_expression` always sends the selector through the plural function regardless of the actual enum or string dispatch type.
187. **[High]** A zero-argument call is indistinguishable from a reference and is not emitted as a call.
188. **[High]** An unknown formatter becomes a silent no-op even though the reference promises a compile-time error.
189. **[High]** `function_name` and `safe_identifier` replace only selected characters such as `.` and `-`; reserved words, leading digits, Unicode escapes, and collisions can produce invalid TypeScript.
190. **[High]** Type and parameter names are emitted almost raw; a schema identifier is not guaranteed to be a valid or unique TypeScript identifier.
191. **[Medium]** String escaping does not fully handle carriage returns and control characters.
192. **[Critical]** Recursive alias resolution for formatter defaults has no cycle guard and can overflow the stack.
193. **[High]** An invalid currency code reaches the Intl or formatter path and may cause a runtime `RangeError`.
194. **[High]** Currency formatting uses a generic pattern and ignores minor units such as JPY and currency-specific rules.
195. **[High]** Number formatting through JavaScript `Number` or `f64` loses precision.
196. **[High]** The date formatter uses local-time-zone getters; SSR and browser output may differ depending on the environment time zone.
197. **[Medium]** Invalid `Date` values are not checked and yield meaningless components.
198. **[High]** Missing CLDR data silently falls back to `String(value)` or generic currency text, hiding an incomplete build.
199. **[Medium]** There are no source maps or mappings from generated lines to Linguini source.
200. **[High]** When the API is used directly, a missing locale message may not be generated even though the `.d.ts` schema still declares it.
201. **[High]** Project loaders statically import every locale and return `Promise.resolve`, so the lazy-shaped API does not provide code splitting.
202. **[Medium]** Tree shaking filters messages but leaves unused forms, functions, variables, and their dependencies.
203. **[Medium]** Unknown `included_messages` entries are silently ignored.
204. **[High]** A namespace module copies all enums, aliases, forms, functions, and variables into every namespace, inflating output and creating collisions.
205. **[High]** Formatter helpers and data are repeated in every namespace module, contradicting the documentation's “once per locale module” and `FORMATTER_DATA` claims.
206. **[High]** Namespace and file names are not sanitized for paths, identifiers, or case-insensitive filesystems.
207. **[High]** Locales differing only by case can collide on Windows and macOS.
208. **[High]** Locale fallback is hyphen truncation rather than CLDR parent-locale resolution.
209. **[Medium]** An invalid base locale is silently replaced by the first locale.
210. **[Medium]** An unknown text direction is silently treated as LTR.
211. **[High]** The `targets.ts.module` option and documented `esm | cjs` choice are dead configuration surface: validation accepts only `esm`, the emitter supports only ESM, and CJS is not part of the current product direction. Remove the field and all CJS documentation instead of implementing another module format.
212. **[Medium]** A top-level parameterless message is generated as a function, while a grouped parameterless message is generated as a string; the API is inconsistent.
213. **[High]** With an empty locale set, the index uses a fake `"" as LinguiniLanguageInput`, after which runtime lookup may return `undefined`.
214. **[High]** `selectBranch` returns an empty string when no branch exists, masking invalid IR or an exhaustiveness failure.

## linguini-codegen-ts — web/SvelteKit runtime

215. **[High]** `readStrategy` implements only base locale, URL, cookie, local storage, header, and navigator; `preferredLanguage`, `globalVariable`, and `custom-*` are effectively no-ops.
216. **[High]** The default strategy includes `preferredLanguage`, so default behavior already differs from configuration.
217. **[Medium]** The header strategy assumes an object with `.get`; a plain headers object may cause a `TypeError`.
218. **[High]** `Accept-Language` parsing ignores quality weights and wildcards.
219. **[High]** Malformed percent encoding in a cookie causes `decodeURIComponent` to throw.
220. **[High]** Cookie name, value, path, domain, and max-age are insufficiently sanitized, and the `SameSite=None` plus `Secure` constraint is not enforced.
221. **[High]** When `setHeaders` is present, `setLocaleCookie` may overwrite existing `Set-Cookie` headers.
222. **[High]** Public `localizeUrl` and `localizeHref` methods may rewrite external URLs; the same-origin guard exists only in `shouldLocalizeHref`.
223. **[Critical]** `stripLeadingLocale` uses fallback matching: a segment such as `en-products` can be interpreted as locale `en` and partially or completely corrupt the route.
224. **[Medium]** `trailingSlash = "directory"` is accepted but not implemented.
225. **[Medium]** An exclusion pattern such as `/api/**` becomes an overly broad prefix and also matches `/apix`.
226. **[Medium]** A user-supplied global or sticky `RegExp` is stateful through `lastIndex`, so exclusion results depend on call history.
227. **[Medium]** URL constructors may throw, but public methods have no consistent error contract.
228. **[High]** HTML is rewritten with regular expressions rather than a parser and breaks on `>` inside quotes, comments and scripts, malformed markup, and entities.
229. **[Medium]** Server and client skip rules differ; for example, `rel=external` is handled asymmetrically.
230. **[High]** SvelteKit response localization buffers the entire HTML response and destroys streaming, creating unbounded memory cost.
231. **[Medium]** The generated handle writes generic `locals.locale`, `locals.direction`, and `locals.l`, which may collide with application fields.
232. **[Medium]** Generated handle, reroute, and load hooks require composition, but their contract and names can easily conflict with existing hooks.
233. **[Low]** `redirectStatus` exists in the runtime path, but generated options do not pass it; the code is nearly dead.
234. **[Medium]** The auto-link observer is a singleton across app instances and HMR, leaving lifecycle ownership unclear.
235. **[Medium]** `MutationObserver` may traverse large DOM subtrees expensively without batching or debounce.
236. **[High]** Static imports of every locale increase bundle size and startup cost despite the loader-shaped API.

## linguini-cli

237. **[Critical]** The configured output path may be absolute or contain `..`; `root.join(absolute)` ignores the root, and the tree writer deletes or renames the entire directory, risking arbitrary data destruction.
238. **[Critical]** Output may overlap the project root, schema directory, or locale directory; a build can replace source files with the generated tree.
239. **[Critical]** No canonical containment or symlink-escape checks run before removal or rename.
240. **[High]** Staging and backup names are based on the PID; stale directories are deleted without an ownership manifest.
241. **[High]** If output commit succeeds but backup removal fails, the CLI returns an error even though output has already changed.
242. **[High]** Rollback errors are ignored; a failure can leave the old and new trees missing or only partially restored.
243. **[Medium]** Whole-tree replacement removes any user files inside generated output; there is no ownership guard, banner, or manifest.
244. **[High]** Schema files are simply extended during merge, so project-wide duplicate declarations do not necessarily block generation.
245. **[High]** Namespace prefixes are applied to messages, but forms, functions, and variables remain global and may collide or resolve incorrectly.
246. **[High]** `locale_index` silently collapses duplicate `(namespace, locale)` entries in a `BTreeMap`.
247. **[High]** Secondary fallback uses only the direct default locale, while codegen and runtime add different fallback logic; semantics are duplicated.
248. **[Critical]** `check_project` sends every diagnostic from `analyze_locale_file` to warning output without filtering by severity; a semantic `Error` may not block `check` or `build`.
249. **[Medium]** Any syntax error causes project semantic analysis to be skipped entirely, hiding independent errors.
250. **[Medium]** Warnings cannot be promoted to errors because `--deny-warnings` is absent.
251. **[High]** `ensure_no_unresolved_references` inherits the incomplete IR reference walker and source-less diagnostics.
252. **[High]** `format` can modify a supplied path outside the project and follow a symlink target; this is a powerful action without containment or a default dry run.
253. **[High]** Formatting writes are not atomic.
254. **[High]** The fix workflow reads an old source snapshot and then overwrites the file without optimistic concurrency, a content hash, or a lock.
255. **[Medium]** A `CreateFile` fix reports “created” even when the file already existed and was not changed.
256. **[Medium]** Fix filtering by suffix or title substring is ambiguous and may select unrelated fixes.
257. **[Medium]** Generated stubs use raw names and limited group-name splitting; nested names and collisions are handled poorly.
258. **[Medium]** Multiple fixes may be obsolete, overlapping, or duplicated; conflict detection is incomplete.
259. **[Medium]** `init` prints paths as created even for pre-existing items or no-op operations.
260. **[Low]** `generate` prints ANSI sequences without a reliable TTY or `NO_COLOR` policy.
261. **[High]** Sample generation reloads the schema for each locale, producing O(locales × schema files) work.
262. **[High]** The sample generator does not qualify namespaces consistently and may collide names.
263. **[Critical]** Sample alias recursion has no cycle guard and may overflow the stack.
264. **[High]** The Cartesian product of enum parameters causes exponential sample growth.
265. **[Medium]** The sample generator recognizes `Integer` and `DateTime`, which do not exist in core or schema models.
266. **[High]** The evaluator returns an empty string for several failures and hides the defect.
267. **[High]** Sample formatting differs from production codegen for currency and dates, so preview output is not a runtime oracle.
268. **[Medium]** Numbers are limited to `i64` and a small sample set and do not cover decimals or large values.
269. **[Medium]** There is no machine-readable JSON or SARIF output for CI and editor integration.
270. **[High]** `linguini lsp` calls the stdio runner through `expect`; failure to construct the Tokio runtime causes a panic.

## linguini-lsp

271. **[Critical]** A locale document is analyzed against every schema document in the project instead of only the matching namespace, creating false missing and unknown diagnostics.
272. **[High]** References are searched only in the current document despite workspace context.
273. **[Critical]** Rename performs lexical replacement of identical token text across all documents without symbol resolution or scope and therefore renames unrelated symbols.
274. **[High]** Rename is allowed for any identifier or locale tag and does not validate the new name against grammar, reserved words, or collisions.
275. **[High]** Definition fallback chooses the first lexical occurrence in the same document and may navigate to a reference rather than a declaration.
276. **[High]** Workspace schema definition and hover select the first unqualified message name and ignore namespace ambiguity.
277. **[Medium]** Completion uses current-document symbols and fixed `count`, `other`, and `_` suggestions without schema, type, or context awareness.
278. **[High]** Schema documents receive syntax diagnostics, but schema semantic-builder diagnostics are not fully integrated.
279. **[Medium]** Any syntax error suppresses semantic diagnostics entirely.
280. **[High]** Filesystem discovery and reads are synchronous inside async request handlers.
281. **[High]** Many requests rescan and reparse the project; no index, cache, or incremental model exists.
282. **[High]** Recursive workspace discovery follows directories without a canonical visited set, depth limit, or file limit; symlink cycles and denial of service are possible.
283. **[High]** There is no debounce, cancellation, or document-version gate for `didChange`; an older analysis can publish after a newer one.
284. **[Medium]** `publishDiagnostics` sends version `None`, so the client cannot reject stale diagnostics.
285. **[Medium]** `didClose` clears one URI but does not recompute dependent documents.
286. **[Medium]** The server does not systematically track disk, config, or source changes outside open documents.
287. **[Medium]** Mutex poisoning becomes an empty state, hiding an internal failure.
288. **[High]** Custom file-URI conversion loses authorities and UNC information and handles non-UTF-8 paths poorly.
289. **[Medium]** Unknown language IDs default to Locale instead of being explicitly unsupported.
290. **[Medium]** `contains` uses an inclusive end, so a cursor immediately after a token is considered inside it.
291. **[Medium]** Semantic tokens omit multiline raw strings entirely.
292. **[Medium]** Token classification relies on capitalization and neighbor heuristics and is frequently semantically wrong.
293. **[Low]** The “Show diagnostic” code action contains no edit or command and does nothing.
294. **[Medium]** A rename action is offered on a zero-length range even when no symbol exists.
295. **[High]** Apply-all quick fixes may return overlapping or same-position edits without deterministic conflict resolution.
296. **[Medium]** LSP diagnostics lose notes, related-file identity, stable codes, and rich quick-fix metadata.
297. **[High]** A `QuickFix` may reference a command ID, but the server neither declares nor implements an `executeCommand` provider.
298. **[Medium]** A formatting failure becomes `None`, hiding the reason from the client.
299. **[Medium]** Formatting returns a whole-document edit and ignores client formatting options.
300. **[Medium]** Workspace symbols mostly include open documents rather than a complete workspace index.
301. **[Medium]** `ReferenceParams.context.include_declaration` is ignored.
302. **[High]** Read and discovery failures are silently skipped, so results appear complete even when the workspace view is incomplete.
303. **[Medium]** Nearest-config fallback may associate a file outside schema and locale roots with an unrelated project.
304. **[High]** There are no limits on document size, project size, file count, recursion, or nesting.
305. **[High]** The LSP sample renderer duplicates the CLI evaluator and production runtime with different semantics.
306. **[Medium]** Hover and preview locale is inferred from the filename, which fails under custom layouts.
307. **[Critical]** `run_stdio_blocking` uses `expect` and panics when a runtime cannot be created.
308. **[High]** A code-action request recomputes diagnostics and rescans the workspace, increasing latency.
309. **[Medium]** Workspace-folder changes and file-operation notifications for rename, create, and delete are unsupported.

## linguini-test-support

310. **[High]** Temporary directory names use PID plus timestamp, are not created atomically, and may collide or be reused.
311. **[High]** `create_dir_all` accepts an existing directory, so a stale fixture may be reused as a new test project.
312. **[Critical]** `name` is inserted into the temporary path without rejecting separators or `..`; `Drop` later calls `remove_dir_all` on that path, allowing deletion outside the temp root.
313. **[Medium]** Times before the Unix epoch and creation failures are handled with panics.
314. **[Medium]** Cleanup errors are ignored and leave stale data.
315. **[Medium]** The crate does not use the proven `tempfile::TempDir`, even though the workspace already uses `tempfile` in CLI tests.
316. **[Medium]** Fixture paths assume a monorepo layout and work poorly in a packaged crate or source archive.
317. **[Low]** Its own tests barely cover isolation, collisions, path traversal, or cleanup.

## Vite plugin

318. **[High]** `isLinguiniSource` treats every `.lgs` or `.lgl` file under any path as a source file without checking configured roots.
319. **[Medium]** The watcher adds new files but does not remove obsolete watches after config, path, or unlink changes.
320. **[High]** `pendingBuild` deduplicates builds, but a change arriving during a build does not mark a dirty follow-up, so the latest source state may never be built.
321. **[High]** The async watcher `add` handler does not catch rejections, which may become unhandled errors.
322. **[High]** `unlink` and `unlinkDir` are not handled, and source deletion does not trigger a rebuild.
323. **[Medium]** Build errors are not converted into a Vite overlay or HMR error with source context.
324. **[Medium]** Generated-module invalidation relies on hard-coded substrings; a custom output path may remain cached.
325. **[Low]** The watch list includes the config path even when it does not exist.
326. **[Medium]** Config paths are read by a separate regex-based TOML parser.
327. **[Medium]** Plugin version 0.1.0 is not synchronized with crate and site alpha.4 versions.
328. **[Medium]** Compatibility with Vite 5 through 8 is claimed, but tests are mock based and do not run those real Vite versions.
329. **[Medium]** Tests are absent for races, build errors, config changes, unlink, custom output, and Windows paths.
330. **[Medium]** A CLI process is launched for every hot update without debounce, cancellation, or backpressure.
331. **[Low]** The plugin entry point is source JavaScript without a build step; this can work for Node ESM, but the compatibility contract should be explicit.

## VS Code extension

332. **[High]** The extension does not bundle a CLI or LSP binary and depends on an external `linguini` of the correct version; there is no compatibility negotiation. Its current esbuild plus `.vscodeignore` packaging also excludes `node_modules`, so adding a binary npm dependency alone would not make the executable available inside the VSIX.
333. **[Medium]** The extension is 0.1.4, crates are 0.1.0-alpha.4, and the plugin is 0.1.0; the release train is unsynchronized.
334. **[High]** Several config and workspace events may start concurrent `restartClient` operations; there is no debounce or serialization.
335. **[Medium]** Each restart pushes a new client disposable into subscriptions, so stopped clients and disposables accumulate.
336. **[High]** `client` is reassigned before the previous client finishes stopping, allowing races and double starts.
337. **[Low]** `createClient(context)` does not use its `context` parameter.
338. **[High]** Only the first workspace folder is used for the working directory and substitutions, so multi-root workspaces are handled incorrectly.
339. **[Medium]** `${file}` and `${languageId}` substitutions are supported by a helper, but `createClient` does not pass a document, so they are always empty in server paths and arguments.
340. **[Medium]** The `**/linguini.toml` watcher may cause restart storms without debounce.
341. **[High]** There are no extension unit or integration tests and no smoke test with a real language server.
342. **[High]** Packaging invokes `npx --yes @vscode/vsce` and `ovsx` without a strictly pinned executable, reducing reproducibility and increasing supply-chain risk.
343. **[Medium]** npm and pnpm lockfiles coexist, creating dependency-graph drift.
344. **[Medium]** Grammar and language configuration are not checked by golden tests against the actual lexer.
345. **[Medium]** Binary update and version mismatches are discovered only by the user; no health or version handshake exists.
346. **[Low]** The extension compile could not be fully verified in the audit environment because `npm ci` did not finish before timeout; this is an audit limitation, not a confirmed repository defect.

## Site and documentation

347. **[High]** Site build and check invoke `cargo run`, and the procedural macro may download CLDR; an ordinary frontend build therefore depends on network, `git`, and a Rust toolchain.
348. **[High]** The reference promises inline `fn`, a CJS mode, six lints, unknown-formatter errors, and shared `FORMATTER_DATA` that are absent from the implementation. The CJS promise should be deleted now; it should not remain as a roadmap commitment while the project is intentionally SvelteKit/ESM-first.
349. **[High]** `docs/why.md` presents compile-time exhaustiveness and type-mismatch detection as completed differentiators even though the analyzer is incomplete.
350. **[Medium]** The site strategy includes `preferredLanguage`, which the generated runtime does not implement as a separate strategy.
351. **[Medium]** Documentation describes project-level namespaces and path namespaces ambiguously.
352. **[Medium]** Examples are not run as compile, typecheck, and runtime golden tests.
353. **[Medium]** Documented formatter capabilities exceed the actual CLDR constraints and fallback behavior.
354. **[Medium]** `REPOSITORY_STATE` contains an unfinished audit and cleanup checklist for every crate, plugin, and site component; the project itself records unstable contracts.
355. **[Low]** Version, copy, and status may diverge between the published site, README, packages, and extension.

## CI, release, and supply chain

356. **[High]** The workspace declares Rust 1.76, but CI runs only `stable`; the MSRV is not tested.
357. **[Critical]** CI compilation may clone CLDR through a procedural macro, so the build is non-hermetic and the supply-chain boundary sits inside `rustc`.
358. **[High]** There is no Windows or macOS CI despite extensive path, URI, and case-sensitivity logic.
359. **[High]** The primary pipeline has no `cargo-deny`, `cargo-audit`, or equivalent dependency, license, and advisory check.
360. **[Medium]** Coverage thresholds, fuzz/property jobs, and mutation tests are absent.
361. **[High]** There is no generated-TypeScript corpus that runs `tsc` and runtime execution across the key language constructs.
362. **[High]** There is no real Vite integration matrix or VS Code extension integration and end-to-end testing.
363. **[Medium]** No `cargo package` or publish dry run covers every publishable crate; missing vendored CLDR content in source archives remains a risk.
364. **[Medium]** GitHub Actions are pinned to mutable major tags rather than commit SHAs.
365. **[Medium]** There is no `rust-toolchain.toml`, so local reproducibility differs from CI.
366. **[Medium]** The JavaScript ecosystem uses different package managers and lock conventions across plugin, site, and extension.
367. **[Medium]** Component release versions are not synchronized or checked by automation.
368. **[Medium]** There are no API, ABI, or semantic-version compatibility checks for public crates or the generated JavaScript/JSDoc and TypeScript declaration surfaces.
369. **[High]** Documentation and examples are not part of an executable conformance suite.

## Required documentation migration

The implementation changes in this report require a coordinated documentation rewrite, not isolated edits to individual examples. Documentation must describe only shipped behavior, use the final public API, and be executable as part of the conformance suite. The following file-level changes are part of the remediation acceptance criteria.

### `README.md`

- Present Linguini explicitly as SvelteKit-first and ESM-only for the current release line.
- Remove CJS, multi-framework, and completed-type-safety claims that are not currently implemented.
- Use the final generated API consistently: parameterless messages are values such as `l.shop.main.title`; parameterized messages use the existing positional form such as `l.cart.items(count)`, with `l.cart.items({ count })` documented as an additional generated overload.
- Explain that Vite owns tree-shaking, route code splitting, shared chunks, preload, and network loading. Do not describe a Linguini route loader or imply that all locale/message modules are loaded eagerly after the new architecture is implemented.
- Show the npm-installed CLI path as the default JavaScript-ecosystem installation, while retaining Cargo installation as an alternative.

### `docs/getting-started.md`

- Replace the old flat `[web]` example with the nested SvelteKit policy namespaces and show the genuinely minimal configuration first. Do not require `[web.routing]` when the default `locale_prefix = "except-default"` is sufficient.
- Remove `targets.ts.module` and every `esm | cjs` choice.
- Explain the single ECMAScript runtime model: JavaScript output carries JSDoc; TypeScript projects consume the same runtime plus generated `.d.ts` declarations.
- Document Vite/SvelteKit setup without asking users to create manual message-loading files. Generated hooks and virtual modules should be implementation details unless users need an explicit integration hook.
- Include one verified JavaScript example and one verified TypeScript/Svelte example.

### `docs/reference.md`

- Define recursive message groups and canonical qualified paths, including collisions between groups and messages.
- Define parameterless message access, positional calls, and the named-object codegen overload precisely. State that the named-object overload does not change Linguini source syntax.
- Specify ordinary triple-quoted multiline messages with trim-indent semantics and `raw\"\"\"...\"\"\"` messages with byte-preserving whitespace semantics. Specify interpolation and literal-brace escaping separately.
- Specify how `///` documentation attaches to declarations and is preserved in generated JSDoc, `.d.ts`, editor hover, and navigation metadata.
- Remove inline `fn` from the reference until it is implemented and tested, or mark it as an explicit future proposal outside the normative grammar. The normative reference must not accept syntax the parser rejects.
- Remove CJS completely.
- Correct formatter, plural-category, lint, exhaustiveness, type-checking, and unknown-formatter claims to match implemented behavior. Re-add stronger guarantees only when conformance tests prove them.

### `docs/web-sveltekit.md`

Document the exact configuration contract rather than describing web behavior generically:

- `[web.routing].locale_prefix = "always" | "except-default" | "never"`, defaulting to `"except-default"`;
- ordered `[web.locale].sources` with the supported explicit values and `default_locale` as an implicit final fallback;
- `[web.cookie]` and `[web.local_storage]` namespaces, defaults, validation, and the rule that configuration does not enable a source unless that source is selected;
- `[web.links].mode`, with static transformation and the existing `localizeHref` for dynamic links;
- `[web.routes].exclude` matching semantics;
- the optional `[web.switch_route]` as an HTTP transport over the same `LocaleSwitchPlan` used by `switchLocale`, not as a separate persistence strategy;
- path, cookie, hybrid, and pathless/local-storage examples;
- the exact no-JavaScript limitation: an HTTP route cannot persist a local-storage-only locale choice, while path and cookie strategies can support ordinary static links;
- origin, base path, and trailing-slash behavior as values derived from SvelteKit/request context rather than user configuration.

### `docs/examples/sveltekit-locale-provider.md`

- Update every call site to the final value/function distinction.
- Replace any duplicated URL-localization helper with the existing `localizeHref`.
- Demonstrate `switchLocale` and static switch links as two transports over the same configured locale transition.
- Include an example with `locale_prefix = "never"` and explain which persistence source makes it work.
- Avoid examples that manually import every locale or every message.

### `docs/why.md`

- Remove claims that complete exhaustiveness, type mismatch detection, unused-message analysis, and all documented lint checks are already delivered.
- Describe the intended advantages as goals until they are backed by executable tests.
- Explain the concrete bundler-native model: static `l` leaf access becomes an exact ESM dependency and Vite determines route/shared chunks.
- Do not claim payload reductions without a reproducible benchmark that records route, locale count, message count, and build configuration.

### Site, templates, and generated examples

- Update `site/linguini.toml`, site SvelteKit hooks, examples, and displayed code snippets to use the new nested web configuration and API.
- Remove CJS options from project templates, sample configuration, generated comments, CLI help, schema/autocomplete data, and screenshots.
- Update VS Code hover/completion examples for recursive groups, doc comments, multiline blocks, parameterless values, and both generated call forms.
- Add a migration document or release-note section mapping every removed flat field to its replacement or automatic source, including `prefix_default_locale`, `strategy`, cookie fields, `local_storage_key`, `localize_links`, `exclude`, `origin`, `base_path`, `trailing_slash`, and `targets.ts.module`.

### Documentation enforcement

- Extract or register every normative code block as a fixture. CI must parse, analyze, generate, typecheck, and execute the applicable examples.
- Build the documentation against the packaged CLI/codegen artifacts, not against ad hoc source-tree behavior.
- Fail CI when generated configuration schema, CLI help, public TypeScript declarations, or documented examples change without their corresponding documentation snapshots.
- Require documentation, migration notes, and conformance fixtures in the definition of done for every public syntax, configuration, runtime, or generated-API change.

## Additional architectural requirements for remediation

These are product-direction requirements added after the initial finding inventory. They refine the remediation target without changing the count of 369 defects recorded by the audit.

### Replace the flat web configuration with compile-time SvelteKit policy namespaces

The web configuration should remain in `linguini.toml`. Moving cookie, routing, locale resolution, exclusions, and link behavior into separate application files would make setup harder and would prevent code generation from excluding unused capabilities. The problem is not that web configuration exists; the problem is that the current flat `WebConfig` exposes unrelated low-level fields and compiles them into one universal runtime.

The replacement must be a typed, nested, SvelteKit-first configuration. Each selected capability maps to a generated ESM module. No runtime loop should dispatch over every strategy implementation.

#### Canonical configuration model

A complete explicit configuration should look like this:

```toml
[web.routing]
locale_prefix = "except-default" # "always" | "except-default" | "never"
canonical = "redirect"           # "redirect" | "preserve"

[web.locale]
sources = ["path", "cookie", "local-storage", "accept-language"]

[web.cookie]
name = "LINGUINI_LOCALE"
path = "auto"
max_age = "365d"
same_site = "lax"
secure = "auto"
http_only = false
# domain = "example.com"

[web.local_storage]
key = "linguini:shop:locale"

[web.links]
mode = "transform" # "transform" | "runtime" | "manual"

[web.routes]
exclude = ["/api/**", "/_app/**", "/favicon.ico"]

[web.switch_route]
path = "/_linguini/locale/{locale}"
return_query = "return"
status = 303
```

This is the full surface, not the normal amount of configuration. Every section is optional and has defined SvelteKit-oriented defaults.

#### Routing: replace `prefix_default_locale` with one explicit enum

Delete `prefix_default_locale`. Its default is currently `false`, and a boolean cannot express the required no-locale-in-path mode. Replace it with:

```toml
[web.routing]
locale_prefix = "except-default"
```

The exact values are:

- `"always"`: every locale is represented in the pathname: `/en/shop`, `/de/shop`;
- `"except-default"`: the project default locale has no prefix, other locales do: `/shop`, `/de/shop`; this is the default and therefore does not need to be written;
- `"never"`: no locale is represented in the pathname: `/shop` for every locale.

The minimal standard SvelteKit project therefore needs no `[web.routing]` section. It receives `locale_prefix = "except-default"` automatically.

Canonical URL handling should be an enum rather than the independent `redirect` boolean:

```toml
[web.routing]
canonical = "redirect"
```

- `"redirect"` returns a canonical redirect when the incoming path does not match the selected locale policy;
- `"preserve"` accepts the incoming path without a canonical redirect.

`canonical` is ignored when `locale_prefix = "never"` because there is no locale path representation to canonicalize.

Delete `base_path`, `trailing_slash`, and `origin` from Linguini configuration:

- base path comes from SvelteKit `paths.base`;
- trailing-slash behavior comes from the matched SvelteKit route;
- request origin comes from `event.url.origin` on the server and `window.location.origin` in the browser.

These are request/framework facts, not localization policy.

#### Locale sources: exact names and exact validation

Replace the current `strategy` strings with:

```toml
[web.locale]
sources = ["path", "cookie", "accept-language"]
```

Supported initial values are only:

- `"path"`;
- `"cookie"`;
- `"local-storage"`;
- `"accept-language"`.

The project `default_locale` is always the final fallback and must not be repeated as `baseLocale` or `default` in the array. Remove `preferredLanguage`, `navigator`, `globalVariable`, `custom-*`, and the duplicated `baseLocale` strategy from the stable configuration contract.

Defaults are derived from routing:

- with `locale_prefix = "always"` or `"except-default"`, omitted `sources` means `["path", "cookie", "accept-language"]`;
- with `locale_prefix = "never"`, omitted `sources` means `["cookie", "accept-language"]`.

Validation must reject contradictory configurations:

- `"path"` is invalid when `locale_prefix = "never"`;
- `"cookie"` requires the cookie capability, using defaults when `[web.cookie]` is omitted;
- `"local-storage"` requires the browser storage capability, using a derived key when `[web.local_storage]` is omitted;
- duplicate sources are an error;
- an empty source list is valid and means “always use `project.default_locale`”.

A project that intentionally keeps locale out of the URL and resolves it only in the browser is written explicitly:

```toml
[web.routing]
locale_prefix = "never"

[web.locale]
sources = ["local-storage"]
```

This mode has an unavoidable SSR limitation: local storage is not readable by the server. The generated integration must not pretend otherwise. It must either be used on client-rendered routes or render the default locale on the server and document the post-hydration locale change. The compiler/plugin should emit a diagnostic when a local-storage-only resolver is combined with SSR expectations. Adding a cookie source is the way to make subsequent server requests locale-aware.

#### Cookie namespace

Replace all flat `cookie_*` fields with `[web.cookie]`:

```toml
[web.cookie]
name = "LINGUINI_LOCALE"
path = "auto"
max_age = "365d"
same_site = "lax"
secure = "auto"
http_only = false
# domain = "example.com"
```

Defaults:

- `name`: `LINGUINI_LOCALE` or a deterministic project-derived name;
- `path = "auto"`: use the SvelteKit base path, falling back to `/`;
- `max_age = "365d"`: parse a duration instead of requiring raw seconds;
- `same_site = "lax"`;
- `secure = "auto"`: secure on HTTPS deployments, disabled for local HTTP development;
- `http_only = false`, because the existing browser locale switcher writes the cookie; a future server-only mode may allow `true`;
- omitted `domain`: host-only cookie.

The cookie module is generated only when `sources` contains `"cookie"`. The switch route does not independently enable cookie persistence: it inherits the same compiled locale-switch behavior as `switchLocale` and therefore writes a cookie only when cookie persistence is part of that behavior. Merely customizing `[web.cookie]` without selecting `"cookie"` should produce an unused-configuration diagnostic.

#### Local-storage namespace

Replace `local_storage_key` with:

```toml
[web.local_storage]
key = "linguini:shop:locale"
```

If `"local-storage"` is selected and this section is omitted, derive the key from the project name. Do not generate local-storage reads, writes, exception handling, or hydration bootstrap code unless this source is selected.

#### Link behavior: use the existing `localizeHref`

Do not invent a second URL helper. `localizeHref` already exists and remains the API for dynamic links. Replace `localize_links` with one explicit mode:

```toml
[web.links]
mode = "transform"
```

The exact modes are:

- `"transform"` (default): the Svelte/Vite transform rewrites statically analyzable internal links at compile time. Dynamic href expressions are left to the existing `localizeHref` API. Do not generate the current `MutationObserver` link scanner in this mode;
- `"runtime"`: preserve the current server markup rewriting and client DOM observation behavior for applications that explicitly need it;
- `"manual"`: perform no automatic rewriting. `localizeHref` remains available and is included only when imported/referenced.

Route exclusions remain declarative because they are deterministic build inputs:

```toml
[web.routes]
exclude = ["/api/**", "/health", "/admin/**"]
```

The same compiled matcher must be used by locale resolution, canonical redirects, link transformation, and the locale-switch endpoint. Delete the flat `exclude` field.

#### Locale switching: one semantic operation for `switchLocale` and static routes

There must be exactly one locale-transition model. The browser `switchLocale` API (the current implementation calls it `setLocale`) and the generated switch route must both be emitted from the same validated `LocaleSwitchPlan`; the route must not define its own persistence `mode`, source order, or side effects.

`LocaleSwitchPlan` is compiled from `[web.routing]`, `[web.locale].sources`, `[web.cookie]`, and `[web.local_storage]`. Its default behavior is concrete:

1. validate and canonicalize the requested locale against the compiled locale set;
2. update the in-memory reactive locale in the browser;
3. write every selected writable persistence source that is available in the current environment;
4. construct the destination with the existing `localizeHref` and the configured routing policy;
5. navigate or redirect back to the requested return location;
6. synchronize generated link state after a browser-side switch.

Source capabilities are fixed:

- `"path"` is writable through navigation. Both `switchLocale` and the switch route use `localizeHref(returnHref, locale)` to encode the locale according to `locale_prefix`;
- `"cookie"` is writable in both environments. Browser `switchLocale` writes `document.cookie`; the switch route emits `Set-Cookie` using the same compiled cookie options;
- `"local-storage"` is writable only in the browser. Browser `switchLocale` writes it directly. After a path- or cookie-based switch route reaches the destination and JavaScript hydrates, generated bootstrap code may synchronize the resolved locale into local storage;
- `"accept-language"` is read-only and is never modified by either switching mechanism;
- `project.default_locale` is only the final fallback and is never treated as persistence.

The public switch API may expose navigation controls such as replace-state, focus, scroll, and invalidation, but those are transport options around the same transition plan. They must not change which locale sources are configured. The static route uses the plan's default persistence and navigation behavior.

##### Path-based switching

When `"path"` is selected, a special endpoint is not required for ordinary links. A no-JavaScript language switch is a direct localized URL:

```svelte
<a href={localizeHref(currentHref, "de")}>Deutsch</a>
```

This works with static adapters because it is a normal link. If `[web.switch_route]` is also enabled, its redirect target must still be produced by the same `localizeHref` call; it must not reimplement path rewriting.

##### Generated static switch route

The optional route is configured only as HTTP transport:

```toml
[web.switch_route]
path = "/_linguini/locale/{locale}"
return_query = "return"
status = 303
```

The section's presence enables generation; its absence disables it. There is deliberately no `mode = "cookie"`, `strategy`, or persistence field. Those would duplicate and potentially contradict `switchLocale`.

The generated SvelteKit integration must handle URLs such as:

```text
/_linguini/locale/de
/_linguini/locale/de?return=/account/settings
```

It must:

1. validate the locale with the same matcher used by `switchLocale`;
2. choose the return target from the configured query parameter, then a same-origin `Referer`, then the SvelteKit base path;
3. reject absolute, protocol-relative, encoded-external, and otherwise cross-origin return targets;
4. execute every server-capable write from the shared transition plan, such as `Set-Cookie` only when `"cookie"` is selected;
5. derive the redirect URL through `localizeHref` when `"path"` is selected, otherwise preserve the return pathname;
6. return the configured redirect status.

Static no-JavaScript links can therefore use:

```svelte
<a href="/_linguini/locale/en">English</a>
<a href="/_linguini/locale/de">Deutsch</a>
```

The resulting behavior depends on the configured sources rather than on route-specific logic:

- `sources = ["path"]`: redirect to the localized return path, no cookie;
- `sources = ["cookie"]` with `locale_prefix = "never"`: set the locale cookie and redirect to the unchanged return path;
- `sources = ["path", "cookie"]`: set the cookie and redirect to the localized return path;
- `sources = ["cookie", "local-storage"]`: set the cookie on the server, then synchronize local storage after hydration;
- `sources = ["local-storage"]` with `locale_prefix = "never"`: reject `[web.switch_route]` at compile time, because an HTTP response cannot reproduce `switchLocale` persistence without JavaScript;
- `sources = ["accept-language"]`: reject `[web.switch_route]` unless another writable source is selected, because Accept-Language cannot be changed by the application.

This keeps `switchLocale`, static links, SSR, and client navigation semantically aligned instead of maintaining separate locale-switch implementations.

#### Compile-time feature output

The configuration must lower to a closed feature set, not a generic options object:

```rust
struct WebFeatures {
    routing: LocalePrefixMode,
    canonical: CanonicalMode,
    source_order: Vec<LocaleSource>,
    cookie: Option<CookieConfig>,
    local_storage: Option<LocalStorageConfig>,
    links: LinkMode,
    route_exclusions: CompiledRouteMatcher,
    switch_route: Option<SwitchRouteTransportConfig>,
    locale_switch: LocaleSwitchPlan,
}
```

The SvelteKit/Vite adapter then requests only the required modules, for example:

```text
virtual:linguini/web/path
virtual:linguini/web/cookie
virtual:linguini/web/accept-language
virtual:linguini/web/local-storage
virtual:linguini/web/link-transform
virtual:linguini/web/runtime-links
virtual:linguini/web/switch-route
```

If `sources = ["local-storage"]`, no path parser, cookie serializer, or Accept-Language parser is emitted. If `links.mode = "manual"`, no SSR HTML rewriting or MutationObserver is emitted. If `[web.switch_route]` is absent, no switch-route branch is added to the generated hook. When it is present, the branch imports and executes the same generated `LocaleSwitchPlan` used by browser `switchLocale`; it does not select its own persistence strategy.

#### Exact migration from the current fields

- `strategy` → `[web.locale].sources`, with renamed and reduced values;
- `cookie_name`, `cookie_path`, `cookie_domain`, `cookie_max_age`, `cookie_same_site`, `cookie_secure`, `cookie_http_only` → `[web.cookie].*`;
- `local_storage_key` → `[web.local_storage].key`;
- `prefix_default_locale` → delete and replace with `[web.routing].locale_prefix`, whose default is `"except-default"`;
- `redirect` → `[web.routing].canonical`;
- `exclude` → `[web.routes].exclude`;
- `localize_links` → `[web.links].mode`;
- `origin`, `base_path`, and `trailing_slash` → delete and derive from SvelteKit/request context;
- `global_variable_name`, `globalVariable`, `preferredLanguage`, `custom-*`, and explicit `baseLocale` → remove from the stable SvelteKit-first contract.

The result keeps configuration centralized, makes behavior explicit, supports URL-prefixed and pathless localization, generates an optional no-JavaScript switch-route transport from the same `LocaleSwitchPlan` as `switchLocale`, reuses `localizeHref`, and allows code generation to omit every unselected capability.


### Make SvelteKit and ESM the only current web target

The current development phase should optimize for one coherent product: a SvelteKit integration that emits bundler-native ESM. CJS, generic framework abstractions, additional output formats, and additional target languages should not consume design or maintenance budget until the compiler core and SvelteKit path are reliable.

Immediate documentation and configuration cleanup:

- Delete every CJS claim from the reference, getting-started material, examples, rationale, and generated starter configuration. CJS is not a missing feature to implement now; it is an intentionally unsupported format.
- Remove `module` from `TypeScriptTargetConfig`, `TypeScriptTargetBuilder`, parser assignment, validation, tests, CLI init templates, `site/linguini.toml`, and all documentation examples. A setting whose only valid value is `esm` is not configuration.
- Do not replace it with another `format`, `module_kind`, or `output_type` enum until a second format is actually implemented and supported. ESM should be an invariant of the current web backend.
- Treat `framework = "sveltekit"` as the primary supported adapter. A generic Svelte mode may remain only if it shares the same implementation and does not force generic abstractions into the compiler core. Other frameworks should be postponed.
- Keep JavaScript generation as a first-class requirement. SvelteKit and Vite consume JavaScript modules at runtime even when the application source is TypeScript.

The code generator should be modular, but TypeScript should not duplicate the JavaScript emitter. The clean dependency direction is:

```text
validated message MIR
        ↓
shared ECMAScript module model/emitter
        ├── ESM JavaScript runtime modules
        ├── per-message virtual modules for Vite
        └── source maps and shared runtime helpers
                 ↓
        TypeScript surface layer
        ├── .d.ts declarations
        ├── argument overloads
        ├── generated namespace types
        └── optional .ts source rendering, if still useful
```

A practical crate/module split is:

- `linguini-codegen-ecmascript` or `linguini-codegen-js`: owns runtime semantics, expression emission, locale dispatch, formatter calls, imports/exports, per-message ESM modules, identifier allocation, source maps, and JavaScript rendering.
- A shared language-neutral `TypeModel` owns message signatures, named parameter-object shapes, namespace paths, enum/union types, optionality, overloads, documentation text, and source links. Neither the JavaScript nor TypeScript renderer may infer this information independently.
- The JavaScript type renderer emits JSDoc from `TypeModel`, including `@param`, `@returns`, imported/declared typedefs, and overload documentation for both positional and named-object calls.
- The TypeScript surface renderer emits `.d.ts` declarations from the same `TypeModel`. It must not duplicate runtime message emission or maintain a second semantic implementation.
- `plugins/vite`: requests exact per-message ESM modules from the shared backend and exposes them as virtual modules. It must not contain a second message compiler.

For the SvelteKit path, the preferred generated artifacts are ESM JavaScript with JSDoc plus TypeScript declarations:

```text
virtual:linguini/message/shop.main.title   -> JavaScript ESM + JSDoc
virtual:linguini/message/shop.cart.items   -> JavaScript ESM + JSDoc
.linguini/linguini.d.ts                    -> TypeScript API surface
```

The default contract should be:

- **JavaScript consumers:** receive ordinary ESM JavaScript annotated with JSDoc. Editors and `// @ts-check` can validate calls without requiring TypeScript source files.
- **TypeScript consumers:** execute exactly the same JavaScript runtime modules and receive generated `.d.ts` declarations for the complete nested `l` namespace and exact message signatures.
- **No separate TypeScript runtime emitter:** generating `.ts` implementation files is unnecessary for the primary SvelteKit/Vite path and would create another surface that can drift. If such output is added later for source inspection or another toolchain, it must be only a rendering view over the same ECMAScript module model and `TypeModel`.

This gives both JavaScript and TypeScript applications checking and autocomplete without maintaining parallel runtime generators. If physical files are requested for debugging or non-Vite use, the default should likewise be `.js` with JSDoc plus `.d.ts`.

Example JavaScript emission for a parameterized message should preserve both supported call forms in its JSDoc contract:

```js
/**
 * @overload
 * @param {number} count
 * @returns {string}
 */
/**
 * @overload
 * @param {{ count: number }} args
 * @returns {string}
 */
/**
 * @param {number | { count: number }} value
 * @returns {string}
 */
export function items(value) {
  const count = typeof value === "object" ? value.count : value;
  // Shared generated message implementation.
}
```

The corresponding declaration is rendered from the same signature model:

```ts
export interface CartItemsMessage {
  (count: number): string;
  (args: { count: number }): string;
}
```

Parameterless message modules should be typed as values, not zero-argument functions, in both surfaces. Their internal getter or reactive implementation remains an emitter detail; the public declaration and transformed source expression stay equivalent to `l.shop.main.title: string`.

The implementation should not achieve reuse through string post-processing such as “generate JavaScript and inject type annotations into the resulting text.” JavaScript runtime code, JSDoc, and TypeScript declarations should be rendered from the same structured ECMAScript/module representation plus the same `TypeModel`. Target-specific syntax belongs only at the final rendering boundary.

The SvelteKit-first configuration should become close to zero-config. In plugin mode, output location, ESM format, virtual-module namespace, framework adapter, and declaration generation can all be adapter defaults. Retain configuration only for stable product decisions that cannot be derived from SvelteKit/Vite or project sources.

Concrete repository cleanup locations include:

- `crates/linguini-config/src/model.rs`: remove `TypeScriptTargetConfig.module` and its validation.
- `crates/linguini-config/src/parser.rs`: remove the builder field, default, assignment arm, and parser tests for `targets.ts.module`.
- `crates/linguini-cli/src/project/io.rs`, CLI tests, `site/linguini.toml`, `docs/getting-started.md`, and `docs/web-sveltekit.md`: remove `module = "esm"`.
- `docs/reference.md`: remove the `esm | cjs` contract and state that generated web modules are ESM.
- Rename or internally split `linguini-codegen-ts` only when doing so reduces duplication; avoid a repository-wide rename that does not first establish the shared ECMAScript backend boundary.

### Preserve Linguini documentation comments through code generation

Documentation comments written in Linguini source must survive the entire compiler pipeline and appear on the generated JavaScript and TypeScript API. This is not a cosmetic formatter feature: the comments are part of the developer-facing contract presented by hover, completion, generated declarations, and editor navigation.

The repository already carries documentation through several syntax and IR nodes and emits it for some top-level declarations. The implementation is incomplete, however: the generated nested `MessageTree` stores no group documentation, grouped message properties are rendered without their message comments, and the current emitter flattens every source comment into a one-line `/** ... */` block. Recursive groups and per-message virtual modules would otherwise make more documentation disappear unless this is made an explicit invariant.

Required behavior:

- Preserve documentation on messages, recursive groups, enums, type aliases, forms, functions, and variables in lossless or normalized structured form through AST, semantic IR, `TypeModel`, and module emission.
- Use schema documentation as the canonical public API documentation. Locale implementation comments may additionally be retained on locale-specific generated internals, but they must not unpredictably replace the schema contract.
- Attach message documentation to the exact nested leaf in generated JSDoc and `.d.ts` output. For example:

```lgs
shop {
  cart {
    /// Number of products currently in the cart.
    items(count: Number)
  }
}
```

should produce equivalent documentation on both supported surfaces:

```js
/**
 * Number of products currently in the cart.
 * @overload
 * @param {number} count
 * @returns {string}
 */
export function items(count) { /* generated implementation */ }
```

```ts
interface CartItemsMessage {
  /** Number of products currently in the cart. */
  (count: number): string;
  /** Number of products currently in the cart. */
  (args: { count: number }): string;
}
```

- Preserve group documentation in nested namespace declarations so hovering `l.shop.main` explains the namespace, while hovering `l.shop.main.title` explains the message.
- Keep paragraphs and line breaks rather than concatenating every comment into unrelated one-line blocks. Escape `*/` and other generated-comment hazards without silently dropping text.
- Generate built-in `@param`, `@returns`, overload, and type metadata separately from source prose. Do not make users write backend-specific JSDoc tags in Linguini syntax.
- Define deterministic merging when several `///` lines precede one declaration and add golden tests for nested groups, multiline comments, Unicode, comment terminators, JSDoc emission, `.d.ts` emission, and editor hover.
- Preserve the same documentation identity in source maps or generated metadata so “go to definition” and hover can resolve back to the original Linguini declaration.

### Distribute the CLI/LSP independently and package the VS Code extension intentionally

A standalone npm distribution is useful, but it does not by itself make a VSIX dynamically install the correct executable. A VSIX is a self-contained archive. Installing an extension does not run `npm install`; production dependencies and binary assets must already have been included or copied into the package during `vsce package`. The current extension bundles its JavaScript with esbuild and explicitly excludes `node_modules/**`, so merely adding an npm dependency containing the CLI would not place that binary in the existing VSIX.

Use one release pipeline with three related artifacts:

1. **Native CLI packages for npm.** Publish a small JavaScript launcher such as `@linguini/cli` with a `bin` entry. It selects a platform package installed through `optionalDependencies`, following the established native-package pattern:

```text
@linguini/cli
@linguini/cli-linux-x64
@linguini/cli-linux-arm64
@linguini/cli-darwin-x64
@linguini/cli-darwin-arm64
@linguini/cli-win32-x64
@linguini/cli-win32-arm64
```

Each platform package contains one signed or checksummed native `linguini` executable. The launcher resolves the matching package and executes it. This gives users `npm install --save-dev @linguini/cli`, `npm install --global @linguini/cli`, or `npx linguini ...` without requiring Cargo. Package versions must be locked to the same compiler/LSP protocol version.

2. **Platform-specific desktop VSIX packages.** Build one VSIX per supported VS Code target and copy exactly one matching native binary from the corresponding npm platform package into an extension-owned directory such as `dist/server/linguini`. Package with `vsce package --target <platform>` and publish the matching target metadata to Open VSX. This is preferable to shipping every native binary in one universal VSIX: it is smaller, starts faster, and keeps native LSP behavior. The npm packages are the reusable binary supply source, but the VSIX build must vendor the selected executable explicitly.

3. **A universal WebAssembly fallback.** Compile the language-server core to WebAssembly and provide a browser-worker LSP transport for a universal extension build. This build can run on desktop and potentially VS Code for the Web, where child processes are unavailable. It should be a deliberate fallback or web variant, not assumed to be equivalent to the native CLI until filesystem, threading, cancellation, performance, and virtual-workspace behavior have conformance tests.

Recommended extension resolution order for desktop development:

1. An explicit `linguini.server.path` override.
2. A project-pinned local CLI from the workspace package manager, when explicitly enabled.
3. The binary bundled in the platform-specific extension.
4. A compatible CLI on `PATH` as a final fallback.

Every path must perform an LSP/compiler protocol handshake before use. A mismatched project CLI should produce a clear diagnostic rather than silently starting an incompatible server.

Do not download an executable on first activation by default. That creates proxy, offline, checksum, update, and supply-chain failure modes and makes extension behavior non-reproducible. If an opt-in downloader is ever added, it must verify a full cryptographic digest and signed release manifest before execution.

The native CLI package and the WASM server should share compiler, parser, analyzer, and protocol crates. They should differ only at the host boundary: native filesystem/process integration versus VS Code virtual filesystem, worker transport, and WASM-compatible services. Do not fork the language semantics into a separate JavaScript implementation.

### Preserve the nested API while making message usage bundler-native

The public API must preserve the intended nested syntax. Parameterless messages are values, while messages with runtime parameters are callable:

```ts
l.shop.main.title
l.shop.main.description
l.shop.cart.items(count)
```

The implementation must not materialize one monolithic runtime object containing every locale and message. Instead, `l` should act as a compile-time typed namespace. The Vite/Svelte transform resolves each statically known leaf access and injects a dependency on a module for that exact message.

For example, source code:

```svelte
<h1>{l.shop.main.title}</h1>
<p>{l.shop.cart.items(count)}</p>
```

may be lowered internally to code equivalent to:

```ts
import { message as __linguini_0 } from "virtual:linguini/message/shop.main.title";
import { message as __linguini_1 } from "virtual:linguini/message/shop.cart.items";

__linguini_0();
__linguini_1(count);
```

The internal call for a parameterless message is an implementation detail required for locale resolution and reactivity. It must never leak into the public API: users write `l.shop.main.title`, not `l.shop.main.title()`.

The target architecture is:

1. Generate recursive TypeScript declarations for the complete typed namespace, including `string` leaves for parameterless messages and callable leaves for parameterized messages.
2. Add a Svelte/TypeScript compile-time transform that recognizes static paths rooted at the generated `l` binding.
3. Rewrite each leaf access to a unique imported binding from a virtual or physical per-message ESM module.
4. Let Vite/Rollup use its ordinary module graph, tree-shaking, shared-chunk extraction, route code splitting, preload, client-navigation loading, and HMR. Linguini must not recreate those systems with a separate route manifest or public message-loading API.
5. Generate only the semantic dependencies required by that message module: referenced forms, functions, variables, aliases, formatter helpers, and necessary CLDR data.
6. Invalidate only affected virtual message modules when a schema or locale source changes.
7. Treat dynamic lookup such as `l[group][name]` as non-tree-shakable. Strict mode should reject it; an explicit escape hatch may import a declared namespace or message set and intentionally accept the larger bundle.
8. Keep a physical-module backend for non-Vite bundlers and debugging, but make both backends use the same single-message compiler API.

This gives the bundler a precise dependency graph:

```text
SvelteKit route
  -> imported components
     -> transformed Linguini message imports
        -> exact per-message ESM modules
```

Only messages referenced by the route and its imported components enter that route's chunks. Shared messages naturally become shared chunks. Unreferenced messages do not enter the browser build.

Locale splitting is a separate optimization from message tree-shaking. The first mandatory step is one ESM dependency per referenced message. If the product requirement is also to omit inactive locales from the initial browser payload, generate locale-specific message modules behind a framework-level dynamic import boundary. Vite must still own chunk creation and network loading; Linguini should only generate the import graph.

### Support both positional and named-object calls in generated JavaScript and TypeScript surfaces

The current generated API uses positional arguments and this syntax must remain supported:

```ts
l.shop.cart.items(count)
l.delivery(fruit, size, count)
```

Code generation should additionally expose a named-object overload without changing the Linguini source language or semantic IR:

```ts
l.shop.cart.items({ count })
l.delivery({ fruit, size, count })
```

Both forms must call the same generated implementation and produce identical output. This is a TypeScript/JavaScript code-generation feature, not a new message-language syntax feature.

Recommended declarations:

```ts
interface CartItemsMessage {
  (count: number): string;
  (args: { count: number }): string;
}

interface DeliveryMessage {
  (fruit: Fruit, size: Size, count: number): string;
  (args: { fruit: Fruit; size: Size; count: number }): string;
}
```

The nested API then exposes:

```ts
readonly items: CartItemsMessage;
readonly delivery: DeliveryMessage;
```

Runtime emission should normalize both calling conventions once at the generated function boundary. It should not duplicate the message body or locale dispatch logic. Generated parameter-object property order must be irrelevant, unknown keys should be rejected by TypeScript, and missing required keys must be type errors. The positional overload preserves compact calls and backward compatibility; the named-object overload improves readability and refactoring safety for messages with several parameters.

Parameterless messages remain values in both JavaScript and TypeScript:

```ts
l.shop.main.title
```

They must not acquire an empty call form merely for implementation convenience.

### Define dedented and raw multiline messages

Linguini needs first-class multiline message values rather than treating triple quotes as a lexer-only experiment. The ordinary multiline form should provide Kotlin `trimIndent()`-style normalization so source indentation can follow the surrounding declaration without becoming user-visible text:

```lgl
receipt = """
    Order {order_id}
      {item_count} items
    Thank you.
"""
```

The semantic value should be equivalent to:

```text
Order {order_id}
  {item_count} items
Thank you.
```

Required semantics for the ordinary `"""` form:

- Parse the block as a real `TextPattern`, including newlines and placeholders; do not pass multiline newlines through the global trivia-removal path.
- Treat an immediately blank first line and a blank final line before the closing delimiter as structural and remove them.
- Compute and remove the common leading whitespace prefix of all nonblank content lines. This provides deterministic trim-indent behavior while preserving relative indentation.
- Preserve internal blank lines, relative indentation, trailing spaces, placeholders, and message references after dedenting.
- Specify line-ending behavior. The recommended normal form canonicalizes CRLF and CR to `\n` in the semantic value for reproducible output.
- Define tab behavior explicitly rather than measuring indentation differently across formatter, parser, and editor implementations. A longest-common-leading-whitespace-prefix rule is deterministic even for mixed tabs and spaces.

A separate raw multiline form must preserve content without edge trimming or dedenting:

```lgl
wire_format = raw"""  leading spaces
    indentation is data
trailing spaces stay here  """
```

For `raw"""..."""`, every character between the opening and closing delimiters is message content, including leading and trailing newlines, spaces, tabs, blank lines, and original line endings. The raw mode controls whitespace preservation; placeholders such as `{name}` should remain active unless the language later introduces a distinct literal/no-interpolation string form. Literal brace escaping must therefore still be specified separately.

Implementation requirements:

- Introduce an explicit text-block mode in syntax and semantic IR, for example `TextBlockMode::Dedented | Raw`, instead of inferring behavior later from token shapes.
- Make the lexer emit multiline content, including newline spans, as text-mode tokens that the locale parser consumes directly. `TripleQuote` delimiters must be consumed by a dedicated parser production.
- Apply dedenting once during parsing or lowering and store the resulting semantic text plus source mapping. Codegen, analyzer, formatter, LSP hover, and tests must not independently reimplement trimming.
- The formatter may reindent ordinary dedented blocks if semantic equivalence is proven, but it must never alter bytes inside a raw block. Raw blocks should be treated as formatter-protected regions.
- Preserve source spans across removed indentation so diagnostics and go-to-definition remain aligned with the original source.
- Add golden tests for empty blocks, one-line triple-quoted blocks, placeholders across lines, nested indentation, internal blank lines, all-whitespace lines, tabs, CRLF, leading/trailing newlines, trailing spaces, delimiter-like quote sequences, Unicode, formatter idempotence, and JS/JSDoc/`.d.ts` output.

This should be a language feature shared by every backend, not a JavaScript-specific helper named `trimIndent`. The normalization belongs in the compiler semantic pipeline; generated applications should receive the final string and should not run a trimming function at runtime.

### Support arbitrarily nested message groups

The language currently models a group as a name plus a flat list of messages. It should instead support recursive groups, for example:

```lgl
shop {
  main {
    title = Shop
    checkout = Checkout
  }
  local {
    title = Local shop
  }
}
```

The corresponding schema should support the same recursive shape. Canonical message identities become paths such as `shop.main.title`, while generated TypeScript can expose the natural typed access path `l.shop.main.title`.

Required implementation changes:

- Replace `MessageGroup { messages: Vec<Message...> }` with a recursive declaration tree containing both messages and child groups.
- Use one canonical qualified-name representation across syntax, schema, locale indexes, analyzer, IR, LSP, CLI stubs, and code generation.
- Detect duplicate path segments and collisions between a message and a group, for example a message `shop.main` conflicting with a group at the same path.
- Preserve a source span and source ID for every path segment so rename, completion, references, and diagnostics remain precise.
- Generate nested declaration types and runtime objects without copying unrelated symbols into every namespace.
- Allow arbitrary finite nesting in the language semantics. The implementation should still enforce a defensive parser/analyzer depth limit or use an iterative traversal to prevent stack exhaustion on hostile input.
- Define empty-group behavior explicitly. Empty groups may be useful in schemas as reserved namespaces, but should normally be rejected in locale files because they produce no translations.

This recursive namespace model must feed the compile-time namespace transform. Static leaf access imports one exact message module; an explicitly declared namespace bundle is only an escape hatch for dynamic access and must not be the default.

## Prioritized remediation plan

### P0 — before any public production release

- Remove every network, `git`, and `remove_dir_all` operation from the procedural macro. Generate CLDR data offline in the release pipeline, publish a verified artifact with a complete SHA-256 and manifest, and consume it through a read-only data crate.
- Introduce `SafeOutputRoot`: canonicalize project and output paths; reject absolute paths, `..`, symlink escapes, and overlap with project, schema, or locale roots; make the writer operate only on manifest-owned files.
- Correct the IR form model and code generator: branches and values must be distinct explicit types. Add an end-to-end corpus that emits JavaScript/JSDoc plus `.d.ts`, typechecks the public API, and executes runtime assertions.
- Make validated IR the only codegen entry point, using a `ValidatedProject` or sealed constructor produced after schema, locale, type, and coverage checks.
- Fix CLI severity routing: every `DiagnosticSeverity::Error` must block `check` and `build`.
- Correct locale symbol indexes and replace overrides atomically across symbol kinds.
- Disable destructive lexical workspace rename until symbol IDs and a reference index exist.
- Reject path traversal in test support and replace the helper with `tempfile::TempDir`.

### P1 — language correctness

- Make the current target contract explicitly SvelteKit/ESM-first: remove CJS from documentation and remove `targets.ts.module` from the configuration model, parser, templates, and tests.
- Freeze a language specification for v0.1: decide whether inline `fn` remains in scope, implement the documented lint/type contracts that are retained, and remove CJS from the reference, examples, configuration, and roadmap. Do not implement CJS during the SvelteKit-first phase.
- Replace one-level message groups with recursive namespace nodes and carry canonical qualified paths through the complete compiler and tooling pipeline.
- Implement dedicated dedented and raw multiline text-block productions. Preserve multiline newlines through parsing, perform trim-indent normalization once in the semantic pipeline, and make raw blocks formatter-protected exact content.
- Split AST nodes for Reference versus Call and Form versus Fn, and preserve source IDs and spans through code generation.
- Build one symbol table and type checker with namespaces, overload, arity and type checking, alias-cycle detection, and exhaustive multidimensional pattern checking.
- Replace duplicate walkers and reference checkers with one semantic database shared by CLI and LSP.
- Define locale canonicalization and fallback through CLDR parent locales and use one algorithm in every layer.

### P2 — runtime and tooling

- Establish one shared ECMAScript backend for all JavaScript runtime emission and per-message ESM modules, plus one shared `TypeModel`. Render JSDoc for JavaScript and `.d.ts` declarations for TypeScript from that model; do not maintain parallel JS and TS runtime implementations.
- Preserve Linguini `///` documentation through semantic IR and the shared `TypeModel`; emit it on exact JSDoc exports, nested namespace properties, and `.d.ts` overloads, including recursive group hover documentation.
- Emit ordinary and raw multiline messages from the same parsed text-block IR; do not perform whitespace normalization in generated JavaScript or in framework runtime helpers.
- Sanitize TypeScript identifiers and filenames with a collision table and reserved-word handling.
- Remove eager imports of every locale. Where inactive-locale splitting is enabled, express locale boundaries as bundler-visible ESM dynamic imports so Vite owns chunk creation and loading; deduplicate formatter data per locale and do not add a CJS branch.
- Make the generated `l` API a compile-time typed namespace: parameterless leaves are values, parameterized leaves retain positional calls, and the Vite/Svelte transform rewrites static leaf access to exact per-message virtual-module imports. Let Vite/Rollup derive route and shared chunks from its own module graph.
- Add generated positional and named-object overloads, for example both `items(count)` and `items({ count })`, without changing the Linguini language syntax or duplicating runtime implementations.
- Replace the flat `WebConfig` with the exact nested namespaces defined above: `[web.routing]`, `[web.locale]`, `[web.cookie]`, `[web.local_storage]`, `[web.links]`, `[web.routes]`, and `[web.switch_route]`. Delete `prefix_default_locale`; use `locale_prefix = "always" | "except-default" | "never"`, defaulting to `"except-default"`.
- Generate locale-source, cookie, local-storage, link-transform/runtime-link, route-matcher, and switch-route transport modules only when selected. Compile one shared `LocaleSwitchPlan` and use it for browser `switchLocale`/current `setLocale`, direct `localizeHref` links, and the optional generated switch route. The route must have no persistence mode of its own: it writes cookies only when `"cookie"` is selected, rewrites the return URL only when `"path"` is selected, and rejects configurations whose selected behavior cannot be reproduced server-side, such as local-storage-only pathless switching. Derive origin, base path, and trailing-slash behavior from SvelteKit/request context.
- Replace regex HTML rewriting with a parser or adapter-level implementation, or constrain the API to a safe DOM and server-transform contract.
- Make the LSP incremental, versioned, cancellable, and namespace-aware; do not perform synchronous full-tree I/O in request handlers.
- Publish a standalone `@linguini/cli` npm launcher backed by platform-specific optional packages, then vendor the selected native package into target-specific desktop VSIX builds. Add a separately tested WASM/browser-worker LSP build as the universal/web fallback; do not rely on extension installation to run npm or download executables.
- Replace pseudo-TOML readers with `serde` plus `toml`, schema validation, and migration diagnostics.
- Add atomic writes, hash-based optimistic concurrency, and conflict-aware quick fixes.

### P3 — evidence of quality

- CI matrix: Rust 1.76 plus stable, Linux/Windows/macOS, offline build, and `cargo package`.
- Fuzz and property tests for parser, formatter, plural rules, and path discovery, including idempotence and no-panic invariants.
- Conformance corpus: documentation examples → parse → analyze → IR → generate JavaScript/JSDoc and `.d.ts` → typecheck → Node/browser/SSR assertions, including dedented and byte-preserving raw multiline fixtures.
- Differential ICU and CLDR tests for plural, number, date, currency, and locale fallback.
- Real Vite-version matrix and VS Code integration tests for native target-specific VSIX builds, npm-installed CLI resolution, protocol mismatch handling, and the WASM/browser-worker LSP fallback.
- `cargo-deny` or audit checks, pinned GitHub Action SHAs, reproducible npm/pnpm packaging, and synchronized release versions.

## Final consistency recheck

The final pass re-read the complete report and rechecked the repository paths relevant to the added product requirements.

- The numbered inventory remains contiguous from 1 through 369 with no duplicate numbers. Severity totals still match the executive summary: 22 critical, 166 high, 162 medium, and 19 low.
- The API examples now consistently use values for parameterless messages (`l.shop.main.title`), preserve the current positional API for parameterized messages (`items(count)`), and add the named-object form only as a codegen overload (`items({ count })`).
- The web direction is consistently SvelteKit-first and ESM-only. CJS is described only as documentation/configuration to remove, not as a feature to implement.
- JavaScript is the single runtime output model. JSDoc and TypeScript `.d.ts` declarations are rendered from one shared type model rather than from separate runtime compilers.
- The bundler owns tree-shaking, route code splitting, shared chunks, preload, and network loading. Linguini contributes exact per-message ESM dependencies through the compile-time `l` transform; it does not introduce a public route loader or duplicate Vite's module graph.
- Recursive message groups, source documentation propagation, npm/native/WASM CLI and LSP distribution, multiline text modes, and the concrete nested web configuration are represented in both the architecture sections and the prioritized remediation plan.
- The web configuration now has exact enum values and migration rules. `prefix_default_locale` is removed; `locale_prefix` defaults to `"except-default"` and supports `"never"`. The report reuses the existing `localizeHref` and defines one shared `LocaleSwitchPlan` for browser `switchLocale`/current `setLocale`, direct links, and the optional generated switch-route transport. The route inherits configured persistence and routing behavior instead of being cookie-specific, while local-storage-only pathless no-JavaScript persistence is rejected as impossible.
- A previously invalid documentation example using a nonexistent `message` keyword was corrected to the actual schema declaration style.
- The multiline implementation was checked directly: the lexer recognizes triple-quoted delimiters, but the locale parser does not consume `TripleQuote`, and the common trivia filter discards newlines. The report now states this explicitly and defines the required dedented and raw semantics.

This consistency pass does not remove the original limitation: Rust tests and Clippy were not executable in the audit environment, so dynamic confirmation of Rust behavior still requires a Rust-enabled CI run.

## Overall assessment of the concept

The concept—“schema as a typed public contract, locale as programmable grammar, and a generated native API”—is strong. The core problem is not the idea itself, but that the project is simultaneously attempting to be a language, compiler, CLDR runtime, web router, formatter, LSP, and framework integration before a single semantic core has stabilized. Consequently, its promises are phrased at the level of a mature compiler toolchain while the implementation remains a set of only partially aligned passes.

A rational architecture for the next iteration is: `recursive lossless syntax tree with documentation and explicit dedented/raw text blocks → one-time text normalization with source maps → project database/source map → name resolution → typed HIR → exhaustive pattern IR → validated MIR → shared TypeModel + ECMAScript single-message emitter → JSDoc renderer + TypeScript declaration renderer → compile-time namespace transform → ordinary Vite/SvelteKit module graph`. Configuration and discovery, locale fallback, and CLDR data should become separate deterministic services. ESM is the only current web module format; CJS and additional framework/language targets should be deferred. Framework adapters should derive request URL, origin, base path, and trailing-slash behavior from SvelteKit/Vite, while a centralized nested TOML policy selects only the required routing, locale-source, persistence, link, exclusion, and switch-route modules. The CLI and LSP should query the same project database rather than independently repeating parse, analyze, and resolve logic.
