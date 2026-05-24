import { selectBranch } from "../shared";

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

type GeneratedNumberPatternPart = { prefix: string; suffix: string; minIntegerDigits: number; minFractionDigits: number; maxFractionDigits: number; primaryGroupSize?: number; secondaryGroupSize?: number };
type GeneratedNumberPattern = { positive: GeneratedNumberPatternPart; negative?: GeneratedNumberPatternPart };
type GeneratedCurrencyFormatterOptions = { code?: string; accounting?: "true" | "false" };
type GeneratedDateFormatterOptions = { style?: "full" | "long" | "medium" | "short" };

const FORMATTER_LOCALE = "ru";
const NUMBER_DECIMAL_SYMBOL = ",";
const NUMBER_GROUP_SYMBOL = " ";
const NUMBER_DECIMAL_PATTERN: GeneratedNumberPattern | undefined = { positive: { prefix: "", suffix: "", minIntegerDigits: 1, minFractionDigits: 0, maxFractionDigits: 3, primaryGroupSize: 3, secondaryGroupSize: undefined }, negative: undefined };
const CURRENCY_STANDARD_PATTERN: GeneratedNumberPattern | undefined = { positive: { prefix: "", suffix: " ¤", minIntegerDigits: 1, minFractionDigits: 2, maxFractionDigits: 2, primaryGroupSize: 3, secondaryGroupSize: undefined }, negative: undefined };
const CURRENCY_ACCOUNTING_PATTERN: GeneratedNumberPattern | undefined = { positive: { prefix: "", suffix: " ¤", minIntegerDigits: 1, minFractionDigits: 2, maxFractionDigits: 2, primaryGroupSize: 3, secondaryGroupSize: undefined }, negative: undefined };
const DATE_FORMATS = { full: "EEEE, d MMMM y 'г'.", long: "d MMMM y 'г'.", medium: "d MMM y 'г'.", short: "dd.MM.y" };

function formatNumber(value: number | string): string {
  return formatGeneratedNumber(Number(value), NUMBER_DECIMAL_PATTERN);
}

function formatCurrency(
  value: number | string,
  options: GeneratedCurrencyFormatterOptions = {},
): string {
  const currency = options.code ?? "USD";
  const pattern = options.accounting === "true"
    ? CURRENCY_ACCOUNTING_PATTERN ?? CURRENCY_STANDARD_PATTERN
    : CURRENCY_STANDARD_PATTERN;
  return formatGeneratedNumber(Number(value), pattern, currencySymbol(currency));
}

function formatDate(
  value: Date | number | string,
  options: GeneratedDateFormatterOptions = {},
): string {
  if (typeof value === "string") return value;
  const style = options.style ?? "medium";
  if (!DATE_FORMATS?.[style]) return new Intl.DateTimeFormat(FORMATTER_LOCALE).format(value);
  return new Intl.DateTimeFormat(FORMATTER_LOCALE, { dateStyle: style }).format(value);
}

function formatGeneratedNumber(
  value: number,
  pattern: GeneratedNumberPattern | undefined,
  currency?: string,
): string {
  if (!Number.isFinite(value)) return String(value);
  const negative = value < 0 || Object.is(value, -0);
  const positive = pattern?.positive;
  const part = negative ? pattern?.negative ?? negativePatternPart(positive) : positive;
  if (!part) return String(value);

  const rounded = roundToFractionDigits(Math.abs(value), part.maxFractionDigits);
  let [integer, fraction = ""] = rounded.toFixed(part.maxFractionDigits).split(".");
  integer = integer.padStart(part.minIntegerDigits, "0");
  fraction = trimOptionalFractionDigits(fraction, part.minFractionDigits);

  const grouped = groupIntegerDigits(integer, part.primaryGroupSize, part.secondaryGroupSize);
  const formatted = fraction ? `${grouped}${NUMBER_DECIMAL_SYMBOL ?? "."}${fraction}` : grouped;
  return `${formatNumberAffix(part.prefix, currency)}${formatted}${formatNumberAffix(part.suffix, currency)}`;
}

function negativePatternPart(part: GeneratedNumberPatternPart | undefined): GeneratedNumberPatternPart | undefined {
  return part ? { ...part, prefix: `-${part.prefix}` } : undefined;
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
  return groups.join(NUMBER_GROUP_SYMBOL ?? ",");
}

function formatNumberAffix(affix: string, currency: string | undefined): string {
  let output = "";
  for (const character of affix) {
    output += character === "¤" ? currency ?? "" : character;
  }
  return output;
}

function currencySymbol(currency: string): string {
  return new Intl.NumberFormat("ru", { style: "currency", currency })
    .formatToParts(0)
    .find((part) => part.type === "currency")?.value ?? currency;
}

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
  return "В корзине " + String(formatNumber(count)) + " " + String(FruitForms[fruit].nom(count));
}

export function price(amount: Money, date: ShortDate): string {
  return "Цена " + String(formatCurrency(amount, { code: "RUB" })) + " на " + String(formatDate(date, { style: "short" }));
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
