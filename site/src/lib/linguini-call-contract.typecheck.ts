import { l } from "$lib/generated/linguini/svelte";
import type { Fruit, Size } from "$lib/generated/linguini/locales/en";

declare const fruit: Fruit;
declare const size: Size;
declare const count: number;
declare const amount: number;
declare const date: string;

// Existing positional calls remain source-compatible.
l.main.playground.sentence(fruit, size, count, amount, date);

// Named calls are order-independent and use the source parameter names.
l.main.playground.sentence({ date, amount, count, size, fruit });
l.main.playground.date_format(new Date());
l.main.playground.date_format({ date: new Date() });

// @ts-expect-error named calls require every message parameter
l.main.playground.sentence({ fruit, size, count, amount });

// @ts-expect-error named calls reject unknown object-literal properties
l.main.playground.sentence({ fruit, size, count, amount, date, extra: true });

// @ts-expect-error parameterless messages remain string values, not functions
const parameterlessMessage: () => string = l.main.hero.title;
void parameterlessMessage;
