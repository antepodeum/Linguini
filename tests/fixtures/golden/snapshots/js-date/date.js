/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatFlexible(value) {
  const date = coerceDate(value);
  const hours = date.getUTCHours();
  const minutes = date.getUTCMinutes();
  const seconds = date.getUTCSeconds();
  const milliseconds = date.getUTCMilliseconds();
  const exactTime = minutes === 0 && seconds === 0 && milliseconds === 0;
  let dayPeriod;
  if (hours === 0 && exactTime) dayPeriod = "midnight";
  else if (hours === 12 && exactTime) dayPeriod = "noon";
  else if (hours < 12) dayPeriod = "in the morning";
  else if (hours < 18) dayPeriod = "in the afternoon";
  else if (hours < 21) dayPeriod = "in the evening";
  else dayPeriod = "at night";
  return String(hours % 12 || 12) + ":" + padNumber(minutes, 2) + ":" + padNumber(seconds, 2) + "." + padNumber(milliseconds, 3) + " " + dayPeriod;
}

/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatFixed(value) {
  const date = coerceDate(value);
  const hours = date.getUTCHours();
  const minutes = date.getUTCMinutes();
  const exactTime = minutes === 0;
  let dayPeriod;
  if (hours === 0 && exactTime) dayPeriod = "полночь";
  else if (hours === 12 && exactTime) dayPeriod = "полдень";
  else if (hours < 12) dayPeriod = "AM";
  else dayPeriod = "PM";
  return padNumber(hours, 2) + ":" + padNumber(minutes, 2) + " " + dayPeriod;
}

/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatNarrow(value) {
  const date = coerceDate(value);
  const hours = date.getUTCHours();
  let dayPeriod;
  if (hours < 1) dayPeriod = "منتصف الليل";
  else if (hours < 3) dayPeriod = "ليلاً";
  else if (hours < 6) dayPeriod = "فجرًا";
  else if (hours < 12) dayPeriod = "صباحًا";
  else if (hours < 13) dayPeriod = "ظهرًا";
  else if (hours < 18) dayPeriod = "بعد الظهر";
  else dayPeriod = "مساءً";
  return dayPeriod;
}

/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatYear(value) {
  const date = coerceDate(value);
  const extendedYear = date.getUTCFullYear();
  const year = extendedYear > 0 ? extendedYear : 1 - extendedYear;
  return String(year) + " " + padNumber(year % 100, 2) + " " + padNumber(year, 4) + " " + (extendedYear > 0 ? "AD" : "BC");
}

/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatLocalized(value) {
  const date = coerceDate(value);
  let year = date.getUTCFullYear();
  if (year <= 0) year = 1 - year;
  return (padNumber(year, 4) + " " + padNumber(date.getUTCMonth() + 1, 2) + " " + padNumber(date.getUTCDate(), 2) + " " + padNumber(date.getUTCHours(), 2) + ":" + padNumber(date.getUTCMinutes(), 2) + ":" + padNumber(date.getUTCSeconds(), 2) + "." + padNumber(date.getUTCMilliseconds(), 3)).replace(/\d/g, localizeDigit);
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
 * @param {string} digit
 * @returns {string}
 */
function localizeDigit(digit) {
  return "٠١٢٣٤٥٦٧٨٩"[digit.charCodeAt(0) - 48];
}
