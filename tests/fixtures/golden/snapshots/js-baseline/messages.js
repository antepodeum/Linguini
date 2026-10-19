/** @typedef {import("./shared.js").Fruit} Fruit */
/** @typedef {import("./shared.js").Size} Size */
/** @typedef {import("./shared.js").Money} Money */
/** @typedef {import("./shared.js").ShortDate} ShortDate */
/** @typedef {{
  readonly delivery: ((fruit: Fruit, size: Size, count: number | bigint | string) => string) & ((args: { fruit: Fruit; size: Size; count: number | bigint | string }) => string);
  readonly counted: ((count: number | bigint | string, fruit: Fruit) => string) & ((args: { count: number | bigint | string; fruit: Fruit }) => string);
  readonly price: ((amount: Money, date: ShortDate) => string) & ((args: { amount: Money; date: ShortDate }) => string);
  readonly email_input: {
    readonly label: string;
    readonly placeholder: string;
    readonly aria: string;
  };
}} LinguiniMessages */
export {};
