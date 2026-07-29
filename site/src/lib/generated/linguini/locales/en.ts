import { main } from "./en/main";
import type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../shared";
import { selectBranch } from "../shared";


function pluralEn(value: number | string): string {
  const operands = pluralOperands(value);
  if (((operands.i === 1) && (operands.v === 0))) return "one";
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
  return formatGeneratedNumber(Number(value), "", "", undefined, undefined, 1, 0, 3, 3, undefined, ".", ",");
}

function formatCurrency(
  value: number | string,
  options: GeneratedCurrencyFormatterOptions = {},
): string {
  const symbol = currencySymbol(options.code ?? "USD");
  if (options.accounting === "true") {
    return formatGeneratedNumber(Number(value), "" + symbol + "", "", "(" + symbol + "", ")", 1, 2, 2, 3, undefined, ".", ",");
  }
  return formatGeneratedNumber(Number(value), "" + symbol + "", "", undefined, undefined, 1, 2, 2, 3, undefined, ".", ",");
}

function currencySymbol(currency: string): string {
  return new Intl.NumberFormat("en", { style: "currency", currency })
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
      return ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"][date.getDay()] + ", " + ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"][date.getMonth()] + " " + String(date.getDate()) + ", " + String(date.getFullYear());
    case "long":
      return ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"][date.getMonth()] + " " + String(date.getDate()) + ", " + String(date.getFullYear());
    case "short":
      return String(date.getMonth() + 1) + "/" + String(date.getDate()) + "/" + padNumber(date.getFullYear() % 100, 2);
    default:
      return ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"][date.getMonth()] + " " + String(date.getDate()) + ", " + String(date.getFullYear());
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

export type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../shared";

export type Fruit = __lgl_name_6D61696E2E4672756974;

export type Size = __lgl_name_6D61696E2E53697A65;

export type Money = __lgl_name_6D61696E2E4D6F6E6579;

export type ShortDate = __lgl_name_6D61696E2E53686F727444617465;

export type Measurement = __lgl_name_6D61696E2E4D6561737572656D656E74;

export { main };

const __lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C = "Size form";

const __lgl_form_6D61696E2E4672756974 = {
  apple: { emoji: "🍎", nom: (value: number | string) => selectBranch(pluralEn(value), { one: "apple", _: "apples" }) },
  pear: { emoji: "🍐", nom: (value: number | string) => selectBranch(pluralEn(value), { one: "pear", _: "pears" }) },
  orange: { emoji: "🍊", nom: (value: number | string) => selectBranch(pluralEn(value), { one: "orange", _: "oranges" }) },
} as const;

function __lgl_name_6D61696E2E53697A65576F7264(p0: string | number): string {
  return selectBranch(String(p0), { small: "compact", big: "full-size" });
}

const lgl = {
  main,
} as const;

export default lgl;
