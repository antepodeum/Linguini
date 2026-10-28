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
  const integerTail = decimalPosition <= 0 || decimalPosition - digitCount >= 2 ? 0
    : Number(digits.slice(Math.max(0, decimalPosition - 2), decimalPosition)
      + (decimalPosition > digitCount ? "0".repeat(decimalPosition - digitCount) : ""));
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
