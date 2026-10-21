import { pluralRu } from "./_runtime.js";

/** @typedef {import("../../shared.js").Fruit} Fruit */
/** @typedef {import("../../shared.js").Size} Size */
/** @typedef {import("../../shared.js").Money} Money */
/** @typedef {import("../../shared.js").ShortDate} ShortDate */

/** @typedef {"male" | "female" | "neuter" | "other"} Gender */
const cart_label = "В корзине";

const __lgl_form_Fruit = {
  apple: { Gender: /** @type {const} */ ("neuter"), emoji: /** @type {const} */ ("🍎"), nom: /** @type {(value: number | bigint | string) => string} */ ((value) => {
  switch (pluralRu(value)) {
    case "one":
      return "яблоко";
    case "few":
      return "яблока";
    default:
      return "яблок";
  }
}), gen: /** @type {(value: number | bigint | string) => string} */ ((value) => {
  if (pluralRu(value) === "one") return "яблока";
  return "яблок";
}), display: { short: /** @type {const} */ ("ябл."), long: /** @type {const} */ ("спелое яблоко") } },
  pear: { Gender: /** @type {const} */ ("female"), emoji: /** @type {const} */ ("🍐"), nom: /** @type {(value: number | bigint | string) => string} */ ((value) => {
  switch (pluralRu(value)) {
    case "one":
      return "груша";
    case "few":
      return "груши";
    default:
      return "груш";
  }
}), gen: /** @type {(value: number | bigint | string) => string} */ ((value) => {
  if (pluralRu(value) === "one") return "груши";
  return "груш";
}) },
  orange: { Gender: /** @type {const} */ ("male"), emoji: /** @type {const} */ ("🍊"), nom: /** @type {(value: number | bigint | string) => string} */ ((value) => {
  switch (pluralRu(value)) {
    case "one":
      return "апельсин";
    case "few":
      return "апельсина";
    default:
      return "апельсинов";
  }
}), gen: /** @type {(value: number | bigint | string) => string} */ ((value) => {
  if (pluralRu(value) === "one") return "апельсина";
  return "апельсинов";
}) },
};

/**
 * @param {number | bigint | string} plural
 * @param {Gender} gender
 * @returns {string}
 */
function Delivered(plural, gender) {
  if (pluralRu(plural) !== "one") return "Доставлены";
  switch (String(gender)) {
    case "male":
      return "Доставлен";
    case "female":
      return "Доставлена";
    default:
      return "Доставлено";
  }
}

/**
 * @param {Size} size
 * @param {number | bigint | string} plural
 * @param {Gender} gender
 * @returns {string}
 */
function SizeAdj(size, plural, gender) {
  switch (String(size)) {
    case "small":
      if (pluralRu(plural) !== "one") return "маленьких";
      switch (String(gender)) {
        case "female":
          return "маленькая";
        case "neuter":
          return "маленькое";
        default:
          return "маленький";
      }
    case "big":
      if (pluralRu(plural) !== "one") return "больших";
      switch (String(gender)) {
        case "female":
          return "большая";
        case "neuter":
          return "большое";
        default:
          return "большой";
      }
    default:
      return "обычные";
  }
}

/**
 * @param {number | bigint | string} plural
 * @param {Gender} gender
 * @param {string} item
 * @returns {string}
 */
function DeliveryNote(plural, gender, item) {
  if (pluralRu(plural) !== "one") return "Доставлены " + String(item);
  if (String(gender) === "female") return "Доставлена " + String(item);
  return "Доставлен " + String(item);
}

export { cart_label, __lgl_form_Fruit, Delivered, SizeAdj, DeliveryNote };
