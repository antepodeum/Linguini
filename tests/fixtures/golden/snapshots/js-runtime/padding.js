/** @typedef {number | bigint | string} __lgl_Numeric */

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding0_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  let integer = rounded.integer;
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
  const prefix = decimal.negative ? "-pre" : "pre";
  const suffix = "tail";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 6 : 7) - formatted.length));
  return prefix + padding + formatted + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding1_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  let integer = rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  if (integer.length > 3) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += "٬" + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const formatted = integer + "٫" + fraction;
  const localized = formatted.replace(/\d/g, localizeDigit);
  const prefix = decimal.negative ? "➖pre" : "💠0pre";
  const suffix = decimal.negative ? "💠tail" : "tail💠1";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 5 : 3) - formatted.length));
  return prefix + padding + localized + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding2_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  let integer = rounded.integer;
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
  const localized = formatted.replace(/\d/g, __lgl_digit2);
  const prefix = decimal.negative ? "-pre" : "pre";
  const suffix = "tail";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 6 : 7) - formatted.length));
  return prefix + padding + localized + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding3_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  let integer = rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  let groupPaddingCorrection = 0;
  if (integer.length > 3) {
    groupPaddingCorrection = Math.ceil((integer.length - 3) / 2);
    const primaryStart = integer.length - 3;
    const firstGroup = primaryStart % 2 || 2;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < primaryStart; offset += 2) {
      grouped += "🟦," + integer.slice(offset, offset + 2);
    }
    integer = grouped + "🟦," + integer.slice(primaryStart);
  }
  const formatted = integer + "💠." + fraction;
  const localized = formatted.replace(/\d/g, __lgl_digit3);
  const prefix = decimal.negative ? "-pre" : "pre";
  const suffix = "tail";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 6 : 7) - formatted.length + groupPaddingCorrection + 1));
  return prefix + padding + localized + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding4_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  let integer = rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  let groupPaddingCorrection = 0;
  if (integer.length > 3) {
    groupPaddingCorrection = Math.ceil((integer.length - 3) / 3);
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += "🟦," + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const formatted = integer + "💠." + fraction;
  const prefix = decimal.negative ? "➖pre" : "💠pre";
  const suffix = decimal.negative ? "💠tail" : "tail";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 5 : 6) - formatted.length + groupPaddingCorrection + 1));
  return prefix + padding + formatted + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding5_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  let integer = rounded.integer;
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
  const prefix = decimal.negative ? "➖" : "p";
  const suffix = decimal.negative ? "💠" : "s";
  const padding = "_".repeat(Math.max(0, 12 - formatted.length));
  return prefix + padding + formatted + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding6_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  let exponent = mantissa.exponent;
  let rounded = roundTo1FractionDigits(mantissa);
  if (rounded.integer.length > 1) {
    exponent++;
    rounded = { integer: "1", fraction: "0" };
  }
  const integer = rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  const formatted = integer + "💠." + fraction;
  const exponentDigits = String(Math.abs(exponent)).padStart(2, "0");
  const scientific = formatted + "E💠" + (exponent < 0 ? "➖💠" : "➕💠") + exponentDigits;
  const prefix = decimal.negative ? "-pre" : "pre";
  const suffix = "tail";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 2 : 3) - formatted.length + 1 - exponentDigits.length));
  return prefix + padding + scientific + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding7_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const mantissa = scientificGeneratedDecimal(decimal);
  let exponent = mantissa.exponent;
  let rounded = roundTo1FractionDigits(mantissa);
  if (rounded.integer.length > 1) {
    exponent++;
    rounded = { integer: "1", fraction: "0" };
  }
  const integer = rounded.integer;
  const fraction = rounded.fraction.padEnd(1, "0");
  const formatted = integer + "." + fraction;
  const localized = formatted.replace(/\d/g, __lgl_digit2);
  const exponentDigits = String(Math.abs(exponent)).padStart(2, "0");
  const scientific = localized + "E💠" + (exponent < 0 ? "➖💠" : "") + exponentDigits.replace(/\d/g, __lgl_digit2);
  const prefix = decimal.negative ? "-pre" : "pre";
  const suffix = "tail";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 4 : 5) - formatted.length - exponentDigits.length - (exponent < 0 ? 2 : 0)));
  return prefix + padding + scientific + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding8_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  let integer = rounded.integer;
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
  const localized = formatted.replace(/\d/g, __lgl_digit4);
  const prefix = decimal.negative ? "➖mixed0" : "mixed0";
  const suffix = "mixed1";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 1 : 2) - formatted.length));
  return prefix + padding + localized + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding9_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo1FractionDigits(decimal);
  let integer = rounded.integer;
  const fraction = rounded === decimal ? rounded.fraction : rounded.fraction.replace(/0+$/, "");
  let groupPaddingCorrection = 0;
  if (integer.length > 4) {
    groupPaddingCorrection = Math.ceil((integer.length - 3) / 2) * 2;
    const primaryStart = integer.length - 3;
    const firstGroup = primaryStart % 2 || 2;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < primaryStart; offset += 2) {
      grouped += "🟦,🟦" + integer.slice(offset, offset + 2);
    }
    integer = grouped + "🟦,🟦" + integer.slice(primaryStart);
  }
  const formatted = fraction ? integer + "💠.💠" + fraction : integer;
  const prefix = decimal.negative ? "-p" : "p";
  const suffix = "s";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 11 : 12) - formatted.length + groupPaddingCorrection + (fraction ? 2 : 0)));
  return prefix + padding + formatted + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding10_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  let groupPaddingCorrection = 0;
  if (integer.length > 4) {
    groupPaddingCorrection = Math.ceil((integer.length - 3) / 2) * 2;
    const primaryStart = integer.length - 3;
    const firstGroup = primaryStart % 2 || 2;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < primaryStart; offset += 2) {
      grouped += "🟦,🟦" + integer.slice(offset, offset + 2);
    }
    integer = grouped + "🟦,🟦" + integer.slice(primaryStart);
  }
  const prefix = decimal.negative ? "-p" : "p";
  const suffix = "s";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 11 : 12) - integer.length + groupPaddingCorrection));
  return prefix + padding + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding11_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  let groupPaddingCorrection = 0;
  if (integer.length > 4) {
    groupPaddingCorrection = Math.ceil((integer.length - 3) / 2) * 2;
    const primaryStart = integer.length - 3;
    const firstGroup = primaryStart % 2 || 2;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < primaryStart; offset += 2) {
      grouped += "🟦,🟦" + integer.slice(offset, offset + 2);
    }
    integer = grouped + "🟦,🟦" + integer.slice(primaryStart);
  }
  const formatted = integer + "💠.💠";
  const prefix = decimal.negative ? "-p" : "p";
  const suffix = "s";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 11 : 12) - formatted.length + groupPaddingCorrection + 2));
  return prefix + padding + formatted + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function padding12_14_1(value) {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const rounded = roundTo3SignificantDigits(decimal);
  let integer = rounded.integer;
  let fraction = rounded.fraction.replace(/0+$/, "");
  fraction = padTo2SignificantDigits(integer, fraction);
  let groupPaddingCorrection = 0;
  if (integer.length > 4) {
    groupPaddingCorrection = Math.ceil((integer.length - 3) / 2) * 2;
    const primaryStart = integer.length - 3;
    const firstGroup = primaryStart % 2 || 2;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < primaryStart; offset += 2) {
      grouped += "🟦,🟦" + integer.slice(offset, offset + 2);
    }
    integer = grouped + "🟦,🟦" + integer.slice(primaryStart);
  }
  const formatted = fraction ? integer + "💠.💠" + fraction : integer;
  const prefix = decimal.negative ? "-p" : "p";
  const suffix = "s";
  const padding = "_".repeat(Math.max(0, (decimal.negative ? 11 : 12) - formatted.length + groupPaddingCorrection + (fraction ? 2 : 0)));
  return prefix + padding + formatted + suffix;
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
function roundTo1FractionDigits(decimal) {
  if (decimal.fraction.length <= 1) return decimal;
  const fraction = decimal.fraction.slice(0, 1);
  if (decimal.fraction[1] < "5") return { integer: decimal.integer, fraction };
  const digits = (BigInt(`${decimal.integer}${fraction}`) + 1n).toString().padStart(2, "0");
  return { integer: digits.slice(0, -1), fraction: digits.slice(-1) };
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
function __lgl_digit4(digit) {
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
}
/**
 * @param {string} digit
 * @returns {string}
 */
function localizeDigit(digit) {
  return "٠١٢٣٤٥٦٧٨٩"[digit.charCodeAt(0) - 48];
}
