const formatWeekYear_months_3 = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sept", "Oct", "Nov", "Dec"];
/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatWeekYear(value) {
  const date = coerceDate(value);
  const extendedYear = date.getUTCFullYear();
  const month = date.getUTCMonth();
  const weekYear = getWeekYear(date, extendedYear, month);
  const weekYearMagnitude = Math.abs(weekYear);
  const weekYearSign = weekYear < 0 ? "-" : "";
  const year = extendedYear > 0 ? extendedYear : 1 - extendedYear;
  return String(weekYear) + "|" + (weekYearSign + padNumber(weekYearMagnitude % 100, 2)) + "|" + (weekYearSign + padNumber(weekYearMagnitude, 4)) + "|" + String(year) + "|" + (extendedYear > 0 ? "AD" : "BC") + "|" + formatWeekYear_months_3[month] + "|" + padNumber(Math.floor(month / 3) + 1, 2);
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
 * @param {globalThis.Date} date
 * @param {number} year
 * @param {number} month
 * @returns {number}
 */
function getWeekYear(date, year, month) {
  if (month === 0) {
    const day = date.getUTCDate();
    if (day <= 3 && day + 3 - ((date.getUTCDay() + 6) % 7) < 1) return year - 1;
  }
  if (month === 11) {
    const day = date.getUTCDate();
    if (day >= 29 && day + 3 - ((date.getUTCDay() + 6) % 7) > 31) return year + 1;
  }
  return year;
}
