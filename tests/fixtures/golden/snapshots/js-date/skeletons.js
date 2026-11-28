const formatDate_skeleton_GyMMMd_months_3 = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatDate_skeleton_GyMMMd(value) {
  const date = coerceDate(value);
  const extendedYear = date.getUTCFullYear();
  const year = extendedYear > 0 ? extendedYear : 1 - extendedYear;
  return formatDate_skeleton_GyMMMd_months_3[date.getUTCMonth()] + " " + String(date.getUTCDate()) + ", " + String(year) + " " + (extendedYear > 0 ? "AD" : "BC");
}

/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatDate_skeleton_Hm(value) {
  const date = coerceDate(value);
  return padNumber(date.getUTCHours(), 2) + ":" + padNumber(date.getUTCMinutes(), 2);
}

/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatDate_skeleton_yMMMd(value) {
  const date = coerceDate(value);
  let year = date.getUTCFullYear();
  if (year <= 0) year = 1 - year;
  return formatDate_skeleton_GyMMMd_months_3[date.getUTCMonth()] + " " + String(date.getUTCDate()) + ", " + String(year);
}

const formatDate_skeleton_yQQQ_quarters_3 = ["Q1", "Q2", "Q3", "Q4"];
/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatDate_skeleton_yQQQ(value) {
  const date = coerceDate(value);
  let year = date.getUTCFullYear();
  if (year <= 0) year = 1 - year;
  return formatDate_skeleton_yQQQ_quarters_3[Math.floor(date.getUTCMonth() / 3)] + " " + String(year);
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
