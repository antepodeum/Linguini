export type Fruit = "apple" | "pear" | "orange";

export type Size = "small" | "big";

export type Money = number;

export type ShortDate = string;

/**  Displayed on the product delivery confirmation card. */
export declare function delivery(fruit: Fruit, size: Size, count: number): string;

/**  Shown near cart item count. */
export declare function counted(count: number, fruit: Fruit): string;

export declare function price(amount: Money, date: ShortDate): string;

export declare const email_input: {
  readonly label: string;
  readonly placeholder: string;
  readonly aria: string;
};

declare const lgl: {
  readonly delivery: typeof delivery;
  readonly counted: typeof counted;
  readonly price: typeof price;
  readonly email_input: typeof email_input;
};

export default lgl;
