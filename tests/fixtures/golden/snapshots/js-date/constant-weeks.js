/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatDate_skeleton_yw(value) {
  const date = coerceDate(value);
  const extendedYear = date.getUTCFullYear();
  const month = date.getUTCMonth();
  const leapYear = extendedYear % 4 === 0 && (extendedYear % 100 !== 0 || extendedYear % 400 === 0);
  let weekAnchor = Math.floor((275 * month + 5) / 9) + date.getUTCDate() + 6 - date.getUTCDay();
  if (month > 1) weekAnchor -= leapYear ? 1 : 2;
  let weekYear = extendedYear;
  if (weekAnchor > (leapYear ? 366 : 365)) {
    weekAnchor -= leapYear ? 366 : 365;
    weekYear = extendedYear + 1;
  }
  const weekNumber = Math.floor((weekAnchor - 1) / 7) + 1;
  return ("week " + String(weekNumber) + " of " + String(weekYear)).replace(/\d/g, localizeDigit);
}

const formatDate_skeleton_MMMMW_months_3 = ["जनवरी:", "फरवरी:", "मार्च:", "अप्रैल:", "मई", "जून:", "जुलाई:", "अगस्त:", "सितंबर:", "अक्तूबर:", "नवंबर:", "दिसंबर:"];
/**
 * @param {globalThis.Date | number | string} value
 * @returns {string}
 */
export function formatDate_skeleton_MMMMW(value) {
  const date = coerceDate(value);
  const month = date.getUTCMonth();
  const weekNumber = Math.floor((date.getUTCDate() + 5 - date.getUTCDay()) / 7) + 1;
  return ("week " + String(weekNumber) + " of " + formatDate_skeleton_MMMMW_months_3[month]).replace(/\d/g, localizeDigit);
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
    const dateOnly = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
    if (dateOnly) {
      const year = Number(dateOnly[1]);
      const month = Number(dateOnly[2]);
      const day = Number(dateOnly[3]);
      date = createUTCDate(year, month, day);
    } else {
      const dateTime =
        /^(\d{4})-(\d{2})-(\d{2})T\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?(?:Z|[+-]\d{2}:\d{2})?$/.exec(value);
      if (!dateTime) throwInvalidDate();
      validateDateFields(Number(dateTime[1]), Number(dateTime[2]), Number(dateTime[3]));
      const hasTimeZone = /(?:Z|[+-]\d{2}:\d{2})$/.test(value);
      date = new Date(hasTimeZone ? value : `${value}Z`);
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
 * @returns {globalThis.Date}
 */
function createUTCDate(year, month, day) {
  validateDateFields(year, month, day);
  const date = new Date(0);
  date.setUTCFullYear(year, month - 1, day);
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
  return "०१२३४५६७८९"[digit.charCodeAt(0) - 48];
}
