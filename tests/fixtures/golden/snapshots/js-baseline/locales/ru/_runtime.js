/** @typedef {number | bigint | string} __lgl_Numeric */

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function formatCurrency_code_RUB(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo2FractionDigits(decimal);
  let integer = rounded.integer;
  const fraction = rounded.fraction.padEnd(2, "0");
  if (integer.length > 3) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += " " + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const formatted = integer + "," + fraction;
  const prefix = decimal.negative ? "-" : "";
  const suffix = " ₽";
  return prefix + formatted + suffix;
}

/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatDate_style_short(value) {
  const date = coerceDate(value);
  let year = date.getUTCFullYear();
  if (year <= 0) year = 1 - year;
  return padNumber(date.getUTCDate(), 2) + "." + padNumber(date.getUTCMonth() + 1, 2) + "." + String(year);
}

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
      grouped += " " + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const formatted = fraction ? integer + "," + fraction : integer;
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
function roundTo2FractionDigits(decimal) {
  if (decimal.fraction.length <= 2) return decimal;
  const fraction = decimal.fraction.slice(0, 2);
  if (decimal.fraction[2] < "5") return { integer: decimal.integer, fraction };
  const digits = (BigInt(`${decimal.integer}${fraction}`) + 1n).toString().padStart(3, "0");
  return { integer: digits.slice(0, -2), fraction: digits.slice(-2) };
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
/**
 * @param {number} value
 * @param {number} length
 * @returns {string}
 */
function padNumber(value, length) {
  return String(value).padStart(length, "0");
}

/**
 * @param {globalThis.Date | number | string} value
 * @returns {globalThis.Date}
 */
function coerceDate(value) {
  /** @type {globalThis.Date} */
  let date;
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
      validateDateFields(Number(dateTime[1]), Number(dateTime[2]), Number(dateTime[3]));
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

/**
 * @param {number} year
 * @param {number} month
 * @param {number} day
 * @returns {globalThis.Date}
 */
function createUTCDate(year, month, day) {
  validateDateFields(year, month, day);
  const date = new Date(0);
  date.setUTCFullYear(year, month - 1, day);
  return date;
}

/**
 * @param {number} year
 * @param {number} month
 * @param {number} day
 * @returns {void}
 */
function validateDateFields(year, month, day) {
  const days = month === 2
    ? (year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0) ? 29 : 28)
    : (month === 4 || month === 6 || month === 9 || month === 11 ? 30 : 31);
  if (
    month < 1 || month > 12 || day < 1 || day > days
  ) {
    throwInvalidDate();
  }
}

/** @returns {never} */
function throwInvalidDate() {
  throw new RangeError("Linguini: invalid date value");
}


/**
 * @param {number | bigint | string} value
 * @returns {string}
 */
export function pluralRu(value) {
  let integerTail;
  if (typeof value === "number" && Number.isSafeInteger(value)) {
    integerTail = Math.abs(value) % 100;
  } else {
    if (value === "zero" || value === "one" || value === "two" || value === "few" || value === "many" || value === "other") return value;
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

    const digitCount = whole.length + sourceFraction.length;
    const decimalPosition = whole.length + exponent;
    const expandedLength = decimalPosition <= 0
      ? -decimalPosition + digitCount
      : Math.max(decimalPosition, digitCount);
    if (expandedLength > MAX_PLURAL_DECIMAL_DIGITS) throwInvalidPluralNumber();

    const visibleFractionDigits = Math.max(0, digitCount - decimalPosition);
    if (visibleFractionDigits !== 0) return "other";
    const digits = `${whole}${sourceFraction}`;
    integerTail = decimalPosition <= 0 || decimalPosition - digitCount >= 2 ? 0
      : Number(digits.slice(Math.max(0, decimalPosition - 2), decimalPosition)
        + (decimalPosition > digitCount ? "0".repeat(decimalPosition - digitCount) : ""));
  }
  const integerTailModulo10 = integerTail % 10;
  if (integerTailModulo10 === 1 && integerTail !== 11) return "one";
  if ((integerTailModulo10 >= 2 && integerTailModulo10 <= 4) && (integerTail < 12 || integerTail > 14)) return "few";
  return "many";
}

const MAX_PLURAL_DECIMAL_DIGITS = 8192;

/** @returns {never} */
function throwInvalidPluralNumber() {
  throw new RangeError("Linguini: invalid plural numeric value");
}
