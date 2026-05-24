import { email_input } from "./ru/email_input";

import type { Fruit, Size, Money, ShortDate } from "../shared";

export type { Fruit, Size, Money, ShortDate } from "../shared";

export declare const email_input: typeof email_input;

/**  Displayed on the product delivery confirmation card. */
export declare function delivery(fruit: Fruit, size: Size, count: number): string;

/**  Shown near cart item count. */
export declare function counted(count: number, fruit: Fruit): string;

export declare function price(amount: Money, date: ShortDate): string;

declare const lgl: {
  readonly delivery: typeof delivery;
  readonly counted: typeof counted;
  readonly price: typeof price;
  readonly email_input: typeof email_input;
};

export default lgl;
