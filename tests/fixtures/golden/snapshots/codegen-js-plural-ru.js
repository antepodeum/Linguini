/**
 * @param {number | bigint | string} value
 * @returns {string}
 */
export function pluralRu(value) {
  if (value === "zero" || value === "one" || value === "two" || value === "few" || value === "many" || value === "other") return value;
  const operands = pluralOperands(value);
  if ((pluralOperandMatches(operands.v, undefined, false, [[0n, 0n]]) && pluralOperandMatches(operands.i, 10n, false, [[1n, 1n]]) && !pluralOperandMatches(operands.i, 100n, false, [[11n, 11n]]))) return "one";
  if ((pluralOperandMatches(operands.v, undefined, false, [[0n, 0n]]) && pluralOperandMatches(operands.i, 10n, false, [[2n, 4n]]) && !pluralOperandMatches(operands.i, 100n, false, [[12n, 14n]]))) return "few";
  if ((pluralOperandMatches(operands.v, undefined, false, [[0n, 0n]]) && pluralOperandMatches(operands.i, 10n, false, [[0n, 0n]])) || (pluralOperandMatches(operands.v, undefined, false, [[0n, 0n]]) && pluralOperandMatches(operands.i, 10n, false, [[5n, 9n]])) || (pluralOperandMatches(operands.v, undefined, false, [[0n, 0n]]) && pluralOperandMatches(operands.i, 100n, false, [[11n, 14n]]))) return "many";
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
