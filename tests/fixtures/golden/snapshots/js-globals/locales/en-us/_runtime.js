/** @typedef {number | bigint | string} GeneratedNumeric */

/**
 * @param {GeneratedNumeric} value
 * @returns {string}
 */
export function formatNumber(value) {
  return formatGeneratedNumber(value, "", "", undefined, undefined, 1, 0, 3, 3, undefined, ".", ",", "0123456789", 1, undefined, undefined, undefined, undefined, false, undefined, undefined, undefined);
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
 * @param {string} prefix
 * @param {string} suffix
 * @param {string | undefined} negativePrefix
 * @param {string | undefined} negativeSuffix
 * @param {number} minIntegerDigits
 * @param {number} minFractionDigits
 * @param {number} maxFractionDigits
 * @param {number | undefined} primaryGroupSize
 * @param {number | undefined} secondaryGroupSize
 * @param {string} decimalSymbol
 * @param {string} groupSymbol
 * @param {string} digits
 * @param {number} scale
 * @param {number | undefined} minSignificantDigits
 * @param {number | undefined} maxSignificantDigits
 * @param {string | undefined} patternRoundingIncrement
 * @param {number | undefined} exponentDigits
 * @param {boolean} exponentSignAlways
 * @param {string | undefined} paddingCharacter
 * @param {number | undefined} formatWidth
 * @param {number | undefined} paddingPosition
 * @param {number} [minFractionDigitsOverride]
 * @param {number} [maxFractionDigitsOverride]
 * @param {number} [roundingIncrementOverride]
 * @returns {string}
 */
function formatGeneratedNumber(
  value,
  prefix,
  suffix,
  negativePrefix,
  negativeSuffix,
  minIntegerDigits,
  minFractionDigits,
  maxFractionDigits,
  primaryGroupSize,
  secondaryGroupSize,
  decimalSymbol,
  groupSymbol,
  digits,
  scale,
  minSignificantDigits,
  maxSignificantDigits,
  patternRoundingIncrement,
  exponentDigits,
  exponentSignAlways,
  paddingCharacter,
  formatWidth,
  paddingPosition,
  minFractionDigitsOverride,
  maxFractionDigitsOverride,
  roundingIncrementOverride,
) {
  let decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  decimal = scaleGeneratedDecimal(decimal, scale);
  const effectiveMinFractionDigits = minFractionDigitsOverride ?? minFractionDigits;
  const effectiveMaxFractionDigits = maxFractionDigitsOverride ?? maxFractionDigits;
  /** @type {number | undefined} */
  let exponent;
  if (exponentDigits !== undefined) {
    ({ decimal, exponent } = scientificGeneratedDecimal(decimal));
  }
  let rounded = maxSignificantDigits === undefined
    ? roundGeneratedDecimal(
        decimal,
        effectiveMaxFractionDigits,
        roundingIncrementOverride ?? patternRoundingIncrement ?? 0,
      )
    : roundGeneratedSignificant(decimal, maxSignificantDigits);
  if (exponent !== undefined && rounded.integer.length > 1) {
    const shift = rounded.integer.length - 1;
    exponent += shift;
    rounded = {
      integer: rounded.integer[0],
      fraction: `${rounded.integer.slice(1)}${rounded.fraction}`,
    };
  }
  let integer = rounded.integer.padStart(minIntegerDigits, "0");
  let fraction = trimOptionalFractionDigits(
    rounded.fraction,
    effectiveMinFractionDigits,
  );
  if (minSignificantDigits !== undefined) {
    fraction = padGeneratedSignificant(integer, fraction, minSignificantDigits);
  }

  integer = groupIntegerDigits(integer, primaryGroupSize, secondaryGroupSize, groupSymbol);
  let ascii = fraction ? `${integer}${decimalSymbol}${fraction}` : integer;
  if (exponent !== undefined && exponentDigits !== undefined) {
    const sign = exponent < 0 ? "-" : exponentSignAlways ? "+" : "";
    ascii += `E${sign}${String(Math.abs(exponent)).padStart(exponentDigits, "0")}`;
  }
  const formatted = localizeGeneratedDigits(ascii, digits);
  const effectivePrefix = decimal.negative ? negativePrefix ?? `-${prefix}` : prefix;
  const effectiveSuffix = decimal.negative ? negativeSuffix ?? suffix : suffix;
  return padGeneratedNumber(
    effectivePrefix,
    formatted,
    effectiveSuffix,
    paddingCharacter,
    formatWidth,
    paddingPosition,
  );
}

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
 * @param {number} scale
 * @returns {GeneratedDecimal}
 */
function scaleGeneratedDecimal(decimal, scale) {
  const shift = scale === 1 ? 0 : scale === 100 ? 2 : scale === 1000 ? 3 : -1;
  if (shift < 0) throwInvalidNumber();
  if (shift === 0) return decimal;
  const source = `${decimal.integer}${decimal.fraction}`;
  const decimalPosition = decimal.integer.length + shift;
  if (source.length + shift > MAX_GENERATED_DECIMAL_DIGITS) throwInvalidNumber();
  const expanded = source.padEnd(decimalPosition, "0");
  const integer = expanded.slice(0, decimalPosition).replace(/^0+(?=\d)/, "") || "0";
  const fraction = expanded.slice(decimalPosition).replace(/0+$/, "");
  return { negative: decimal.negative, integer, fraction };
}

/**
 * @param {GeneratedDecimal} decimal
 * @returns {{ decimal: GeneratedDecimal, exponent: number }}
 */
function scientificGeneratedDecimal(decimal) {
  if (/^0+$/.test(decimal.integer) && !/[1-9]/.test(decimal.fraction)) {
    return { decimal: { ...decimal, integer: "0", fraction: "" }, exponent: 0 };
  }
  /** @type {string} */
  let source;
  /** @type {number} */
  let exponent;
  if (/[1-9]/.test(decimal.integer)) {
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
 * @param {number} maximumDigits
 * @returns {{ integer: string, fraction: string }}
 */
function roundGeneratedSignificant(decimal, maximumDigits) {
  if (
    !Number.isSafeInteger(maximumDigits) ||
    maximumDigits < 1 ||
    maximumDigits > MAX_GENERATED_DECIMAL_DIGITS
  ) {
    throwInvalidNumber();
  }
  const combined = `${decimal.integer}${decimal.fraction}`;
  const first = combined.search(/[1-9]/);
  if (first < 0) return { integer: "0", fraction: "" };
  const cut = first + maximumDigits;
  if (cut >= decimal.integer.length) {
    return roundGeneratedDecimal(decimal, cut - decimal.integer.length, 0);
  }

  const kept = decimal.integer.slice(0, cut);
  const discarded = `${decimal.integer.slice(cut)}${decimal.fraction}`;
  const rounded = BigInt(kept) + (discarded[0] >= "5" ? 1n : 0n);
  return {
    integer: `${rounded}${"0".repeat(decimal.integer.length - cut)}`,
    fraction: "",
  };
}

/**
 * @param {string} integer
 * @param {string} fraction
 * @param {number} minimumDigits
 * @returns {string}
 */
function padGeneratedSignificant(integer, fraction, minimumDigits) {
  if (!Number.isSafeInteger(minimumDigits) || minimumDigits < 1) throwInvalidNumber();
  const combined = `${integer}${fraction}`;
  const first = combined.search(/[1-9]/);
  const current = first < 0 ? 1 : combined.length - first;
  return fraction.padEnd(fraction.length + Math.max(0, minimumDigits - current), "0");
}

/**
 * @param {string} prefix
 * @param {string} number
 * @param {string} suffix
 * @param {string | undefined} character
 * @param {number | undefined} width
 * @param {number | undefined} position
 * @returns {string}
 */
function padGeneratedNumber(prefix, number, suffix, character, width, position) {
  if (character === undefined || width === undefined || position === undefined) {
    return `${prefix}${number}${suffix}`;
  }
  if ([...character].length !== 1 || !Number.isSafeInteger(width) || width < 0) {
    throwInvalidNumber();
  }
  const padding = character.repeat(Math.max(0, width - [...`${prefix}${number}${suffix}`].length));
  switch (position) {
    case 0: return `${padding}${prefix}${number}${suffix}`;
    case 1: return `${prefix}${padding}${number}${suffix}`;
    case 2: return `${prefix}${number}${padding}${suffix}`;
    case 3: return `${prefix}${number}${suffix}${padding}`;
    default: throwInvalidNumber();
  }
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

/**
 * @param {string} integer
 * @param {number | undefined} primaryGroupSize
 * @param {number | undefined} secondaryGroupSize
 * @param {string} groupSymbol
 * @returns {string}
 */
function groupIntegerDigits(integer, primaryGroupSize, secondaryGroupSize, groupSymbol) {
  if (!primaryGroupSize || integer.length <= primaryGroupSize) return integer;
  /** @type {string[]} */
  const groups = [];
  let end = integer.length;
  let groupSize = primaryGroupSize;
  while (end > 0) {
    const start = Math.max(0, end - groupSize);
    groups.unshift(integer.slice(start, end));
    end = start;
    groupSize = secondaryGroupSize ?? primaryGroupSize;
  }
  return groups.join(groupSymbol);
}

/** @returns {never} */
function throwInvalidNumber() {
  throw new RangeError("Linguini: invalid numeric value");
}
/**
 * @param {string} value
 * @param {string} digits
 * @returns {string}
 */
function localizeGeneratedDigits(value, digits) {
  if (digits === "0123456789") return value;
  const symbols = Array.from(digits);
  if (symbols.length !== 10) throw new RangeError("Linguini: invalid numbering system");
  return value.replace(/\d/g, (digit) => symbols[digit.charCodeAt(0) - 48]);
}


/**
 * @param {number | bigint | string} value
 * @returns {string}
 */
export function pluralEnUs(value) {
  if (value === "zero" || value === "one" || value === "two" || value === "few" || value === "many" || value === "other") return value;
  const operands = pluralOperands(value);
  if ((pluralOperandMatches(operands.i, undefined, false, [[1n, 1n]]) && pluralOperandMatches(operands.v, undefined, false, [[0n, 0n]]))) return "one";
  return "other";
}

/**
 * @typedef {object} PluralOperand
 * @property {bigint} integer
 * @property {boolean} hasFraction
 */
const MAX_PLURAL_DECIMAL_DIGITS = 8192;

/**
 * @param {number | bigint | string} value
 */
function pluralOperands(value) {
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

  const digits = `${whole}${sourceFraction}` || "0";
  const decimalPosition = whole.length + exponent;
  const expandedLength = decimalPosition <= 0
    ? -decimalPosition + digits.length
    : Math.max(decimalPosition, digits.length);
  if (expandedLength > MAX_PLURAL_DECIMAL_DIGITS) throwInvalidPluralNumber();

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
  const trimmedFraction = fraction.replace(/0+$/, "");
  const compactExponent = typeof value === "string" ? match[4] ?? "0" : "0";
  /**
   * @param {string} digits
   * @param {boolean} [hasFraction]
   * @returns {PluralOperand}
   */
  const operand = (digits, hasFraction = false) => ({
    integer: BigInt(digits || "0"),
    hasFraction,
  });

  return {
    n: operand(integer, /[1-9]/.test(fraction)),
    i: operand(integer),
    v: operand(String(fraction.length)),
    w: operand(String(trimmedFraction.length)),
    f: operand(fraction),
    t: operand(trimmedFraction),
    c: operand(compactExponent),
    e: operand(compactExponent),
  };
}

/**
 * @param {PluralOperand} value
 * @param {bigint | undefined} modulo
 * @param {boolean} allowFraction
 * @param {ReadonlyArray<readonly [bigint, bigint]>} ranges
 * @returns {boolean}
 */
function pluralOperandMatches(
  value,
  modulo,
  allowFraction,
  ranges,
) {
  const integer = modulo === undefined || modulo === 0n
    ? value.integer
    : value.integer % modulo;
  return ranges.some(([start, end]) => {
    if (!allowFraction) {
      return !value.hasFraction && integer >= start && integer <= end;
    }
    return integer >= start &&
      (integer < end || (integer === end && !value.hasFraction));
  });
}

/** @returns {never} */
function throwInvalidPluralNumber() {
  throw new RangeError("Linguini: invalid plural numeric value");
}
//# sourceMappingURL=_runtime.js.map
