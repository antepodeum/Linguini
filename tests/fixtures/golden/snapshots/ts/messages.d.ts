import type { Fruit, Size, Money, ShortDate } from "./shared";

export type LinguiniMessages = {
  readonly delivery: {
    /**
     * Displayed on the product delivery confirmation card.
     *
     * @param {Fruit} fruit
     * @param {Size} size
     * @param {number | bigint | string} count
     * @returns {string}
     */
    (fruit: Fruit, size: Size, count: number | bigint | string): string;
    /**
     * Displayed on the product delivery confirmation card.
     *
     * @param {{ fruit: Fruit; size: Size; count: number | bigint | string }} args
     * @returns {string}
     */
    (args: { fruit: Fruit; size: Size; count: number | bigint | string }): string;
  };
  readonly counted: {
    /**
     * Shown near cart item count.
     *
     * @param {number | bigint | string} count
     * @param {Fruit} fruit
     * @returns {string}
     */
    (count: number | bigint | string, fruit: Fruit): string;
    /**
     * Shown near cart item count.
     *
     * @param {{ count: number | bigint | string; fruit: Fruit }} args
     * @returns {string}
     */
    (args: { count: number | bigint | string; fruit: Fruit }): string;
  };
  readonly price: {
    /**
     * @param {Money} amount
     * @param {ShortDate} date
     * @returns {string}
     */
    (amount: Money, date: ShortDate): string;
    /**
     * @param {{ amount: Money; date: ShortDate }} args
     * @returns {string}
     */
    (args: { amount: Money; date: ShortDate }): string;
  };
  readonly email_input: {
    readonly label: string;
    readonly placeholder: string;
    readonly aria: string;
  };
};
