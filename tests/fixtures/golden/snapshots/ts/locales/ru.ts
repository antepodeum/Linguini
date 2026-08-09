import { email_input } from "./ru/email_input";
import type { Fruit, Size, Money, ShortDate } from "../shared";
import { selectBranch } from "../shared";

import { formatNumber, formatCurrency, formatDate, pluralRu } from "./ru/_runtime";

export type { Fruit, Size, Money, ShortDate } from "../shared";

type Gender = "male" | "female" | "neuter" | "other";

export { email_input };

const cart_label = "В корзине";

const __lgl_form_4672756974 = {
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | bigint | string) => selectBranch(pluralRu(value), { one: "яблоко", few: "яблока", _: "яблок" }), gen: (value: number | bigint | string) => selectBranch(pluralRu(value), { one: "яблока", _: "яблок" }), display: { short: "ябл.", long: "спелое яблоко" } },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | bigint | string) => selectBranch(pluralRu(value), { one: "груша", few: "груши", _: "груш" }), gen: (value: number | bigint | string) => selectBranch(pluralRu(value), { one: "груши", _: "груш" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | bigint | string) => selectBranch(pluralRu(value), { one: "апельсин", few: "апельсина", _: "апельсинов" }), gen: (value: number | bigint | string) => selectBranch(pluralRu(value), { one: "апельсина", _: "апельсинов" }) },
} as const;

function Delivered(__lgl_p0: number | bigint | string, __lgl_p1: Gender): string {
  return selectBranch(pluralRu(__lgl_p0), { one: (): string => selectBranch(String(__lgl_p1), { male: (): string => "Доставлен", female: (): string => "Доставлена", neuter: (): string => "Доставлено", _: (): string => "Доставлено" })(), _: (): string => "Доставлены" })();
}

function SizeAdj(__lgl_p0: Size, __lgl_p1: number | bigint | string, __lgl_p2: Gender): string {
  return selectBranch(String(__lgl_p0), { small: (): string => selectBranch(pluralRu(__lgl_p1), { one: (): string => selectBranch(String(__lgl_p2), { male: (): string => "маленький", female: (): string => "маленькая", neuter: (): string => "маленькое", _: (): string => "маленький" })(), _: (): string => "маленьких" })(), big: (): string => selectBranch(pluralRu(__lgl_p1), { one: (): string => selectBranch(String(__lgl_p2), { male: (): string => "большой", female: (): string => "большая", neuter: (): string => "большое", _: (): string => "большой" })(), _: (): string => "больших" })(), _: (): string => "обычные" })();
}

function DeliveryNote(__lgl_p0: number | bigint | string, __lgl_p1: Gender, item: string): string {
  return selectBranch(pluralRu(__lgl_p0), { one: (): string => selectBranch(String(__lgl_p1), { female: (): string => "Доставлена " + String(item), _: (): string => "Доставлен " + String(item) })(), _: (): string => "Доставлены " + String(item) })();
}

/** Displayed on the product delivery confirmation card. */
export function delivery(fruit: Fruit, size: Size, count: number | bigint | string): string {
  return String(Delivered(count, __lgl_form_4672756974[fruit].Gender)) + " " + String(SizeAdj(size, count, __lgl_form_4672756974[fruit].Gender)) + " " + String(__lgl_form_4672756974[fruit].nom(count));
}

/** Shown near cart item count. */
export function counted(count: number | bigint | string, fruit: Fruit): string {
  return String(cart_label) + " " + String(formatNumber(count)) + " " + String(__lgl_form_4672756974[fruit].nom(count));
}

export function price(amount: Money, date: ShortDate): string {
  return "Цена " + String(formatCurrency(amount, 2, 0, { code: "RUB" })) + " на " + String(formatDate(date, { style: "short" }));
}

const lgl = {
  delivery,
  counted,
  price,
  email_input,
} as const;

export default lgl;
