import { email_input } from "./ru/email_input";
import type { Fruit, Size, Money, ShortDate } from "../shared";
import { normalizeMessageArgs } from "../shared";

import { formatNumber, formatCurrency, formatDate } from "./ru/_runtime";
import { cart_label, __lgl_form_4672756974, Delivered, SizeAdj, DeliveryNote } from "./ru/_globals";

export type { Fruit, Size, Money, ShortDate } from "../shared";

export { email_input };

/**
 * Displayed on the product delivery confirmation card.
 *
 * @param {Fruit} fruit
 * @param {Size} size
 * @param {number | bigint | string} count
 * @returns {string}
 */
export function delivery(fruit: Fruit, size: Size, count: number | bigint | string): string;
/**
 * Displayed on the product delivery confirmation card.
 *
 * @param {{ fruit: Fruit; size: Size; count: number | bigint | string }} args
 * @returns {string}
 */
export function delivery(args: { fruit: Fruit; size: Size; count: number | bigint | string }): string;
export function delivery(...__lgl_args: [fruit: Fruit, size: Size, count: number | bigint | string] | [args: { fruit: Fruit; size: Size; count: number | bigint | string }]): string {
  const [fruit, size, count] = normalizeMessageArgs(__lgl_args, ["fruit", "size", "count"]) as [Fruit, Size, number | bigint | string];
  return String(Delivered(count, __lgl_form_4672756974[fruit].Gender)) + " " + String(SizeAdj(size, count, __lgl_form_4672756974[fruit].Gender)) + " " + String(__lgl_form_4672756974[fruit].nom(count));
}

/**
 * Shown near cart item count.
 *
 * @param {number | bigint | string} count
 * @param {Fruit} fruit
 * @returns {string}
 */
export function counted(count: number | bigint | string, fruit: Fruit): string;
/**
 * Shown near cart item count.
 *
 * @param {{ count: number | bigint | string; fruit: Fruit }} args
 * @returns {string}
 */
export function counted(args: { count: number | bigint | string; fruit: Fruit }): string;
export function counted(...__lgl_args: [count: number | bigint | string, fruit: Fruit] | [args: { count: number | bigint | string; fruit: Fruit }]): string {
  const [count, fruit] = normalizeMessageArgs(__lgl_args, ["count", "fruit"]) as [number | bigint | string, Fruit];
  return String(cart_label) + " " + String(formatNumber(count)) + " " + String(__lgl_form_4672756974[fruit].nom(count));
}

/**
 * @param {Money} amount
 * @param {ShortDate} date
 * @returns {string}
 */
export function price(amount: Money, date: ShortDate): string;
/**
 * @param {{ amount: Money; date: ShortDate }} args
 * @returns {string}
 */
export function price(args: { amount: Money; date: ShortDate }): string;
export function price(...__lgl_args: [amount: Money, date: ShortDate] | [args: { amount: Money; date: ShortDate }]): string {
  const [amount, date] = normalizeMessageArgs(__lgl_args, ["amount", "date"]) as [Money, ShortDate];
  return "Цена " + String(formatCurrency(amount, 2, 0, { code: "RUB" })) + " на " + String(formatDate(date, { style: "short" }));
}

const lgl = {
  delivery,
  counted,
  price,
  email_input,
} as const;

export default lgl;
