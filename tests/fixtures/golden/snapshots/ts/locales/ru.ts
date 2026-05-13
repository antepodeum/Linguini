import { formatCurrency, formatDate, selectBranch } from "../shared";

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

export type Fruit = "apple" | "pear" | "orange";

export type Size = "small" | "big";

export type Money = number;

export type ShortDate = string;

const FruitForms = {
  apple: { gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "яблоко", few: "яблока", many: "яблок", other: "яблока" }), gen: (value: number | string) => selectBranch(pluralRu(value), { one: "яблока", few: "яблок", many: "яблок", other: "яблока" }), display: { short: "ябл.", long: "спелое яблоко" } },
  pear: { gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "груша", few: "груши", many: "груш", other: "груши" }), gen: (value: number | string) => selectBranch(pluralRu(value), { one: "груши", few: "груш", many: "груш", other: "груши" }) },
  orange: { gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "апельсин", few: "апельсина", many: "апельсинов", other: "апельсина" }), gen: (value: number | string) => selectBranch(pluralRu(value), { one: "апельсина", few: "апельсинов", many: "апельсинов", other: "апельсина" }) },
} as const;

const SizeForms = {
  small: (gender: string) => selectBranch(gender, { male: "маленький", female: "маленькая", neuter: "маленькое", other: "маленький" }),
  big: (gender: string) => selectBranch(gender, { male: "большой", female: "большая", neuter: "большое", other: "большой" }),
} as const;

function delivered(gender: string, plural: string): string {
  if (gender === "male" && plural === "one") return "Доставлен";
  if (gender === "female" && plural === "one") return "Доставлена";
  if (gender === "neuter" && plural === "one") return "Доставлено";
  if (gender === "other" && plural === "one") return "Доставлено";
  return "Доставлено";
}

function size_label(size: string, gender: string, plural: string): string {
  if (size === "small" && gender === "male" && plural === "one") return "маленький";
  if (size === "small" && gender === "female" && plural === "one") return "маленькая";
  if (size === "small" && gender === "neuter" && plural === "one") return "маленькое";
  if (size === "small" && gender === "other" && plural === "one") return "маленький";
  if (size === "small" && gender === "female" && plural === "few") return "маленькие";
  if (size === "small" && gender === "male" && plural === "few") return "маленьких";
  if (size === "small" && gender === "neuter" && plural === "few") return "маленьких";
  if (size === "small" && gender === "other" && plural === "few") return "маленьких";
  if (size === "small" && gender === "male" && plural === "many") return "маленьких";
  if (size === "small" && gender === "female" && plural === "many") return "маленьких";
  if (size === "small" && gender === "neuter" && plural === "many") return "маленьких";
  if (size === "small" && gender === "other" && plural === "many") return "маленьких";
  if (size === "big" && gender === "male" && plural === "one") return "большой";
  if (size === "big" && gender === "female" && plural === "one") return "большая";
  if (size === "big" && gender === "neuter" && plural === "one") return "большое";
  if (size === "big" && gender === "other" && plural === "one") return "большой";
  if (size === "big" && gender === "female" && plural === "few") return "большие";
  if (size === "big" && gender === "male" && plural === "few") return "больших";
  if (size === "big" && gender === "neuter" && plural === "few") return "больших";
  if (size === "big" && gender === "other" && plural === "few") return "больших";
  if (size === "big" && gender === "male" && plural === "many") return "больших";
  if (size === "big" && gender === "female" && plural === "many") return "больших";
  if (size === "big" && gender === "neuter" && plural === "many") return "больших";
  if (size === "big" && gender === "other" && plural === "many") return "больших";
  return "обычные";
}

/**  Displayed on the product delivery confirmation card. */
export function delivery(fruit: Fruit, size: Size, count: number): string {
  return String(delivered(FruitForms[fruit].gender, pluralRu(count))) + " " + String(size_label(size, FruitForms[fruit].gender, pluralRu(count))) + " " + String(FruitForms[fruit].nom(count));
}

/**  Shown near cart item count. */
export function counted(count: number, fruit: Fruit): string {
  return "В корзине " + String(count) + " " + String(FruitForms[fruit].nom(count));
}

export function price(amount: Money, date: ShortDate): string {
  return "Цена " + String(formatCurrency(amount, "ru", { code: "RUB" })) + " на " + String(formatDate(date, "ru", { style: "short" }));
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
