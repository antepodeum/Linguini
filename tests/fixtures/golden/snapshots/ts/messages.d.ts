import type { Fruit, Size, Money, ShortDate } from "./shared";

export type LinguiniMessages = {
  readonly delivery: {
    /** Displayed on the product delivery confirmation card. */
    (fruit: Fruit, size: Size, count: number | bigint | string): string;
    /** Displayed on the product delivery confirmation card. */
    (args: { fruit: Fruit; size: Size; count: number | bigint | string }): string;
  };
  readonly counted: {
    /** Shown near cart item count. */
    (count: number | bigint | string, fruit: Fruit): string;
    /** Shown near cart item count. */
    (args: { count: number | bigint | string; fruit: Fruit }): string;
  };
  readonly price: {
    (amount: Money, date: ShortDate): string;
    (args: { amount: Money; date: ShortDate }): string;
  };
  readonly email_input: {
    readonly label: string;
    readonly placeholder: string;
    readonly aria: string;
  };
};
