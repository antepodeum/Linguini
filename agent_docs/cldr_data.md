# CLDR data architecture

`linguini-cldr` consumes the versioned Rust tables under `src/data/generated`.
`linguini-cldr-macros` is an offline maintainer tool, rather than a procedural macro or
consumer build dependency. Regeneration is explicit; ordinary builds use the existing tables.

The source is CLDR JSON 48.2.0, commit
`bb334e8d6250c9363e957e131bf7e6d08ec72f91`. The generator verifies the checkout identity,
package versions, locale coverage, and consumed-input SHA-256 before writing output. The
manifest records the source identity, input profile, artifact hash, and coverage. Generation
and `--check` use the same deterministic path without fetching upstream data.

Numeric patterns compile in `linguini-cldr-macros/src/formatting.rs`. Subpattern scanning keeps
escaped padding characters opaque; padding is valid only at the four affix/number boundaries.
Rounding digits contribute to mandatory widths as ordinary zero placeholders do. Padding uses
the shared `linguini_cldr::NumberPadding` model consumed by the common number-format plan.
Significant precision starts at the first `@`; earlier hashes only position grouping. Required
digits must precede optional digits in fractions and follow them in fixed integer patterns.
Significant digits form one contiguous required run. Empty integer groups, grouped scientific
mantissas, and mixed significant/fixed decimal syntax are rejected by the offline compiler.
The skeleton scanner reserves `+` for an exponent sign only immediately after `E`; a later plus
belongs to the localized affix. A final empty negative subpattern preserves implicit negation.
Affix quoting is checked before symbol substitution, preventing numeric metacharacters outside
the skeleton from silently becoming literals. Leading decimal/grouping forms require additional
model support and currently fail. Padding escapes remain opaque to these checks.
Affix lexing and symbol substitution live in `formatting/affix.rs`. Literal text and currency
slots use the shared `NumberAffixPart` model with owned text during compilation and borrowed
text in the checked tables. Compact `NumberAffix` values distinguish literals, single slots,
and composite fragments. The common number plan resolves only explicit slots to currency text;
quoted currency marks remain opaque literals. Wider currency display markers fail explicitly
until their semantics are implemented.

Profile v8 added decimal-separator visibility in the shared numeric pattern metadata. A
trailing decimal in the mantissa sets `decimal_separator_always`; quoted punctuation and optional
fraction placeholders do not. The common number plan carries the choice into native previews
and build-time template selection. All 3,438 pinned pattern parts use `false`; that addition
otherwise preserved the previous compiler artifact. Profile v9 adds typed affixes. Flattening
all 6,876 affixes back to text reproduces the v8 artifact byte for byte; all other metadata is
unchanged. Pinned literal fragments contain no currency marks, preserving existing substitutions.

The calendar profile covers Gregorian, Buddhist, ROC, and Japanese data: 3,060 locale/calendar
pairs, contextual symbols, eras, skeletons, day periods, week rules, and GMT formats. Equal
symbol tables share storage. Hawaiian retains its explicit unsupported Roman-month exclusion.

Week preference validity comes from the pinned territory/subdivision containment and alias
metadata. The regular sets contain 257 regions and 5,027 subdivisions, excluding grouping
regions and obsolete aliases. They are checked for exact agreement with the bundled CLDR 48.2
validity file, which also supplies 626 deprecated subdivisions and 257 unknown subdivision IDs.
The common extension-alias table applies 147 subdivision replacements to `rg` and `sd` before
validity checks. `sd` accepts regular/deprecated IDs; `rg` accepts regular/unknown IDs.

The bundled source is `linguini-cldr-macros/data/subdivision.xml`, copied unchanged from CLDR
`release-48-2`, commit `11299982335beb974c1c63c45265184e759c0f41`. Its provenance, byte count,
and verified SHA-256 are recorded under `inputs.bundled_sources` in the generated manifest.
The existing input-tree counts and hash describe the CLDR JSON checkout. XML parsing belongs
only to the offline maintainer tool; `linguini-cldr` retains no normal dependencies.

The maintainer generator owns table construction in `src/week.rs`, validity parsing in
`src/validity.rs`, and alias lowering in `src/locale.rs`. The shared resolver is `linguini-cldr/src/locale/week.rs`; native previews and
code generators use its effective first weekday and minimum days. `linguini-codegen` owns
week arithmetic and finite wording plans. Application JavaScript receives specialized
arithmetic and text choices, without these validity tables or a CLDR interpreter.
