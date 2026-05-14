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
  apple: { Gender: "neuter", emoji: "🍎", nom: (value) => selectBranch(pluralRu(value), { one: "яблоко", few: "яблока", _: "яблок" }), gen: (value) => selectBranch(pluralRu(value), { one: "яблока", _: "яблок" }), display: { short: "ябл.", long: "спелое яблоко" } },
  pear: { Gender: "female", emoji: "🍐", nom: (value) => selectBranch(pluralRu(value), { one: "груша", few: "груши", _: "груш" }), gen: (value) => selectBranch(pluralRu(value), { one: "груши", _: "груш" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value) => selectBranch(pluralRu(value), { one: "апельсин", few: "апельсина", _: "апельсинов" }), gen: (value) => selectBranch(pluralRu(value), { one: "апельсина", _: "апельсинов" }) },
};

function Delivered(p0, p1) {
  return selectBranch(pluralRu(p0), { one: selectBranch(String(p1), { male: "Доставлен", female: "Доставлена", neuter: "Доставлено", _: "Доставлено" }), _: "Доставлены" });
}

function SizeAdj(p0, p1, p2) {
  return selectBranch(String(p0), { small: selectBranch(pluralRu(p1), { one: selectBranch(String(p2), { male: "маленький", female: "маленькая", neuter: "маленькое", _: "маленький" }), _: "маленьких" }), big: selectBranch(pluralRu(p1), { one: selectBranch(String(p2), { male: "большой", female: "большая", neuter: "большое", _: "большой" }), _: "больших" }), _: "обычные" });
}

function DeliveryNote(item, p1, p2) {
  return selectBranch(pluralRu(p1), { one: selectBranch(String(p2), { female: "Доставлена " + String(item), _: "Доставлен " + String(item) }), _: "Доставлены " + String(item) });
}

/**  Displayed on the product delivery confirmation card. */
export function delivery(fruit, size, count) {
  return String(Delivered(count, FruitForms[fruit].Gender)) + " " + String(SizeAdj(size, count, FruitForms[fruit].Gender)) + " " + String(FruitForms[fruit].nom(count));
}

/**  Shown near cart item count. */
export function counted(count, fruit) {
  return "В корзине " + String(count) + " " + String(FruitForms[fruit].nom(count));
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
