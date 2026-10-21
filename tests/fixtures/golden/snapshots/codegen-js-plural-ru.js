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
