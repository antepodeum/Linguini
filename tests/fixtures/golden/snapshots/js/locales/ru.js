import { formatCurrency, formatDate, selectBranch } from "../shared.js";

function pluralRu(value) {
  const operands = pluralOperands(value);
  if (operands.v === 0 && operands.i % 10 === 1 && !(operands.i % 100 === 11)) return "one";
  if (operands.v === 0 && operands.i % 10 >= 2 && operands.i % 10 <= 4 && !(operands.i % 100 >= 12 && operands.i % 100 <= 14)) return "few";
  if ((operands.v === 0 && operands.i % 10 === 0) || (operands.v === 0 && operands.i % 10 >= 5 && operands.i % 10 <= 9) || (operands.v === 0 && operands.i % 100 >= 11 && operands.i % 100 <= 14)) return "many";
  return "other";
}

function pluralOperands(value) {
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

const FruitForms = {
  apple: { gender: "neuter", emoji: "🍎", nom: (value) => selectBranch(pluralRu(value), { one: "яблоко", few: "яблока", many: "яблок", other: "яблока" }), gen: (value) => selectBranch(pluralRu(value), { one: "яблока", few: "яблок", many: "яблок", other: "яблока" }), display: { short: "ябл.", long: "спелое яблоко" } },
  pear: { gender: "female", emoji: "🍐", nom: (value) => selectBranch(pluralRu(value), { one: "груша", few: "груши", many: "груш", other: "груши" }), gen: (value) => selectBranch(pluralRu(value), { one: "груши", few: "груш", many: "груш", other: "груши" }) },
  orange: { gender: "male", emoji: "🍊", nom: (value) => selectBranch(pluralRu(value), { one: "апельсин", few: "апельсина", many: "апельсинов", other: "апельсина" }), gen: (value) => selectBranch(pluralRu(value), { one: "апельсина", few: "апельсинов", many: "апельсинов", other: "апельсина" }) },
};

const SizeForms = {
  small: (gender) => selectBranch(gender, { male: "маленький", female: "маленькая", neuter: "маленькое", other: "маленький" }),
  big: (gender) => selectBranch(gender, { male: "большой", female: "большая", neuter: "большое", other: "большой" }),
};

function delivered(gender) {
  if (gender === "male") return "Доставлен";
  if (gender === "female") return "Доставлена";
  if (gender === "neuter") return "Доставлено";
  if (gender === "other") return "Доставлено";
  return "";
}

/**  Displayed on the product delivery confirmation card. */
export function delivery(fruit, size, count) {
  return String(delivered(FruitForms[fruit].gender)) + " " + String(SizeForms[size](FruitForms[fruit].gender)) + " " + String(FruitForms[fruit].nom(count));
}

/**  Shown near cart item count. */
export function counted(count, fruit) {
  return "В корзине " + String(count) + " " + String(FruitForms[fruit].gen(count));
}

export function price(amount, date) {
  return "Цена " + String(formatCurrency(amount, "ru", { code: "RUB" })) + " на " + String(formatDate(date, "ru", { style: "short" }));
}

export const email_input = {
  label: "Email",
  placeholder: "name@example.com",
  aria: "Адрес электронной почты",
};

const lgl = {
  delivery,
  counted,
  price,
  email_input,
};

export default lgl;
