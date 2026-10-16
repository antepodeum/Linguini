//! Numeric, date, and digit helper bodies with explicit target type annotations.

use crate::ecmascript::EcmaScriptTarget;

use super::runtime_code::RuntimeCode;

pub(super) fn number_formatter_helpers(target: EcmaScriptTarget) -> String {
    let mut code = RuntimeCode::default();
    code.typed(
        r#"type GeneratedDecimal = { negative: boolean; integer: string; fraction: string };
"#,
        r#"/**
 * @typedef {object} GeneratedDecimal
 * @property {boolean} negative
 * @property {string} integer
 * @property {string} fraction
 */
"#,
    );
    code.shared(
        r#"const MAX_GENERATED_DECIMAL_DIGITS = 8192;

"#,
    );
    code.typed(
        "",
        r#"/**
 * @param {GeneratedNumeric} value
 * @param {string} prefix
 * @param {string} suffix
 * @param {string | undefined} negativePrefix
 * @param {string | undefined} negativeSuffix
 * @param {number} minIntegerDigits
 * @param {number} minFractionDigits
 * @param {number} maxFractionDigits
 * @param {number | undefined} primaryGroupSize
 * @param {number | undefined} secondaryGroupSize
 * @param {string} decimalSymbol
 * @param {string} groupSymbol
 * @param {string} digits
 * @param {number} scale
 * @param {number | undefined} minSignificantDigits
 * @param {number | undefined} maxSignificantDigits
 * @param {string | undefined} patternRoundingIncrement
 * @param {number | undefined} exponentDigits
 * @param {boolean} exponentSignAlways
 * @param {string | undefined} paddingCharacter
 * @param {number | undefined} formatWidth
 * @param {number | undefined} paddingPosition
 * @param {number} [minFractionDigitsOverride]
 * @param {number} [maxFractionDigitsOverride]
 * @param {number} [roundingIncrementOverride]
 * @returns {string}
 */
"#,
    );
    code.shared(
        r#"function formatGeneratedNumber(
"#,
    );
    code.typed(
        r#"  value: GeneratedNumeric,
  prefix: string,
  suffix: string,
  negativePrefix: string | undefined,
  negativeSuffix: string | undefined,
  minIntegerDigits: number,
  minFractionDigits: number,
  maxFractionDigits: number,
  primaryGroupSize: number | undefined,
  secondaryGroupSize: number | undefined,
  decimalSymbol: string,
  groupSymbol: string,
  digits: string,
  scale: number,
  minSignificantDigits: number | undefined,
  maxSignificantDigits: number | undefined,
  patternRoundingIncrement: string | undefined,
  exponentDigits: number | undefined,
  exponentSignAlways: boolean,
  paddingCharacter: string | undefined,
  formatWidth: number | undefined,
  paddingPosition: number | undefined,
  minFractionDigitsOverride?: number,
  maxFractionDigitsOverride?: number,
  roundingIncrementOverride?: number,
): string {
"#,
        r#"  value,
  prefix,
  suffix,
  negativePrefix,
  negativeSuffix,
  minIntegerDigits,
  minFractionDigits,
  maxFractionDigits,
  primaryGroupSize,
  secondaryGroupSize,
  decimalSymbol,
  groupSymbol,
  digits,
  scale,
  minSignificantDigits,
  maxSignificantDigits,
  patternRoundingIncrement,
  exponentDigits,
  exponentSignAlways,
  paddingCharacter,
  formatWidth,
  paddingPosition,
  minFractionDigitsOverride,
  maxFractionDigitsOverride,
  roundingIncrementOverride,
) {
"#,
    );
    code.shared(
        r#"  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleGeneratedDecimal(decimal, scale);
  const effectiveMinFractionDigits = minFractionDigitsOverride ?? minFractionDigits;
  const effectiveMaxFractionDigits = maxFractionDigitsOverride ?? maxFractionDigits;
"#,
    );
    code.typed(
        r#"  let exponent: number | undefined;
"#,
        r#"  /** @type {number | undefined} */
  let exponent;
"#,
    );
    code.shared(
        r#"  if (exponentDigits !== undefined) {
    ({ decimal, exponent } = scientificGeneratedDecimal(decimal));
  }
  let rounded = maxSignificantDigits === undefined
    ? roundGeneratedDecimal(
        decimal,
        effectiveMaxFractionDigits,
        roundingIncrementOverride ?? patternRoundingIncrement ?? 0,
      )
    : roundGeneratedSignificant(decimal, maxSignificantDigits);
  if (exponent !== undefined && rounded.integer.length > 1) {
    const shift = rounded.integer.length - 1;
    exponent += shift;
    rounded = {
      integer: rounded.integer[0],
      fraction: `${rounded.integer.slice(1)}${rounded.fraction}`,
    };
  }
  let integer = rounded.integer.padStart(minIntegerDigits, "0");
  let fraction = trimOptionalFractionDigits(
    rounded.fraction,
    effectiveMinFractionDigits,
  );
  if (minSignificantDigits !== undefined) {
    fraction = padGeneratedSignificant(integer, fraction, minSignificantDigits);
  }

  integer = groupIntegerDigits(integer, primaryGroupSize, secondaryGroupSize, groupSymbol);
  let ascii = fraction ? `${integer}${decimalSymbol}${fraction}` : integer;
  if (exponent !== undefined && exponentDigits !== undefined) {
    const sign = exponent < 0 ? "-" : exponentSignAlways ? "+" : "";
    ascii += `E${sign}${String(Math.abs(exponent)).padStart(exponentDigits, "0")}`;
  }
  const formatted = localizeGeneratedDigits(ascii, digits);
  const effectivePrefix = decimal.negative ? negativePrefix ?? `-${prefix}` : prefix;
  const effectiveSuffix = decimal.negative ? negativeSuffix ?? suffix : suffix;
  return padGeneratedNumber(
    effectivePrefix,
    formatted,
    effectiveSuffix,
    paddingCharacter,
    formatWidth,
    paddingPosition,
  );
}

"#,
    );
    code.typed(
        r#"function parseGeneratedDecimal(value: GeneratedNumeric): GeneratedDecimal | undefined {
"#,
        r#"/**
 * @param {GeneratedNumeric} value
 * @returns {GeneratedDecimal | undefined}
 */
function parseGeneratedDecimal(value) {
"#,
    );
    code.shared(
        r#"  if (typeof value === "number" && !Number.isFinite(value)) return undefined;
  const negativeZero = typeof value === "number" && Object.is(value, -0);
  const source = String(value);
  const match = /^([+-]?)(\d*)(?:\.(\d*))?(?:[eE]([+-]?\d+))?$/.exec(source);
  if (!match || (match[2] === "" && (match[3] ?? "") === "")) throwInvalidNumber();

  const whole = match[2];
  const fractional = match[3] ?? "";
  const exponent = Number(match[4] ?? "0");
  if (
    !Number.isSafeInteger(exponent) ||
    Math.abs(exponent) > MAX_GENERATED_DECIMAL_DIGITS ||
    (match[4]?.length ?? 0) > MAX_GENERATED_DECIMAL_DIGITS ||
    whole.length + fractional.length > MAX_GENERATED_DECIMAL_DIGITS
  ) {
    throwInvalidNumber();
  }

  const digits = `${whole}${fractional}` || "0";
  const decimalPosition = whole.length + exponent;
  const expandedLength = decimalPosition <= 0
    ? -decimalPosition + digits.length
    : Math.max(decimalPosition, digits.length);
  if (expandedLength > MAX_GENERATED_DECIMAL_DIGITS) throwInvalidNumber();

"#,
    );
    code.typed(
        r#"  let integer: string;
  let fraction: string;
"#,
        r#"  /** @type {string} */
  let integer;
  /** @type {string} */
  let fraction;
"#,
    );
    code.shared(
        r#"  if (decimalPosition <= 0) {
    integer = "0";
    fraction = `${"0".repeat(-decimalPosition)}${digits}`;
  } else if (decimalPosition >= digits.length) {
    integer = `${digits}${"0".repeat(decimalPosition - digits.length)}`;
    fraction = "";
  } else {
    integer = digits.slice(0, decimalPosition);
    fraction = digits.slice(decimalPosition);
  }
  integer = integer.replace(/^0+(?=\d)/, "");
  fraction = fraction.replace(/0+$/, "");
  return {
    negative: match[1] === "-" || negativeZero,
    integer,
    fraction,
  };
}

"#,
    );
    code.typed(r#"function scaleGeneratedDecimal(decimal: GeneratedDecimal, scale: number): GeneratedDecimal {
"#, r#"/**
 * @param {GeneratedDecimal} decimal
 * @param {number} scale
 * @returns {GeneratedDecimal}
 */
function scaleGeneratedDecimal(decimal, scale) {
"#);
    code.shared(
        r#"  const shift = scale === 1 ? 0 : scale === 100 ? 2 : scale === 1000 ? 3 : -1;
  if (shift < 0) throwInvalidNumber();
  if (shift === 0) return decimal;
  const source = `${decimal.integer}${decimal.fraction}`;
  const decimalPosition = decimal.integer.length + shift;
  if (source.length + shift > MAX_GENERATED_DECIMAL_DIGITS) throwInvalidNumber();
  const expanded = source.padEnd(decimalPosition, "0");
  const integer = expanded.slice(0, decimalPosition).replace(/^0+(?=\d)/, "") || "0";
  const fraction = expanded.slice(decimalPosition).replace(/0+$/, "");
  return { negative: decimal.negative, integer, fraction };
}

"#,
    );
    code.typed(
        r#"function scientificGeneratedDecimal(
  decimal: GeneratedDecimal,
): { decimal: GeneratedDecimal; exponent: number } {
"#,
        r#"/**
 * @param {GeneratedDecimal} decimal
 * @returns {{ decimal: GeneratedDecimal, exponent: number }}
 */
function scientificGeneratedDecimal(decimal) {
"#,
    );
    code.shared(
        r#"  if (/^0+$/.test(decimal.integer) && !/[1-9]/.test(decimal.fraction)) {
    return { decimal: { ...decimal, integer: "0", fraction: "" }, exponent: 0 };
  }
"#,
    );
    code.typed(
        r#"  let source: string;
  let exponent: number;
"#,
        r#"  /** @type {string} */
  let source;
  /** @type {number} */
  let exponent;
"#,
    );
    code.shared(
        r#"  if (/[1-9]/.test(decimal.integer)) {
    source = `${decimal.integer}${decimal.fraction}`;
    exponent = decimal.integer.length - 1;
  } else {
    const first = decimal.fraction.search(/[1-9]/);
    source = decimal.fraction.slice(first);
    exponent = -first - 1;
  }
  return {
    decimal: {
      negative: decimal.negative,
      integer: source[0],
      fraction: source.slice(1).replace(/0+$/, ""),
    },
    exponent,
  };
}

"#,
    );
    code.typed(
        r#"function roundGeneratedSignificant(
  decimal: GeneratedDecimal,
  maximumDigits: number,
): { integer: string; fraction: string } {
"#,
        r#"/**
 * @param {GeneratedDecimal} decimal
 * @param {number} maximumDigits
 * @returns {{ integer: string, fraction: string }}
 */
function roundGeneratedSignificant(decimal, maximumDigits) {
"#,
    );
    code.shared(
        r#"  if (
    !Number.isSafeInteger(maximumDigits) ||
    maximumDigits < 1 ||
    maximumDigits > MAX_GENERATED_DECIMAL_DIGITS
  ) {
    throwInvalidNumber();
  }
  const combined = `${decimal.integer}${decimal.fraction}`;
  const first = combined.search(/[1-9]/);
  if (first < 0) return { integer: "0", fraction: "" };
  const cut = first + maximumDigits;
  if (cut >= decimal.integer.length) {
    return roundGeneratedDecimal(decimal, cut - decimal.integer.length, 0);
  }

  const kept = decimal.integer.slice(0, cut);
  const discarded = `${decimal.integer.slice(cut)}${decimal.fraction}`;
  const rounded = BigInt(kept) + (discarded[0] >= "5" ? 1n : 0n);
  return {
    integer: `${rounded}${"0".repeat(decimal.integer.length - cut)}`,
    fraction: "",
  };
}

"#,
    );
    code.typed(
        r#"function padGeneratedSignificant(
  integer: string,
  fraction: string,
  minimumDigits: number,
): string {
"#,
        r#"/**
 * @param {string} integer
 * @param {string} fraction
 * @param {number} minimumDigits
 * @returns {string}
 */
function padGeneratedSignificant(integer, fraction, minimumDigits) {
"#,
    );
    code.shared(
        r#"  if (!Number.isSafeInteger(minimumDigits) || minimumDigits < 1) throwInvalidNumber();
  const combined = `${integer}${fraction}`;
  const first = combined.search(/[1-9]/);
  const current = first < 0 ? 1 : combined.length - first;
  return fraction.padEnd(fraction.length + Math.max(0, minimumDigits - current), "0");
}

"#,
    );
    code.typed(
        r#"function padGeneratedNumber(
  prefix: string,
  number: string,
  suffix: string,
  character: string | undefined,
  width: number | undefined,
  position: number | undefined,
): string {
"#,
        r#"/**
 * @param {string} prefix
 * @param {string} number
 * @param {string} suffix
 * @param {string | undefined} character
 * @param {number | undefined} width
 * @param {number | undefined} position
 * @returns {string}
 */
function padGeneratedNumber(prefix, number, suffix, character, width, position) {
"#,
    );
    code.shared(
        r#"  if (character === undefined || width === undefined || position === undefined) {
    return `${prefix}${number}${suffix}`;
  }
  if ([...character].length !== 1 || !Number.isSafeInteger(width) || width < 0) {
    throwInvalidNumber();
  }
  const padding = character.repeat(Math.max(0, width - [...`${prefix}${number}${suffix}`].length));
  switch (position) {
    case 0: return `${padding}${prefix}${number}${suffix}`;
    case 1: return `${prefix}${padding}${number}${suffix}`;
    case 2: return `${prefix}${number}${padding}${suffix}`;
    case 3: return `${prefix}${number}${suffix}${padding}`;
    default: throwInvalidNumber();
  }
}

"#,
    );
    code.typed(
        r#"function roundGeneratedDecimal(
  decimal: GeneratedDecimal,
  fractionDigits: number,
  roundingIncrement: number | string,
): { integer: string; fraction: string } {
"#,
        r#"/**
 * @param {GeneratedDecimal} decimal
 * @param {number} fractionDigits
 * @param {number | string} roundingIncrement
 * @returns {{ integer: string, fraction: string }}
 */
function roundGeneratedDecimal(decimal, fractionDigits, roundingIncrement) {
"#,
    );
    code.shared(
        r#"  const quantumSource = String(roundingIncrement || 1);
  if (
    !Number.isSafeInteger(fractionDigits) ||
    fractionDigits < 0 ||
    fractionDigits > MAX_GENERATED_DECIMAL_DIGITS ||
    !/^\d+$/.test(quantumSource) ||
    quantumSource.length > MAX_GENERATED_DECIMAL_DIGITS
  ) {
    throwInvalidNumber();
  }

  const keptFraction = decimal.fraction.slice(0, fractionDigits).padEnd(fractionDigits, "0");
  const discarded = decimal.fraction.slice(fractionDigits);
  const scaled = BigInt(`${decimal.integer}${keptFraction}` || "0");
  const quantum = BigInt(quantumSource);
  const remainder = scaled % quantum;
"#,
    );
    code.typed(
        r#"  let roundUp: boolean;
"#,
        r#"  /** @type {boolean} */
  let roundUp;
"#,
    );
    code.shared(
        r#"  if (discarded === "") {
    roundUp = remainder * 2n >= quantum;
  } else {
    const denominator = 10n ** BigInt(discarded.length);
    const exactRemainder = remainder * denominator + BigInt(discarded);
    roundUp = exactRemainder * 2n >= quantum * denominator;
  }
  const rounded = (scaled / quantum + (roundUp ? 1n : 0n)) * quantum;
  const digits = rounded.toString().padStart(fractionDigits + 1, "0");
  return fractionDigits === 0
    ? { integer: digits, fraction: "" }
    : {
        integer: digits.slice(0, -fractionDigits),
        fraction: digits.slice(-fractionDigits),
      };
}

"#,
    );
    code.typed(
        r#"function trimOptionalFractionDigits(fraction: string, minDigits: number): string {
"#,
        r#"/**
 * @param {string} fraction
 * @param {number} minDigits
 * @returns {string}
 */
function trimOptionalFractionDigits(fraction, minDigits) {
"#,
    );
    code.shared(
        r#"  while (fraction.length > minDigits && fraction.endsWith("0")) {
    fraction = fraction.slice(0, -1);
  }
  return fraction;
}

"#,
    );
    code.typed(
        r#"function groupIntegerDigits(
  integer: string,
  primaryGroupSize: number | undefined,
  secondaryGroupSize: number | undefined,
  groupSymbol: string,
): string {
"#,
        r#"/**
 * @param {string} integer
 * @param {number | undefined} primaryGroupSize
 * @param {number | undefined} secondaryGroupSize
 * @param {string} groupSymbol
 * @returns {string}
 */
function groupIntegerDigits(integer, primaryGroupSize, secondaryGroupSize, groupSymbol) {
"#,
    );
    code.shared(
        r#"  if (!primaryGroupSize || integer.length <= primaryGroupSize) return integer;
"#,
    );
    code.typed(
        r#"  const groups: string[] = [];
"#,
        r#"  /** @type {string[]} */
  const groups = [];
"#,
    );
    code.shared(
        r#"  let end = integer.length;
  let groupSize = primaryGroupSize;
  while (end > 0) {
    const start = Math.max(0, end - groupSize);
    groups.unshift(integer.slice(start, end));
    end = start;
    groupSize = secondaryGroupSize ?? primaryGroupSize;
  }
  return groups.join(groupSymbol);
}

"#,
    );
    code.typed(
        r#"function throwInvalidNumber(): never {
"#,
        r#"/** @returns {never} */
function throwInvalidNumber() {
"#,
    );
    code.shared(
        r#"  throw new RangeError("Linguini: invalid numeric value");
}
"#,
    );
    code.typed("\n", "");
    code.render(target)
}

pub(super) fn digit_formatter_helper(target: EcmaScriptTarget) -> String {
    let mut code = RuntimeCode::default();
    code.typed(
        r#"function localizeGeneratedDigits(value: string, digits: string): string {
"#,
        r#"/**
 * @param {string} value
 * @param {string} digits
 * @returns {string}
 */
function localizeGeneratedDigits(value, digits) {
"#,
    );
    code.shared(
        r#"  if (digits === "0123456789") return value;
  const symbols = Array.from(digits);
  if (symbols.length !== 10) throw new RangeError("Linguini: invalid numbering system");
  return value.replace(/\d/g, (digit) => symbols[digit.charCodeAt(0) - 48]);
}
"#,
    );
    code.typed("\n", "");
    code.render(target)
}

pub(super) fn date_formatter_helpers(target: EcmaScriptTarget) -> String {
    let mut code = RuntimeCode::default();
    code.typed(
        r#"function padNumber(value: number, length: number): string {
"#,
        r#"/**
 * @param {number} value
 * @param {number} length
 * @returns {string}
 */
function padNumber(value, length) {
"#,
    );
    code.shared(
        r#"  return String(value).padStart(length, "0");
}

"#,
    );
    code.typed(
        r#"function coerceDate(value: Date | number | string): Date {
  let date: Date;
"#,
        r#"/**
 * @param {Date | number | string} value
 * @returns {Date}
 */
function coerceDate(value) {
  /** @type {Date} */
  let date;
"#,
    );
    code.shared(r#"  if (value instanceof Date) {
    date = value;
  } else if (typeof value === "string") {
    const dateOnly = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
    if (dateOnly) {
      const year = Number(dateOnly[1]);
      const month = Number(dateOnly[2]);
      const day = Number(dateOnly[3]);
      date = createUTCDate(year, month, day);
    } else {
      const dateTime =
        /^(\d{4})-(\d{2})-(\d{2})T\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?(?:Z|[+-]\d{2}:\d{2})?$/.exec(value);
      if (!dateTime) throwInvalidDate();
      createUTCDate(Number(dateTime[1]), Number(dateTime[2]), Number(dateTime[3]));
      const hasTimeZone = /(?:Z|[+-]\d{2}:\d{2})$/.test(value);
      date = new Date(hasTimeZone ? value : `${value}Z`);
    }
  } else {
    date = new Date(value);
  }
  if (!Number.isFinite(date.getTime())) {
    throwInvalidDate();
  }
  return date;
}

"#);
    code.typed(
        r#"function createUTCDate(year: number, month: number, day: number): Date {
"#,
        r#"/**
 * @param {number} year
 * @param {number} month
 * @param {number} day
 * @returns {Date}
 */
function createUTCDate(year, month, day) {
"#,
    );
    code.shared(
        r#"  const date = new Date(0);
  date.setUTCHours(0, 0, 0, 0);
  date.setUTCFullYear(year, month - 1, day);
  if (
    date.getUTCFullYear() !== year ||
    date.getUTCMonth() !== month - 1 ||
    date.getUTCDate() !== day
  ) {
    throwInvalidDate();
  }
  return date;
}

"#,
    );
    code.typed(
        r#"function throwInvalidDate(): never {
"#,
        r#"/** @returns {never} */
function throwInvalidDate() {
"#,
    );
    code.shared(
        r#"  throw new RangeError("Linguini: invalid date value");
}
"#,
    );
    code.typed("\n", "");
    code.render(target)
}
