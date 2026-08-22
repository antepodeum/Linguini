import { email_input } from "./ru/email_input";

import type { Fruit, Size, Money, ShortDate } from "../shared";

export type { Fruit, Size, Money, ShortDate } from "../shared";

export declare const email_input: typeof email_input;

/**
 * Displayed on the product delivery confirmation card.
 *
 * @param {Fruit} fruit
 * @param {Size} size
 * @param {number | bigint | string} count
 * @returns {string}
 */
export declare function delivery(fruit: Fruit, size: Size, count: number | bigint | string): string;
/**
 * Displayed on the product delivery confirmation card.
 *
 * @param {{ fruit: Fruit; size: Size; count: number | bigint | string }} args
 * @returns {string}
 */
export declare function delivery(args: { fruit: Fruit; size: Size; count: number | bigint | string }): string;

/**
 * Shown near cart item count.
 *
 * @param {number | bigint | string} count
 * @param {Fruit} fruit
 * @returns {string}
 */
export declare function counted(count: number | bigint | string, fruit: Fruit): string;
/**
 * Shown near cart item count.
 *
 * @param {{ count: number | bigint | string; fruit: Fruit }} args
 * @returns {string}
 */
export declare function counted(args: { count: number | bigint | string; fruit: Fruit }): string;

/**
 * @param {Money} amount
 * @param {ShortDate} date
 * @returns {string}
 */
export declare function price(amount: Money, date: ShortDate): string;
/**
 * @param {{ amount: Money; date: ShortDate }} args
 * @returns {string}
 */
export declare function price(args: { amount: Money; date: ShortDate }): string;

declare const lgl: {
  readonly delivery: typeof delivery;
  readonly counted: typeof counted;
  readonly price: typeof price;
  readonly email_input: typeof email_input;
};

export default lgl;
