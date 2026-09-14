type GeneratedNumeric = number | bigint | string;
type GeneratedCurrencyFormatterOptions = { code?: string; accounting?: "true" | "false" };
type GeneratedDateFormatterOptions = { style?: "full" | "long" | "medium" | "short" };

export function formatNumber(value: GeneratedNumeric): string {
  return formatGeneratedNumber(value, "", "", undefined, undefined, 1, 0, 3, 3, undefined, ",", " ", "0123456789", 1, undefined, undefined, undefined, undefined, false, undefined, undefined, undefined);
}

export function formatCurrency(
  value: GeneratedNumeric,
  fractionDigits: number,
  roundingIncrement: number,
  options: GeneratedCurrencyFormatterOptions = {},
): string {
  const symbol = currencySymbol(options.code ?? "USD");
  if (options.accounting === "true") {
    return formatGeneratedNumber(value, "", " " + symbol + "", undefined, undefined, 1, 2, 2, 3, undefined, ",", " ", "0123456789", 1, undefined, undefined, undefined, undefined, false, undefined, undefined, undefined, fractionDigits, fractionDigits, roundingIncrement);
  }
  return formatGeneratedNumber(value, "", " " + symbol + "", undefined, undefined, 1, 2, 2, 3, undefined, ",", " ", "0123456789", 1, undefined, undefined, undefined, undefined, false, undefined, undefined, undefined, fractionDigits, fractionDigits, roundingIncrement);
}

function currencySymbol(currency: string): string {
  return new Intl.NumberFormat("ru", { style: "currency", currency })
    .formatToParts(0)
    .find((part) => part.type === "currency")?.value ?? currency;
}

export function formatDate(
  value: Date | number | string,
  options: GeneratedDateFormatterOptions = {},
): string {
  const date = coerceDate(value);
  switch (options.style ?? "medium") {
    case "full":
      return localizeGeneratedDigits(["воскресенье", "понедельник", "вторник", "среда", "четверг", "пятница", "суббота"][date.getUTCDay()] + ", " + String(date.getUTCDate()) + " " + ["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"][date.getUTCMonth()] + " " + String(date.getUTCFullYear()) + " " + "г" + ".", "0123456789");
    case "long":
      return localizeGeneratedDigits(String(date.getUTCDate()) + " " + ["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"][date.getUTCMonth()] + " " + String(date.getUTCFullYear()) + " " + "г" + ".", "0123456789");
    case "short":
      return localizeGeneratedDigits(padNumber(date.getUTCDate(), 2) + "." + padNumber(date.getUTCMonth() + 1, 2) + "." + String(date.getUTCFullYear()), "0123456789");
    default:
      return localizeGeneratedDigits(String(date.getUTCDate()) + " " + ["янв.", "февр.", "мар.", "апр.", "мая", "июн.", "июл.", "авг.", "сент.", "окт.", "нояб.", "дек."][date.getUTCMonth()] + " " + String(date.getUTCFullYear()) + " " + "г" + ".", "0123456789");
  }
}

type GeneratedDecimal = { negative: boolean; integer: string; fraction: string };
const MAX_GENERATED_DECIMAL_DIGITS = 8192;

function formatGeneratedNumber(
  value: GeneratedNumeric,
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
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleGeneratedDecimal(decimal, scale);
  const effectiveMinFractionDigits = minFractionDigitsOverride ?? minFractionDigits;
  const effectiveMaxFractionDigits = maxFractionDigitsOverride ?? maxFractionDigits;
  let exponent: number | undefined;
  if (exponentDigits !== undefined) {
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
  if (exponent !== undefined) {
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

function parseGeneratedDecimal(value: GeneratedNumeric): GeneratedDecimal | undefined {
  if (typeof value === "number" && !Number.isFinite(value)) return undefined;
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

  let integer: string;
  let fraction: string;
  if (decimalPosition <= 0) {
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

function scaleGeneratedDecimal(decimal: GeneratedDecimal, scale: number): GeneratedDecimal {
  const shift = scale === 1 ? 0 : scale === 100 ? 2 : scale === 1000 ? 3 : -1;
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

function scientificGeneratedDecimal(
  decimal: GeneratedDecimal,
): { decimal: GeneratedDecimal; exponent: number } {
  if (/^0+$/.test(decimal.integer) && !/[1-9]/.test(decimal.fraction)) {
    return { decimal: { ...decimal, integer: "0", fraction: "" }, exponent: 0 };
  }
  let source: string;
  let exponent: number;
  if (/[1-9]/.test(decimal.integer)) {
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

function roundGeneratedSignificant(
  decimal: GeneratedDecimal,
  maximumDigits: number,
): { integer: string; fraction: string } {
  if (
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

function padGeneratedSignificant(
  integer: string,
  fraction: string,
  minimumDigits: number,
): string {
  if (!Number.isSafeInteger(minimumDigits) || minimumDigits < 1) throwInvalidNumber();
  const combined = `${integer}${fraction}`;
  const first = combined.search(/[1-9]/);
  const current = first < 0 ? 1 : combined.length - first;
  return fraction.padEnd(fraction.length + Math.max(0, minimumDigits - current), "0");
}

function padGeneratedNumber(
  prefix: string,
  number: string,
  suffix: string,
  character: string | undefined,
  width: number | undefined,
  position: number | undefined,
): string {
  if (character === undefined || width === undefined || position === undefined) {
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

function roundGeneratedDecimal(
  decimal: GeneratedDecimal,
  fractionDigits: number,
  roundingIncrement: number | string,
): { integer: string; fraction: string } {
  const quantumSource = String(roundingIncrement || 1);
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
  let roundUp: boolean;
  if (discarded === "") {
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

function trimOptionalFractionDigits(fraction: string, minDigits: number): string {
  while (fraction.length > minDigits && fraction.endsWith("0")) {
    fraction = fraction.slice(0, -1);
  }
  return fraction;
}

function groupIntegerDigits(
  integer: string,
  primaryGroupSize: number | undefined,
  secondaryGroupSize: number | undefined,
  groupSymbol: string,
): string {
  if (!primaryGroupSize || integer.length <= primaryGroupSize) return integer;
  const groups: string[] = [];
  let end = integer.length;
  let groupSize = primaryGroupSize;
  while (end > 0) {
    const start = Math.max(0, end - groupSize);
    groups.unshift(integer.slice(start, end));
    end = start;
    groupSize = secondaryGroupSize ?? primaryGroupSize;
  }
  return groups.join(groupSymbol);
}

function throwInvalidNumber(): never {
  throw new RangeError("Linguini: invalid numeric value");
}

function padNumber(value: number, length: number): string {
  return String(value).padStart(length, "0");
}

function coerceDate(value: Date | number | string): Date {
  let date: Date;
  if (value instanceof Date) {
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

function createUTCDate(year: number, month: number, day: number): Date {
  const date = new Date(0);
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

function throwInvalidDate(): never {
  throw new RangeError("Linguini: invalid date value");
}

function localizeGeneratedDigits(value: string, digits: string): string {
  if (digits === "0123456789") return value;
  const symbols = Array.from(digits);
  if (symbols.length !== 10) throw new RangeError("Linguini: invalid numbering system");
  return value.replace(/\d/g, (digit) => symbols[digit.charCodeAt(0) - 48]);
}


export function pluralRu(value: number | bigint | string): string {
  if (value === "zero" || value === "one" || value === "two" || value === "few" || value === "many" || value === "other") return value;
  const operands = pluralOperands(value);
  if ((pluralOperandMatches(operands.v, undefined, false, [[0n, 0n]]) && pluralOperandMatches(operands.i, 10n, false, [[1n, 1n]]) && !pluralOperandMatches(operands.i, 100n, false, [[11n, 11n]]))) return "one";
  if ((pluralOperandMatches(operands.v, undefined, false, [[0n, 0n]]) && pluralOperandMatches(operands.i, 10n, false, [[2n, 4n]]) && !pluralOperandMatches(operands.i, 100n, false, [[12n, 14n]]))) return "few";
  if ((pluralOperandMatches(operands.v, undefined, false, [[0n, 0n]]) && pluralOperandMatches(operands.i, 10n, false, [[0n, 0n]])) || (pluralOperandMatches(operands.v, undefined, false, [[0n, 0n]]) && pluralOperandMatches(operands.i, 10n, false, [[5n, 9n]])) || (pluralOperandMatches(operands.v, undefined, false, [[0n, 0n]]) && pluralOperandMatches(operands.i, 100n, false, [[11n, 14n]]))) return "many";
  return "other";
}

type PluralOperand = { integer: bigint; hasFraction: boolean };
const MAX_PLURAL_DECIMAL_DIGITS = 8192;

function pluralOperands(value: number | bigint | string) {
  if (typeof value === "number" && !Number.isFinite(value)) throwInvalidPluralNumber();
  const source = String(value).trim();
  // Canonical CLDR input uses lowercase c for a non-negative compact-decimal
  // exponent. Native numbers may stringify with JavaScript's signed e/E
  // scientific notation; that notation describes only their numeric value and
  // must not set the CLDR c/e operands.
  const match = typeof value === "number"
    ? /^([+-]?)(\d+)(?:\.(\d*))?(?:[eE]([+-]?\d+))?$/.exec(source)
    : /^([+-]?)(\d+)(?:\.(\d*))?(?:c(\d+))?$/.exec(source);
  if (!match) throwInvalidPluralNumber();

  const whole = match[2];
  const sourceFraction = match[3] ?? "";
  const exponent = Number(match[4] ?? "0");
  if (
    !Number.isSafeInteger(exponent) ||
    Math.abs(exponent) > MAX_PLURAL_DECIMAL_DIGITS ||
    (match[4]?.length ?? 0) > MAX_PLURAL_DECIMAL_DIGITS ||
    whole.length + sourceFraction.length > MAX_PLURAL_DECIMAL_DIGITS
  ) {
    throwInvalidPluralNumber();
  }

  const digits = `${whole}${sourceFraction}` || "0";
  const decimalPosition = whole.length + exponent;
  const expandedLength = decimalPosition <= 0
    ? -decimalPosition + digits.length
    : Math.max(decimalPosition, digits.length);
  if (expandedLength > MAX_PLURAL_DECIMAL_DIGITS) throwInvalidPluralNumber();

  let integer: string;
  let fraction: string;
  if (decimalPosition <= 0) {
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
  const trimmedFraction = fraction.replace(/0+$/, "");
  const compactExponent = typeof value === "string" ? match[4] ?? "0" : "0";
  const operand = (digits: string, hasFraction = false): PluralOperand => ({
    integer: BigInt(digits || "0"),
    hasFraction,
  });

  return {
    n: operand(integer, /[1-9]/.test(fraction)),
    i: operand(integer),
    v: operand(String(fraction.length)),
    w: operand(String(trimmedFraction.length)),
    f: operand(fraction),
    t: operand(trimmedFraction),
    c: operand(compactExponent),
    e: operand(compactExponent),
  };
}

function pluralOperandMatches(
  value: PluralOperand,
  modulo: bigint | undefined,
  allowFraction: boolean,
  ranges: readonly (readonly [bigint, bigint])[],
): boolean {
  const integer = modulo === undefined || modulo === 0n
    ? value.integer
    : value.integer % modulo;
  return ranges.some(([start, end]) => {
    if (!allowFraction) {
      return !value.hasFraction && integer >= start && integer <= end;
    }
    return integer >= start &&
      (integer < end || (integer === end && !value.hasFraction));
  });
}

function throwInvalidPluralNumber(): never {
  throw new RangeError("Linguini: invalid plural numeric value");
}
