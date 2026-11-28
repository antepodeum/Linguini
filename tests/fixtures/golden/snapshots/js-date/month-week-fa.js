const formatDate_skeleton_MMMMW_standalone_months_4 = ["ژانویه", "فوریه", "مارس", "آوریل", "مه", "ژوئن", "ژوئیه", "اوت", "سپتامبر", "اکتبر", "نوامبر", "دسامبر"];
/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatDate_skeleton_MMMMW(value) {
  const date = coerceDate(value);
  const month = date.getUTCMonth();
  const weekNumber = Math.floor((date.getUTCDate() + 5 - ((date.getUTCDay() + 1) % 7)) / 7) + 1;
  return ("هفتهٔ " + String(weekNumber) + "م " + formatDate_skeleton_MMMMW_standalone_months_4[month]).replace(/\d/g, localizeDigit);
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
    const fields = /^(\d{4})-(\d{2})-(\d{2})(?:T\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?(Z|[+-]\d{2}:\d{2})?)?$/.exec(value);
    if (!fields) throwInvalidDate();
    const year = Number(fields[1]);
    const month = Number(fields[2]);
    const day = Number(fields[3]);
    validateDateFields(year, month, day);
    if (value.length === 10) {
      date = new Date(0);
      date.setUTCFullYear(year, month - 1, day);
    } else {
      date = new Date(fields[4] ? value : `${value}Z`);
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
 * @param {string} digit
 * @returns {string}
 */
function localizeDigit(digit) {
  return "۰۱۲۳۴۵۶۷۸۹"[digit.charCodeAt(0) - 48];
}
