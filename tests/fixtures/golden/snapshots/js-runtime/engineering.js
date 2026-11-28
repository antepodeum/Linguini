/** @typedef {number | bigint | string} __lgl_Numeric */

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientificFixed3(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificWith3IntegerDigits(decimal);
  let exponent = mantissa.exponent;
  let rounded = roundTo2FractionDigits(mantissa);
  if (rounded.integer.length > 3) {
    exponent++;
    rounded = { integer: "100", fraction: "00" };
  }
  const integer = rounded.integer.padStart(3, "0");
  const fraction = rounded.fraction.padEnd(2, "0");
  const formatted = integer + "." + fraction;
  const exponentDigits = String(Math.abs(exponent));
  const scientific = formatted + "E" + (exponent < 0 ? "-" : "") + exponentDigits;
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function engineering3(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = engineeringWith3IntegerDigits(decimal);
  let exponent = mantissa.exponent;
  let rounded = roundTo3SignificantDigits(mantissa);
  if (rounded.integer.length > 3) {
    exponent += 3;
    rounded = { integer: "1", fraction: "00" };
  }
  const integer = rounded.integer;
  const fraction = rounded.integer === "0" ? "00" : rounded.fraction.slice(0, 3 - rounded.integer.length);
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
export function engineeringUnlimited(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = engineeringWith3IntegerDigits(decimal);
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
export function engineeringIncrement(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = engineeringWith3IntegerDigits(decimal);
  let exponent = mantissa.exponent;
  let rounded = roundTo1FractionDigitsIncrement7(mantissa);
  if (
    mantissa.integer.length === 3 &&
    (rounded.integer.length !== mantissa.integer.length || (rounded.integer === "0" && mantissa.integer !== "0")) &&
    (rounded.integer !== "0" || /[1-9]/.test(rounded.fraction))
  ) {
    exponent += 3;
    rounded = roundTo1FractionDigitsIncrement7(shiftScientificMantissa(mantissa, 3));
  }
  const integer = rounded.integer;
  const fraction = rounded.fraction;
  const formatted = integer + "." + fraction;
  const exponentDigits = String(Math.abs(exponent));
  const scientific = formatted + "E" + (exponent < 0 ? "-" : "") + exponentDigits;
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
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
function engineeringWith3IntegerDigits(decimal) {
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
  const integerDigits = exponent - Math.floor(exponent / 3) * 3 + 1;
  exponent -= integerDigits - 1;
  return {
    negative: decimal.negative,
    integer: source.slice(0, integerDigits).padEnd(integerDigits, "0"),
    fraction: source.slice(integerDigits).replace(/0+$/, ""),
    exponent,
  };
}

/**
 * @param {__lgl_Decimal} decimal
 * @returns {__lgl_Decimal & { exponent: number }}
 */
function scientificWith3IntegerDigits(decimal) {
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
  exponent -= 2;
  return {
    negative: decimal.negative,
    integer: source.slice(0, 3).padEnd(3, "0"),
    fraction: source.slice(3).replace(/0+$/, ""),
    exponent,
  };
}

/**
 * @param {__lgl_Decimal} decimal
 * @param {number} shift
 * @returns {__lgl_Decimal}
 */
function shiftScientificMantissa(decimal, shift) {
  const source = `${decimal.integer}${decimal.fraction}`;
  const decimalPosition = decimal.integer.length - shift;
  return {
    negative: decimal.negative,
    integer: decimalPosition > 0 ? source.slice(0, decimalPosition) : "0",
    fraction: decimalPosition > 0 ? source.slice(decimalPosition) : "0".repeat(-decimalPosition) + source,
  };
}

/**
 * @param {__lgl_Decimal} decimal
 * @returns {{ integer: string, fraction: string }}
 */
function roundTo3SignificantDigits(decimal) {
  if (decimal.integer === "0") return { integer: "0", fraction: "" };
  return roundGeneratedFraction(decimal, 3 - decimal.integer.length);
}

/**
 * @param {__lgl_Decimal} decimal
 * @returns {{ integer: string, fraction: string }}
 */
function roundTo1FractionDigitsIncrement7(decimal) {
  const fraction = decimal.fraction.slice(0, 1).padEnd(1, "0");
  const scaled = BigInt(`${decimal.integer}${fraction}`);
  const remainder = scaled % 7n;
  const roundUp = remainder > 3n || (remainder === 3n && decimal.fraction[1] >= "5");
  const digits = (scaled - remainder + (roundUp ? 7n : 0n)).toString().padStart(2, "0");
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

/**
 * @param {__lgl_Decimal} decimal
 * @param {number} fractionDigits
 * @returns {{ integer: string, fraction: string }}
 */
function roundGeneratedFraction(decimal, fractionDigits) {
  const fraction = decimal.fraction.slice(0, fractionDigits).padEnd(fractionDigits, "0");
  if (decimal.fraction.length <= fractionDigits || decimal.fraction[fractionDigits] < "5") return { integer: decimal.integer, fraction };
  const digits = (BigInt(`${decimal.integer}${fraction}`) + 1n).toString().padStart(fractionDigits + 1, "0");
  return fractionDigits === 0
    ? { integer: digits, fraction: "" }
    : { integer: digits.slice(0, -fractionDigits), fraction: digits.slice(-fractionDigits) };
}

/** @returns {never} */
function throwInvalidNumber() {
  throw new RangeError("Linguini: invalid numeric value");
}
