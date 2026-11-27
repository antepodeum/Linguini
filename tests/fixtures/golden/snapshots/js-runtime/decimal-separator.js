/** @typedef {number | bigint | string} __lgl_Numeric */

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function integer_ordinary(value) {
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
export function integer_visible(value) {
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
  return prefix + integer + ".";
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function optional_ordinary(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo2FractionDigits(decimal);
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
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function optional_visible(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo2FractionDigits(decimal);
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
  const formatted = integer + "." + fraction;
  const prefix = decimal.negative ? "-" : "";
  return prefix + formatted;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function required_ordinary(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo2FractionDigits(decimal);
  let integer = rounded.integer;
  const fraction = rounded.fraction.padEnd(2, "0");
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
export function required_visible(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo2FractionDigits(decimal);
  let integer = rounded.integer;
  const fraction = rounded.fraction.padEnd(2, "0");
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
export function significant_ordinary(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo3SignificantDigits(decimal);
  let integer = rounded.integer;
  let fraction = rounded.fraction.replace(/0+$/, "");
  fraction = padTo2SignificantDigits(integer, fraction);
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
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function significant_visible(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo3SignificantDigits(decimal);
  let integer = rounded.integer;
  let fraction = rounded.fraction.replace(/0+$/, "");
  fraction = padTo2SignificantDigits(integer, fraction);
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
export function scientific_ordinary(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  let exponent;
  ({ decimal, exponent } = scientificGeneratedDecimal(decimal));
  let rounded = roundTo0FractionDigits(decimal);
  if (rounded.integer.length > 1) {
    exponent++;
    rounded = { integer: "1", fraction: "" };
  }
  const integer = rounded.integer;
  const formatted = integer;
  const exponentDigits = String(Math.abs(exponent)).padStart(2, "0");
  const scientific = formatted + "E" + (exponent < 0 ? "-" : "") + exponentDigits;
  const prefix = decimal.negative ? "-" : "";
  return prefix + scientific;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function scientific_visible(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  let exponent;
  ({ decimal, exponent } = scientificGeneratedDecimal(decimal));
  let rounded = roundTo0FractionDigits(decimal);
  if (rounded.integer.length > 1) {
    exponent++;
    rounded = { integer: "1", fraction: "" };
  }
  const integer = rounded.integer;
  const formatted = integer + ".";
  const exponentDigits = String(Math.abs(exponent)).padStart(2, "0");
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
 * @returns {{ decimal: __lgl_Decimal, exponent: number }}
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
 * @param {__lgl_Decimal} decimal
 * @returns {{ integer: string, fraction: string }}
 */
function roundTo3SignificantDigits(decimal) {
  const combined = `${decimal.integer}${decimal.fraction}`;
  const first = combined.search(/[1-9]/);
  if (first < 0) return { integer: "0", fraction: "" };
  const cut = first + 3;
  if (cut >= decimal.integer.length) {
    return roundGeneratedFraction(decimal, cut - decimal.integer.length);
  }

  const kept = decimal.integer.slice(0, cut);
  const rounded = BigInt(kept) + (decimal.integer[cut] >= "5" ? 1n : 0n);
  return {
    integer: `${rounded}${"0".repeat(decimal.integer.length - cut)}`,
    fraction: "",
  };
}

/**
 * @param {string} integer
 * @param {string} fraction
 * @returns {string}
 */
function padTo2SignificantDigits(integer, fraction) {
  const combined = `${integer}${fraction}`;
  const first = combined.search(/[1-9]/);
  const current = first < 0 ? 1 : combined.length - first;
  return fraction.padEnd(fraction.length + Math.max(0, 2 - current), "0");
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
  if (fractionDigits > MAX_GENERATED_DECIMAL_DIGITS) throwInvalidNumber();
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
