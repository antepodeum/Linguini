/**
 * @param {Date | number | string} value
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
 * @param {Date | number | string} value
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
 * @param {Date | number | string} value
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
