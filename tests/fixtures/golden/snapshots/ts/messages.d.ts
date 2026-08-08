import type { Fruit, Size, Money, ShortDate } from "./shared";

export type LinguiniMessages = {
  /** Displayed on the product delivery confirmation card. */
  readonly delivery: (fruit: Fruit, size: Size, count: number | bigint | string) => string;
  /** Shown near cart item count. */
  readonly counted: (count: number | bigint | string, fruit: Fruit) => string;
  readonly price: (amount: Money, date: ShortDate) => string;
  readonly email_input: {
    readonly label: string;
    readonly placeholder: string;
    readonly aria: string;
  };
};
