/**
 * @param {number} value
 * @param {number} length
 * @returns {string}
 */
function padNumber(value, length) {
  return String(value).padStart(length, "0");
}

/**
 * @param {Date | number | string} value
 * @returns {Date}
 */
function coerceDate(value) {
  /** @type {Date} */
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
      createUTCDate(Number(dateTime[1]), Number(dateTime[2]), Number(dateTime[3]));
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
 * @returns {Date}
 */
function createUTCDate(year, month, day) {
  const date = new Date(0);
  date.setUTCHours(0, 0, 0, 0);
  date.setUTCFullYear(year, month - 1, day);
  if (
    date.getUTCFullYear() !== year ||
    date.getUTCMonth() !== month - 1 ||
    date.getUTCDate() !== day
  ) {
    throwInvalidDate();
  }
  return date;
}

/** @returns {never} */
function throwInvalidDate() {
  throw new RangeError("Linguini: invalid date value");
}
