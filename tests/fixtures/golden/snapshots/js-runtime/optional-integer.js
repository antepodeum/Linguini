/** @typedef {number | bigint | string} __lgl_Numeric */

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatLeadingDecimal(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  const integer = rounded.integer === "0" ? "" : rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-" : "";
  return prefix + formatted;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatLeadingOptionalDecimal(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo2FractionDigits(decimal);
  const integer = rounded.integer === "0" ? "" : rounded.integer;
  const fraction = rounded === decimal ? rounded.fraction.padEnd(1, "0") : rounded.fraction.slice(0, 1) + rounded.fraction.slice(1).replace(/0+$/, "");
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-" : "";
  return prefix + formatted;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatLeadingDecimalAffixes(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo2FractionDigits(decimal);
  const integer = rounded.integer === "0" ? "" : rounded.integer;
  const fraction = rounded.fraction.padEnd(2, "0");
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-pre" : "pre";
  const suffix = "tail";
  return prefix + formatted + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatOnlyDecimal(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-" : "";
  return prefix + integer + ".";
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatLeadingGrouping(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  if (integer.length > 3) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += "," + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const prefix = decimal.negative ? "-" : "";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatQuotedGroupingPrefix(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  if (integer.length > 2) {
    const firstGroup = integer.length % 2 || 2;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 2) {
      grouped += "," + integer.slice(offset, offset + 2);
    }
    integer = grouped;
  }
  const prefix = decimal.negative ? "-#" : "#";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatOptionalInteger(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  const integer = rounded.integer === "0" ? "" : rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-" : "";
  return prefix + formatted;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatOptionalIntegerFraction(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo2FractionDigits(decimal);
  const integer = rounded.integer === "0" ? "" : rounded.integer;
  const fraction = rounded === decimal ? rounded.fraction.padEnd(1, "0") : rounded.fraction.slice(0, 1) + rounded.fraction.slice(1).replace(/0+$/, "");
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-" : "";
  return prefix + formatted;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatRequiredInteger(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo2FractionDigits(decimal);
  const integer = rounded.integer;
  const fraction = rounded === decimal ? rounded.fraction.padEnd(1, "0") : rounded.fraction.slice(0, 1) + rounded.fraction.slice(1).replace(/0+$/, "");
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-" : "";
  return prefix + formatted;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatRequiredIntegerFixed(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  const integer = rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-" : "";
  return prefix + formatted;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatOptionalFraction(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo2FractionDigits(decimal);
  const integer = rounded.integer;
  const fraction = rounded === decimal ? rounded.fraction : rounded.fraction.replace(/0+$/, "");
  const formatted = fraction ? integer + "." + fraction : integer;
  const prefix = decimal.negative ? "-" : "";
  return prefix + formatted;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatInteger(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-" : "";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatVisibleSeparator(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-" : "";
  return prefix + integer + ".";
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatPaddedInteger(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  const integer = rounded.integer.padStart(2, "0");
  const fraction = rounded.fraction.padEnd(1, "0");
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-" : "";
  return prefix + formatted;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatGroupedOptionalInteger(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  let integer = rounded.integer === "0" ? "" : rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  if (integer.length > 3) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += "," + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-" : "";
  return prefix + formatted;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatOptionalIntegerPercent(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
  const rounded = roundTo1FractionDigits(decimal);
  const integer = rounded.integer === "0" ? "" : rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-" : "";
  const suffix = "%";
  return prefix + formatted + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatAffixedOptionalInteger(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  const integer = rounded.integer === "0" ? "" : rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-v0:" : "v0:";
  const suffix = ":v1";
  return prefix + formatted + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatWidthOptionalInteger(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  const integer = rounded.integer === "0" ? "" : rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-" : "";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 2 : 3) - formatted.length));
  return padding + prefix + formatted;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatArabicOptionalInteger(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  const integer = rounded.integer === "0" ? "" : rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  const formatted = integer + "٫" + fraction;
  const localized = formatted.replace(/\d/g, localizeDigit);
  const prefix = decimal.negative ? "؜-v0:" : "v0:";
  const suffix = ":v1";
  return prefix + localized + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatAdlamOptionalInteger(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  const integer = rounded.integer === "0" ? "" : rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  const formatted = integer + "." + fraction;
  const localized = formatted.replace(/\d/g, __lgl_digit2);
  const prefix = decimal.negative ? "-v0:" : "v0:";
  const suffix = ":v1";
  return prefix + localized + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatChakmaOptionalInteger(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  const integer = rounded.integer === "0" ? "" : rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  const formatted = integer + "." + fraction;
  const localized = formatted.replace(/\d/g, __lgl_digit3);
  const prefix = decimal.negative ? "-v0:" : "v0:";
  const suffix = ":v1";
  return prefix + localized + suffix;
}

/**
 * @typedef {object} __lgl_Decimal
 * @property {boolean} negative
 * @property {string} integer
 * @property {string} fraction
 */
const MAX_GENERATED_DECIMAL_DIGITS = 8192;

/**
 * @param {__lgl_Numeric} value
 * @returns {__lgl_Decimal | undefined}
 */
function parseGeneratedDecimal(value) {
  if (typeof value === "number" && !Number.isFinite(value)) return undefined;
  const source = String(value);
  if (typeof value === "bigint" || Number.isSafeInteger(value)) {
    const hasSign = source[0] === "-";
    const integer = hasSign ? source.slice(1) : source;
    if (integer.length > MAX_GENERATED_DECIMAL_DIGITS) throwInvalidNumber();
    return { negative: hasSign || (value === 0 && Object.is(value, -0)), integer, fraction: "" };
  }
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

  /** @type {string} */
  let integer;
  /** @type {string} */
  let fraction;
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
    negative: match[1] === "-",
    integer,
    fraction,
  };
}

/**
 * @param {__lgl_Decimal} decimal
 * @returns {__lgl_Decimal}
 */
function scaleBy100(decimal) {
  const source = `${decimal.integer}${decimal.fraction}`;
  const decimalPosition = decimal.integer.length + 2;
  if (source.length + 2 > MAX_GENERATED_DECIMAL_DIGITS) throwInvalidNumber();
  const expanded = source.padEnd(decimalPosition, "0");
  const integer = expanded.slice(0, decimalPosition).replace(/^0+(?=\d)/, "") || "0";
  const fraction = expanded.slice(decimalPosition).replace(/0+$/, "");
  return { negative: decimal.negative, integer, fraction };
}

/**
 * @param {__lgl_Decimal} decimal
 * @returns {{ integer: string, fraction: string }}
 */
function roundTo0FractionDigits(decimal) {
  if (decimal.fraction.length <= 0) return decimal;
  if (decimal.fraction[0] < "5") return { integer: decimal.integer, fraction: "" };
  const digits = (BigInt(decimal.integer) + 1n).toString();
  return { integer: digits, fraction: "" };
}

/**
 * @param {__lgl_Decimal} decimal
 * @returns {{ integer: string, fraction: string }}
 */
function roundTo1FractionDigits(decimal) {
  if (decimal.fraction.length <= 1) return decimal;
  const fraction = decimal.fraction.slice(0, 1);
  if (decimal.fraction[1] < "5") return { integer: decimal.integer, fraction };
  const digits = (BigInt(`${decimal.integer}${fraction}`) + 1n).toString().padStart(2, "0");
  return { integer: digits.slice(0, -1), fraction: digits.slice(-1) };
}

/**
 * @param {__lgl_Decimal} decimal
 * @returns {{ integer: string, fraction: string }}
 */
function roundTo2FractionDigits(decimal) {
  if (decimal.fraction.length <= 2) return decimal;
  const fraction = decimal.fraction.slice(0, 2);
  if (decimal.fraction[2] < "5") return { integer: decimal.integer, fraction };
  const digits = (BigInt(`${decimal.integer}${fraction}`) + 1n).toString().padStart(3, "0");
  return { integer: digits.slice(0, -2), fraction: digits.slice(-2) };
}

/** @returns {never} */
function throwInvalidNumber() {
  throw new RangeError("Linguini: invalid numeric value");
}
/**
 * @param {string} digit
 * @returns {string}
 */
function __lgl_digit2(digit) {
  return String.fromCodePoint(digit.charCodeAt(0) + 125216);
}
/**
 * @param {string} digit
 * @returns {string}
 */
function __lgl_digit3(digit) {
  return String.fromCodePoint(digit.charCodeAt(0) + 69894);
}
/**
 * @param {string} digit
 * @returns {string}
 */
function localizeDigit(digit) {
  return "٠١٢٣٤٥٦٧٨٩"[digit.charCodeAt(0) - 48];
}
