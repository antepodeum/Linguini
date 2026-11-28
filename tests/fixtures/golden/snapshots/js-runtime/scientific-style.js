/** @typedef {number | bigint | string} __lgl_Numeric */

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_en(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "." + fraction : integer;
  const exponentDigits = String(Math.abs(exponent));
  const scientific = formatted + "E" + (exponent < 0 ? "-" : "") + exponentDigits;
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_fr(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "," + fraction : integer;
  const exponentDigits = String(Math.abs(exponent));
  const scientific = formatted + "E" + (exponent < 0 ? "-" : "") + exponentDigits;
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_ru(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "," + fraction : integer;
  const exponentDigits = String(Math.abs(exponent));
  const scientific = formatted + "E" + (exponent < 0 ? "-" : "") + exponentDigits;
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_ar(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "." + fraction : integer;
  const exponentDigits = String(Math.abs(exponent));
  const scientific = formatted + "E" + (exponent < 0 ? "‎-" : "") + exponentDigits;
  const prefix = decimal.negative ? "‎-" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_ar_EG(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "٫" + fraction : integer;
  const localized = formatted.replace(/\d/g, localizeDigit);
  const exponentDigits = String(Math.abs(exponent));
  const scientific = localized + "أس" + (exponent < 0 ? "؜-" : "") + exponentDigits.replace(/\d/g, localizeDigit);
  const prefix = decimal.negative ? "؜-" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_fa(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "٫" + fraction : integer;
  const localized = formatted.replace(/\d/g, __lgl_digit2);
  const exponentDigits = String(Math.abs(exponent));
  const scientific = localized + "×۱۰^" + (exponent < 0 ? "‎−" : "") + exponentDigits.replace(/\d/g, __lgl_digit2);
  const prefix = decimal.negative ? "‎−" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_sv(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "," + fraction : integer;
  const exponentDigits = String(Math.abs(exponent));
  const scientific = formatted + "×10^" + (exponent < 0 ? "−" : "") + exponentDigits;
  const prefix = decimal.negative ? "−" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_ee(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "." + fraction : integer;
  const exponentDigits = String(Math.abs(exponent));
  const scientific = formatted + "E" + (exponent < 0 ? "-" : "") + exponentDigits;
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_es(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "," + fraction : integer;
  const exponentDigits = String(Math.abs(exponent));
  const scientific = formatted + "E" + (exponent < 0 ? "-" : "") + exponentDigits;
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_tr(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "," + fraction : integer;
  const exponentDigits = String(Math.abs(exponent));
  const scientific = formatted + "E" + (exponent < 0 ? "-" : "") + exponentDigits;
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_hi(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "." + fraction : integer;
  const exponentDigits = String(Math.abs(exponent));
  const scientific = formatted + "E" + (exponent < 0 ? "-" : "") + exponentDigits;
  const prefix = decimal.negative ? "-[" : "[";
  const suffix = "]";
  return prefix + scientific + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_ff_Adlm(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "." + fraction : integer;
  const localized = formatted.replace(/\d/g, __lgl_digit3);
  const exponentDigits = String(Math.abs(exponent));
  const scientific = localized + "𞤉" + (exponent < 0 ? "-" : "") + exponentDigits.replace(/\d/g, __lgl_digit3);
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_ccp(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "." + fraction : integer;
  const localized = formatted.replace(/\d/g, __lgl_digit4);
  const exponentDigits = String(Math.abs(exponent));
  const scientific = localized + "E" + (exponent < 0 ? "-" : "") + exponentDigits.replace(/\d/g, __lgl_digit4);
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_lo(value) {
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
export function scientific_si(value) {
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
export function scientific_mr(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "." + fraction : integer;
  const localized = formatted.replace(/\d/g, __lgl_digit5);
  const exponentDigits = String(Math.abs(exponent));
  const scientific = localized + "E" + (exponent < 0 ? "-" : "") + exponentDigits.replace(/\d/g, __lgl_digit5);
  const prefix = decimal.negative ? "-[" : "[";
  const suffix = "]";
  return prefix + scientific + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_gu(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  const exponent = mantissa.exponent;
  const integer = mantissa.integer;
  const fraction = mantissa.fraction;
  const formatted = fraction ? integer + "." + fraction : integer;
  const exponentDigits = String(Math.abs(exponent));
  const scientific = formatted + "E" + (exponent < 0 ? "-" : "") + exponentDigits;
  const prefix = decimal.negative ? "-[" : "[";
  const suffix = "]";
  return prefix + scientific + suffix;
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
 * @returns {__lgl_Decimal & { exponent: number }}
 */
function scientificGeneratedDecimal(decimal) {
  if (decimal.integer === "0" && decimal.fraction === "") {
    return { negative: decimal.negative, integer: "0", fraction: "", exponent: 0 };
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
    negative: decimal.negative,
    integer: source[0],
    fraction: source.slice(1).replace(/0+$/, ""),
    exponent,
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
/**
 * @param {string} digit
 * @returns {string}
 */
function __lgl_digit2(digit) {
  return "۰۱۲۳۴۵۶۷۸۹"[digit.charCodeAt(0) - 48];
}
/**
 * @param {string} digit
 * @returns {string}
 */
function __lgl_digit3(digit) {
  return String.fromCodePoint(digit.charCodeAt(0) + 125216);
}
/**
 * @param {string} digit
 * @returns {string}
 */
function __lgl_digit4(digit) {
  return String.fromCodePoint(digit.charCodeAt(0) + 69894);
}
/**
 * @param {string} digit
 * @returns {string}
 */
function __lgl_digit5(digit) {
  return "०१२३४५६७८९"[digit.charCodeAt(0) - 48];
}
/**
 * @param {string} digit
 * @returns {string}
 */
function localizeDigit(digit) {
  return "٠١٢٣٤٥٦٧٨٩"[digit.charCodeAt(0) - 48];
}
