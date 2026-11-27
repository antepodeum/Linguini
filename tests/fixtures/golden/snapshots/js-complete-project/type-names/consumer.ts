import { normalizeMessageArgs as enumArgs } from "./enum/shared.js";
import { normalizeMessageArgs as aliasArgs } from "./alias/shared.js";
import type { Record as EnumRecord } from "./enum/shared.js";
import type { Record as AliasRecord } from "./alias/shared.js";
import type { GeneratedDecimal, GeneratedNumeric } from "./numeric/shared.js";

const fruit: EnumRecord = "apple";
const label: AliasRecord = "label";
enumArgs([{ value: fruit }], ["value"]);
aliasArgs([{ value: label }], ["value"]);
// @ts-expect-error Source enum values remain finite.
const unknownFruit: EnumRecord = "banana";
// @ts-expect-error Source alias keeps its string type.
const numericLabel: AliasRecord = 42;
void [unknownFruit, numericLabel];

const kind: GeneratedDecimal = "first";
const measure: GeneratedNumeric = "second";
// @ts-expect-error User types keep their finite enum values.
const numericKind: GeneratedDecimal = 42;
// @ts-expect-error The source numeric-named enum stays an enum.
const unknownMeasure: GeneratedNumeric = "third";
void [kind, measure, numericKind, unknownMeasure];
