/** @typedef {number | bigint | string} __lgl_Numeric */

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_en(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
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
  const suffix = "%";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_fr(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  if (integer.length > 3) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += " " + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const prefix = decimal.negative ? "-" : "";
  const suffix = " %";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_ru(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  if (integer.length > 3) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += " " + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const prefix = decimal.negative ? "-" : "";
  const suffix = " %";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_ar(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
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
  const prefix = decimal.negative ? "‎-" : "";
  const suffix = "‎%‎";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_ar_EG(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  if (integer.length > 3) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += "٬" + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const localized = integer.replace(/\d/g, localizeDigit);
  const prefix = decimal.negative ? "؜-" : "";
  const suffix = "٪؜";
  return prefix + localized + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_fa(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  if (integer.length > 3) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += "٬" + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const localized = integer.replace(/\d/g, __lgl_digit2);
  const prefix = decimal.negative ? "‎−" : "";
  const suffix = "٪";
  return prefix + localized + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_sv(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  if (integer.length > 3) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += " " + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const prefix = decimal.negative ? "−" : "";
  const suffix = " %";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_ee(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  if (integer.length > 5) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += "," + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const prefix = decimal.negative ? "-" : "";
  const suffix = "%";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_es(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  if (integer.length > 4) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += "." + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const prefix = decimal.negative ? "-" : "";
  const suffix = " %";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_tr(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  if (integer.length > 3) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += "." + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const prefix = decimal.negative ? "-%" : "%";
  return prefix + integer;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_hi(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  if (integer.length > 3) {
    const primaryStart = integer.length - 3;
    const firstGroup = primaryStart % 2 || 2;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < primaryStart; offset += 2) {
      grouped += "," + integer.slice(offset, offset + 2);
    }
    integer = grouped + "," + integer.slice(primaryStart);
  }
  const prefix = decimal.negative ? "-" : "";
  const suffix = "%";
  return prefix + integer + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_ff_Adlm(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
  const rounded = roundTo0FractionDigits(decimal);
  let integer = rounded.integer;
  if (integer.length > 3) {
    const firstGroup = integer.length % 3 || 3;
    let grouped = integer.slice(0, firstGroup);
    for (let offset = firstGroup; offset < integer.length; offset += 3) {
      grouped += "⹁" + integer.slice(offset, offset + 3);
    }
    integer = grouped;
  }
  const localized = integer.replace(/\d/g, __lgl_digit3);
  const prefix = decimal.negative ? "-" : "";
  const suffix = "%";
  return prefix + localized + suffix;
}

/**
 * @param {__lgl_Numeric} value
 * @returns {string}
 */
export function percent_ccp(value) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleBy100(decimal);
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
  const localized = integer.replace(/\d/g, __lgl_digit4);
  const prefix = decimal.negative ? "-" : "";
  const suffix = "%";
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
function localizeDigit(digit) {
  return "٠١٢٣٤٥٦٧٨٩"[digit.charCodeAt(0) - 48];
}
