import type { Fruit, Size, Money, ShortDate } from "../../shared";
import { selectBranch } from "../../shared";

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

type GeneratedCurrencyFormatterOptions = { code?: string; accounting?: "true" | "false" };
type GeneratedDateFormatterOptions = { style?: "full" | "long" | "medium" | "short" };

function formatNumber(value: number | string): string {
  return formatGeneratedNumber(Number(value), "", "", undefined, undefined, 1, 0, 3, 3, undefined, ",", " ");
}

function formatCurrency(
  value: number | string,
  options: GeneratedCurrencyFormatterOptions = {},
): string {
  const symbol = currencySymbol(options.code ?? "USD");
  if (options.accounting === "true") {
    return formatGeneratedNumber(Number(value), "", " " + symbol + "", undefined, undefined, 1, 2, 2, 3, undefined, ",", " ");
  }
  return formatGeneratedNumber(Number(value), "", " " + symbol + "", undefined, undefined, 1, 2, 2, 3, undefined, ",", " ");
}

function currencySymbol(currency: string): string {
  return new Intl.NumberFormat("ru", { style: "currency", currency })
    .formatToParts(0)
    .find((part) => part.type === "currency")?.value ?? currency;
}

function formatDate(
  value: Date | number | string,
  options: GeneratedDateFormatterOptions = {},
): string {
  const date = coerceDate(value);
  switch (options.style ?? "medium") {
    case "full":
      return ["воскресенье", "понедельник", "вторник", "среда", "четверг", "пятница", "суббота"][date.getDay()] + ", " + String(date.getDate()) + " " + ["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"][date.getMonth()] + " " + String(date.getFullYear()) + " " + "г" + ".";
    case "long":
      return String(date.getDate()) + " " + ["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"][date.getMonth()] + " " + String(date.getFullYear()) + " " + "г" + ".";
    case "short":
      return padNumber(date.getDate(), 2) + "." + padNumber(date.getMonth() + 1, 2) + "." + String(date.getFullYear());
    default:
      return String(date.getDate()) + " " + ["янв.", "февр.", "мар.", "апр.", "мая", "июн.", "июл.", "авг.", "сент.", "окт.", "нояб.", "дек."][date.getMonth()] + " " + String(date.getFullYear()) + " " + "г" + ".";
  }
}

function formatGeneratedNumber(
  value: number,
  prefix: string,
  suffix: string,
  negativePrefix: string | undefined,
  negativeSuffix: string | undefined,
  minIntegerDigits: number,
  minFractionDigits: number,
  maxFractionDigits: number,
  primaryGroupSize: number | undefined,
  secondaryGroupSize: number | undefined,
  decimalSymbol: string,
  groupSymbol: string,
): string {
  if (!Number.isFinite(value)) return String(value);
  const negative = value < 0 || Object.is(value, -0);
  const rounded = roundToFractionDigits(Math.abs(value), maxFractionDigits);
  let [integer, fraction = ""] = rounded.toFixed(maxFractionDigits).split(".");
  integer = integer.padStart(minIntegerDigits, "0");
  fraction = trimOptionalFractionDigits(fraction, minFractionDigits);

  const grouped = groupIntegerDigits(integer, primaryGroupSize, secondaryGroupSize, groupSymbol);
  const formatted = fraction ? `${grouped}${decimalSymbol}${fraction}` : grouped;
  if (negative) return `${negativePrefix ?? `-${prefix}`}${formatted}${negativeSuffix ?? suffix}`;
  return `${prefix}${formatted}${suffix}`;
}

function roundToFractionDigits(value: number, digits: number): number {
  if (digits <= 0) return Math.round(value);
  const factor = 10 ** digits;
  return Math.round(value * factor) / factor;
}

function trimOptionalFractionDigits(fraction: string, minDigits: number): string {
  while (fraction.length > minDigits && fraction.endsWith("0")) {
    fraction = fraction.slice(0, -1);
  }
  return fraction;
}

function groupIntegerDigits(
  integer: string,
  primaryGroupSize: number | undefined,
  secondaryGroupSize: number | undefined,
  groupSymbol: string,
): string {
  if (!primaryGroupSize || integer.length <= primaryGroupSize) return integer;
  const groups: string[] = [];
  let end = integer.length;
  let groupSize = primaryGroupSize;
  while (end > 0) {
    const start = Math.max(0, end - groupSize);
    groups.unshift(integer.slice(start, end));
    end = start;
    groupSize = secondaryGroupSize ?? primaryGroupSize;
  }
  return groups.join(groupSymbol);
}

function padNumber(value: number, length: number): string {
  return String(value).padStart(length, "0");
}

function coerceDate(value: Date | number | string): Date {
  if (value instanceof Date) return value;
  if (typeof value === "string") {
    const dateOnly = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
    if (dateOnly) {
      return new Date(Number(dateOnly[1]), Number(dateOnly[2]) - 1, Number(dateOnly[3]));
    }
  }
  return new Date(value);
}

export type { Fruit, Size, Money, ShortDate } from "../../shared";

const cart_label = "В корзине";

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

export const email_input = {
  label: "Email",
  placeholder: "name@example.com",
  aria: "Адрес электронной почты",
} as const;

const lgl = {
  email_input,
} as const;

export default lgl;
