/** @typedef {number | bigint | string} __lgl_Numeric */

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatNumber(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo3FractionDigits(decimal);
  let integer = rounded.integer;
  const fraction = rounded === decimal ? rounded.fraction : rounded.fraction.replace(/0+$/, "");
  if (integer.length > 3) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += "," + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const formatted = fraction ? integer + "." + fraction : integer;
  const prefix = decimal.negative ? "-" : "";
  return prefix + formatted;
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
function roundTo3FractionDigits(decimal) {
  if (decimal.fraction.length <= 3) return decimal;
  const fraction = decimal.fraction.slice(0, 3);
  if (decimal.fraction[3] < "5") return { integer: decimal.integer, fraction };
  const digits = (BigInt(`${decimal.integer}${fraction}`) + 1n).toString().padStart(4, "0");
  return { integer: digits.slice(0, -3), fraction: digits.slice(-3) };
}

/** @returns {never} */
function throwInvalidNumber() {
  throw new RangeError("Linguini: invalid numeric value");
}
