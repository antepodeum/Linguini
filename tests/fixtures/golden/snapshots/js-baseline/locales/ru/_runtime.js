/** @typedef {number | bigint | string} GeneratedNumeric */

/**
 * @param {GeneratedNumeric} value
 * @returns {string}
 */
export function formatCurrency_code_RUB(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundGeneratedDecimal(decimal, 2, "1");
  let integer = rounded.integer;
  const fraction = trimOptionalFractionDigits(rounded.fraction, 2);
  if (integer.length > 3) {
    const groups = [integer.slice(-3)];
    let end = integer.length - 3;
    while (end > 0) {
      const start = Math.max(0, end - 3);
      groups.push(integer.slice(start, end));
      end = start;
    }
    integer = groups.reverse().join(" ");
  }
  const formatted = fraction ? integer + "," + fraction : integer;
  const prefix = decimal.negative ? "-" : "";
  const suffix = " ₽";
  return prefix + formatted + suffix;
}

/**
 * @param {Date | number | string} value
 * @returns {string}
 */
export function formatDate_style_short(value) {
  const date = coerceDate(value);
  return padNumber(date.getUTCDate(), 2) + "." + padNumber(date.getUTCMonth() + 1, 2) + "." + String(date.getUTCFullYear());
}

/**
 * @param {GeneratedNumeric} value
 * @returns {string}
 */
export function formatNumber(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundGeneratedDecimal(decimal, 3, "1");
  let integer = rounded.integer;
  const fraction = trimOptionalFractionDigits(rounded.fraction, 0);
  if (integer.length > 3) {
    const groups = [integer.slice(-3)];
    let end = integer.length - 3;
    while (end > 0) {
      const start = Math.max(0, end - 3);
      groups.push(integer.slice(start, end));
      end = start;
    }
    integer = groups.reverse().join(" ");
  }
  const formatted = fraction ? integer + "," + fraction : integer;
  const prefix = decimal.negative ? "-" : "";
  return prefix + formatted;
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
    negative: match[1] === "-" || negativeZero,
    integer,
    fraction,
  };
}

/**
 * @param {GeneratedDecimal} decimal
 * @param {number} fractionDigits
 * @param {number | string} roundingIncrement
 * @returns {{ integer: string, fraction: string }}
 */
function roundGeneratedDecimal(decimal, fractionDigits, roundingIncrement) {
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
  /** @type {boolean} */
  let roundUp;
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

/**
 * @param {string} fraction
 * @param {number} minDigits
 * @returns {string}
 */
function trimOptionalFractionDigits(fraction, minDigits) {
  while (fraction.length > minDigits && fraction.endsWith("0")) {
    fraction = fraction.slice(0, -1);
  }
  return fraction;
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
 * @param {Date | number | string} value
 * @returns {Date}
 */
function coerceDate(value) {
  /** @type {Date} */
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

/**
 * @param {number} year
 * @param {number} month
 * @param {number} day
 * @returns {Date}
 */
function createUTCDate(year, month, day) {
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

/** @returns {never} */
function throwInvalidDate() {
  throw new RangeError("Linguini: invalid date value");
}


/**
 * @param {number | bigint | string} value
 * @returns {string}
 */
export function pluralRu(value) {
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
  const integer = decimalPosition <= 0 ? "0"
    : decimalPosition >= digitCount ? `${digits}${"0".repeat(decimalPosition - digitCount)}`
    : digits.slice(0, decimalPosition);
  const integerValue = BigInt(integer);
  const integerValueModulo10 = integerValue % 10n;
  const integerValueModulo100 = integerValue % 100n;
  if (integerValueModulo10 === 1n && integerValueModulo100 !== 11n) return "one";
  if ((integerValueModulo10 >= 2n && integerValueModulo10 <= 4n) && (integerValueModulo100 < 12n || integerValueModulo100 > 14n)) return "few";
  if (integerValueModulo10 === 0n || (integerValueModulo10 >= 5n && integerValueModulo10 <= 9n) || (integerValueModulo100 >= 11n && integerValueModulo100 <= 14n)) return "many";
  return "other";
}

const MAX_PLURAL_DECIMAL_DIGITS = 8192;

/** @returns {never} */
function throwInvalidPluralNumber() {
  throw new RangeError("Linguini: invalid plural numeric value");
}
