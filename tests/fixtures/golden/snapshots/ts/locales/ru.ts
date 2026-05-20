import { formatCurrency, formatDate, formatNumber, selectBranch } from "../shared";

function pluralRu(value: number | string): string {
  const operands = pluralOperands(value);
  if (((operands.v === 0) && ((operands.i % 10) === 1) && !((operands.i % 100) === 11))) return "one";
  if (((operands.v === 0) && (((operands.i % 10) >= 2 && (operands.i % 10) <= 4)) && !(((operands.i % 100) >= 12 && (operands.i % 100) <= 14)))) return "few";
  if (((operands.v === 0) && ((operands.i % 10) === 0)) || ((operands.v === 0) && (((operands.i % 10) >= 5 && (operands.i % 10) <= 9))) || ((operands.v === 0) && (((operands.i % 100) >= 11 && (operands.i % 100) <= 14)))) return "many";
  return "other";
}

function pluralOperands(value: number | string) {
  const source = String(value).replace(/^[+-]/, "");
  const [integer, fraction = ""] = source.split(".");
  const trimmedFraction = fraction.replace(/0+$/, "");

  return {
    n: Number(source),
    i: Number(integer),
    v: fraction.length,
    w: trimmedFraction.length,
    f: fraction === "" ? 0 : Number(fraction),
    t: trimmedFraction === "" ? 0 : Number(trimmedFraction),
    c: 0,
    e: 0,
  };
}

const FORMATTER_DATA = { locale: "ru", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d MMMM y 'г'.", long: "d MMMM y 'г'.", medium: "d MMM y 'г'.", short: "dd.MM.y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } };

export type Fruit = "apple" | "pear" | "orange";

export type Size = "small" | "big";

export type Money = number;

export type ShortDate = string;

const FruitForms = {
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "яблоко", few: "яблока", _: "яблок" }), gen: (value: number | string) => selectBranch(pluralRu(value), { one: "яблока", _: "яблок" }), display: { short: "ябл.", long: "спелое яблоко" } },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "груша", few: "груши", _: "груш" }), gen: (value: number | string) => selectBranch(pluralRu(value), { one: "груши", _: "груш" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "апельсин", few: "апельсина", _: "апельсинов" }), gen: (value: number | string) => selectBranch(pluralRu(value), { one: "апельсина", _: "апельсинов" }) },
} as const;

function Delivered(p0: string | number, p1: string | number): string {
  return selectBranch(pluralRu(p0), { one: selectBranch(String(p1), { male: "Доставлен", female: "Доставлена", neuter: "Доставлено", _: "Доставлено" }), _: "Доставлены" });
}

function SizeAdj(p0: string | number, p1: string | number, p2: string | number): string {
  return selectBranch(String(p0), { small: selectBranch(pluralRu(p1), { one: selectBranch(String(p2), { male: "маленький", female: "маленькая", neuter: "маленькое", _: "маленький" }), _: "маленьких" }), big: selectBranch(pluralRu(p1), { one: selectBranch(String(p2), { male: "большой", female: "большая", neuter: "большое", _: "большой" }), _: "больших" }), _: "обычные" });
}

function DeliveryNote(item: string | number, p1: string | number, p2: string | number): string {
  return selectBranch(pluralRu(p1), { one: selectBranch(String(p2), { female: "Доставлена " + String(item), _: "Доставлен " + String(item) }), _: "Доставлены " + String(item) });
}

/**  Displayed on the product delivery confirmation card. */
export function delivery(fruit: Fruit, size: Size, count: number): string {
  return String(Delivered(count, FruitForms[fruit].Gender)) + " " + String(SizeAdj(size, count, FruitForms[fruit].Gender)) + " " + String(FruitForms[fruit].nom(count));
}

/**  Shown near cart item count. */
export function counted(count: number, fruit: Fruit): string {
  return "В корзине " + String(formatNumber(count, FORMATTER_DATA)) + " " + String(FruitForms[fruit].nom(count));
}

export function price(amount: Money, date: ShortDate): string {
  return "Цена " + String(formatCurrency(amount, FORMATTER_DATA, { code: "RUB" })) + " на " + String(formatDate(date, FORMATTER_DATA, { style: "short" }));
}

export const email_input = {
  label: "Email",
  placeholder: "name@example.com",
  aria: "Адрес электронной почты",
} as const;

const lgl = {
  delivery,
  counted,
  price,
  email_input,
} as const;

export default lgl;
