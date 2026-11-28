/** @typedef {number | bigint | string} __lgl_Numeric */

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_code_prefix(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-USD" : "USD";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_code_suffix(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-" : "";
  const suffix = "USD";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_quoted_then_code(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-¤¤USD" : "¤¤USD";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_symbol_then_code(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-$/USD" : "$/USD";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_code_then_symbol(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-USD/$" : "USD/$";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_negative_code(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "(USD" : "USD";
  const suffix = decimal.negative ? ")" : "";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_quoted_code(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-¤¤" : "¤¤";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_literal_prefix(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-¤" : "¤";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_symbol_prefix(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-$" : "$";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_literal_then_symbol(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-¤$" : "¤$";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_symbol_then_literal(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-$¤" : "$¤";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_literal_suffix(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-$" : "$";
  const suffix = "¤";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_mixed_suffix(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-" : "";
  const suffix = "$¤";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_repeated_slots(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-$¤$" : "$¤$";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_quoted_apostrophe(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-a'¤$" : "a'¤$";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_negative_affix(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "(¤$" : "¤$";
  const suffix = decimal.negative ? ")" : "";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function affix_literal_dollar(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  const integer = rounded.integer;
  const prefix = decimal.negative ? "-$&¤" : "$&¤";
  return prefix + integer;
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
 * @returns {{ integer: string, fraction: string }}
 */
function roundTo0FractionDigits(decimal) {
  if (decimal.fraction.length <= 0) return decimal;
  if (decimal.fraction[0] < "5") return { integer: decimal.integer, fraction: "" };
  const digits = (BigInt(decimal.integer) + 1n).toString();
  return { integer: digits, fraction: "" };
}

/** @returns {never} */
function throwInvalidNumber() {
  throw new RangeError("Linguini: invalid numeric value");
}
