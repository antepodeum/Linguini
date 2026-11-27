# CLDR data architecture

`linguini-cldr` consumes the versioned Rust tables under `src/data/generated`.
`linguini-cldr-macros` is an offline maintainer tool, rather than a procedural macro or
consumer build dependency. Regeneration is explicit; ordinary builds use the existing tables.

The source is CLDR JSON 48.2.0, commit
`bb334e8d6250c9363e957e131bf7e6d08ec72f91`. The generator verifies the checkout identity,
package versions, locale coverage, and consumed-input SHA-256 before writing output. The
manifest records the source identity, input profile, artifact hash, and coverage. Generation
and `--check` use the same deterministic path without fetching upstream data.

The calendar profile covers Gregorian, Buddhist, ROC, and Japanese data: 3,060 locale/calendar
pairs, contextual symbols, eras, skeletons, day periods, week rules, and GMT formats. Equal
symbol tables share storage. Hawaiian retains its explicit unsupported Roman-month exclusion.

Week preference validity comes from the pinned territory/subdivision containment and alias
metadata. The regular sets contain 257 regions and 5,027 subdivisions, excluding grouping
regions and obsolete aliases. The common extension-alias table applies 147 subdivision
replacements to `rg` and `sd`. Obsolete subdivision IDs without replacement metadata are not
covered by this profile.

The maintainer generator owns table construction in `src/week.rs` and alias lowering in
`src/locale.rs`. The shared resolver is `linguini-cldr/src/locale/week.rs`; native previews and
code generators use its effective first weekday and minimum days. `linguini-codegen` owns
week arithmetic and finite wording plans. Application JavaScript receives specialized
arithmetic and text choices, without these validity tables or a CLDR interpreter.
