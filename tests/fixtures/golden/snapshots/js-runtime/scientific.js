/** @typedef {number | bigint | string} GeneratedNumeric */

/**
 * @param {GeneratedNumeric} value
 * @returns {string}
 */
export function scientificLatin(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  let exponent;
  ({ decimal, exponent } = scientificGeneratedDecimal(decimal));
  let rounded = roundTo3FractionDigits(decimal);
  if (rounded.integer.length > 1) {
    exponent++;
    rounded = { integer: "1", fraction: "000" };
  }
  const integer = rounded.integer;
  const fraction = rounded.fraction.replace(/0+$/, "");
  const formatted = fraction ? integer + "." + fraction : integer;
  const exponentDigits = String(Math.abs(exponent)).padStart(2, "0");
  const scientific = formatted + "E" + (exponent < 0 ? "-" : "+") + exponentDigits;
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @param {GeneratedNumeric} value
 * @returns {string}
 */
export function scientificPersian(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  let exponent;
  ({ decimal, exponent } = scientificGeneratedDecimal(decimal));
  let rounded = roundTo3FractionDigits(decimal);
  if (rounded.integer.length > 1) {
    exponent++;
    rounded = { integer: "1", fraction: "000" };
  }
  const integer = rounded.integer;
  const fraction = rounded.fraction.replace(/0+$/, "");
  const formatted = fraction ? integer + "٫" + fraction : integer;
  const localized = formatted.replace(/\d/g, (digit) => "۰۱۲۳۴۵۶۷۸۹"[digit.charCodeAt(0) - 48]);
  const exponentDigits = String(Math.abs(exponent)).padStart(2, "0");
  const scientific = localized + "×۱۰^" + (exponent < 0 ? "‎−" : "‎+") + exponentDigits.replace(/\d/g, (digit) => "۰۱۲۳۴۵۶۷۸۹"[digit.charCodeAt(0) - 48]);
  const prefix = decimal.negative ? "‎−" : "";
  return prefix + scientific;
}

/**
 * @param {GeneratedNumeric} value
 * @returns {string}
 */
export function scientificAdlam(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  let exponent;
  ({ decimal, exponent } = scientificGeneratedDecimal(decimal));
  let rounded = roundTo3FractionDigits(decimal);
  if (rounded.integer.length > 1) {
    exponent++;
    rounded = { integer: "1", fraction: "000" };
  }
  const integer = rounded.integer;
  const fraction = rounded.fraction.replace(/0+$/, "");
  const formatted = fraction ? integer + "." + fraction : integer;
  const localized = formatted.replace(/\d/g, (digit) => String.fromCodePoint(digit.charCodeAt(0) + 125216));
  const exponentDigits = String(Math.abs(exponent)).padStart(2, "0");
  const scientific = localized + "𞤉" + (exponent < 0 ? "-" : "+") + exponentDigits.replace(/\d/g, (digit) => String.fromCodePoint(digit.charCodeAt(0) + 125216));
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @param {GeneratedNumeric} value
 * @returns {string}
 */
export function scientificMixed(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  let exponent;
  ({ decimal, exponent } = scientificGeneratedDecimal(decimal));
  let rounded = roundTo3FractionDigits(decimal);
  if (rounded.integer.length > 1) {
    exponent++;
    rounded = { integer: "1", fraction: "000" };
  }
  const integer = rounded.integer;
  const fraction = rounded.fraction.replace(/0+$/, "");
  const formatted = fraction ? integer + "." + fraction : integer;
  const localized = formatted.replace(/\d/g, (digit) => {
    switch (digit.charCodeAt(0)) {
      case 48: return "0";
      case 49: return "𝟙";
      case 50: return "2";
      case 51: return "𝟛";
      case 52: return "4";
      case 53: return "𝟝";
      case 54: return "6";
      case 55: return "𝟟";
      case 56: return "8";
      default: return "𝟡";
    }
  });
  const exponentDigits = String(Math.abs(exponent)).padStart(2, "0");
  const scientific = localized + "E" + (exponent < 0 ? "-" : "+") + exponentDigits.replace(/\d/g, (digit) => {
    switch (digit.charCodeAt(0)) {
      case 48: return "0";
      case 49: return "𝟙";
      case 50: return "2";
      case 51: return "𝟛";
      case 52: return "4";
      case 53: return "𝟝";
      case 54: return "6";
      case 55: return "𝟟";
      case 56: return "8";
      default: return "𝟡";
    }
  });
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @typedef {object} GeneratedDecimal
 * @property {boolean} negative
 * @property {string} integer
 * @property {string} fraction
 */
const MAX_GENERATED_DECIMAL_DIGITS = 8192;

/**
 * @param {GeneratedNumeric} value
 * @returns {GeneratedDecimal | undefined}
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
 * @param {GeneratedDecimal} decimal
 * @returns {{ decimal: GeneratedDecimal, exponent: number }}
 */
function scientificGeneratedDecimal(decimal) {
  if (decimal.integer === "0" && decimal.fraction === "") {
    return { decimal: { ...decimal, integer: "0", fraction: "" }, exponent: 0 };
  }
  /** @type {string} */
  let source;
  /** @type {number} */
  let exponent;
  if (decimal.integer !== "0") {
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

/**
 * @param {GeneratedDecimal} decimal
 * @returns {{ integer: string, fraction: string }}
 */
function roundTo3FractionDigits(decimal) {
  const fraction = decimal.fraction.slice(0, 3).padEnd(3, "0");
  if (decimal.fraction.length <= 3 || decimal.fraction[3] < "5") return { integer: decimal.integer, fraction };
  const digits = (BigInt(`${decimal.integer}${fraction}`) + 1n).toString().padStart(4, "0");
  return { integer: digits.slice(0, -3), fraction: digits.slice(-3) };
}

/** @returns {never} */
function throwInvalidNumber() {
  throw new RangeError("Linguini: invalid numeric value");
}
