import { email_input } from "./ru/email_input.js";
import { normalizeMessageArgs } from "../shared.js";
import { formatCurrency_code_RUB, formatDate_style_short, formatNumber } from "./ru/_runtime.js";
import { cart_label, __lgl_form_Fruit, Delivered, SizeAdj } from "./ru/_globals.js";

/** @typedef {import("../shared.js").Fruit} Fruit */
/** @typedef {import("../shared.js").Size} Size */
/** @typedef {import("../shared.js").Money} Money */
/** @typedef {import("../shared.js").ShortDate} ShortDate */

export { email_input };

const __lgl_delivery_keys = ["fruit", "size", "count"];
const __lgl_counted_keys = ["count", "fruit"];
const __lgl_price_keys = ["amount", "date"];
/**
 * Displayed on the product delivery confirmation card.
 *
 * @overload
 * @param {Fruit} fruit
 * @param {Size} size
 * @param {number | bigint | string} count
 * @returns {string}
 */
/**
 * Displayed on the product delivery confirmation card.
 *
 * @overload
 * @param {{ fruit: Fruit; size: Size; count: number | bigint | string }} args
 * @returns {string}
 */
/**
 * @param {...*} __lgl_args
 * @returns {string}
 */
export function delivery(...__lgl_args) {
  const [fruit, size, count] = /** @type {[fruit: Fruit, size: Size, count: number | bigint | string]} */ (normalizeMessageArgs(__lgl_args, __lgl_delivery_keys));
  return Delivered(count, __lgl_form_Fruit[fruit].Gender) + " " + SizeAdj(size, count, __lgl_form_Fruit[fruit].Gender) + " " + __lgl_form_Fruit[fruit].nom(count);
}

/**
 * Shown near cart item count.
 *
 * @overload
 * @param {number | bigint | string} count
 * @param {Fruit} fruit
 * @returns {string}
 */
/**
 * Shown near cart item count.
 *
 * @overload
 * @param {{ count: number | bigint | string; fruit: Fruit }} args
 * @returns {string}
 */
/**
 * @param {...*} __lgl_args
 * @returns {string}
 */
export function counted(...__lgl_args) {
  const [count, fruit] = /** @type {[count: number | bigint | string, fruit: Fruit]} */ (normalizeMessageArgs(__lgl_args, __lgl_counted_keys));
  return String(cart_label) + " " + formatNumber(count) + " " + __lgl_form_Fruit[fruit].nom(count);
}

/**
 * @overload
 * @param {Money} amount
 * @param {ShortDate} date
 * @returns {string}
 */
/**
 * @overload
 * @param {{ amount: Money; date: ShortDate }} args
 * @returns {string}
 */
/**
 * @param {...*} __lgl_args
 * @returns {string}
 */
export function price(...__lgl_args) {
  const [amount, date] = /** @type {[amount: Money, date: ShortDate]} */ (normalizeMessageArgs(__lgl_args, __lgl_price_keys));
  return "Цена " + formatCurrency_code_RUB(amount) + " на " + formatDate_style_short(date);
}

const lgl = {
  delivery,
  counted,
  price,
  email_input,
};

export default lgl;
