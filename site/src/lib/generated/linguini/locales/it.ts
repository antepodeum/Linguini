import { main } from "./it/main";
import type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../shared";
import { selectBranch } from "../shared";


function pluralIt(value: number | string): string {
  const operands = pluralOperands(value);
  if (((operands.i === 1) && (operands.v === 0))) return "one";
  if (((operands.e === 0) && !(operands.i === 0) && ((operands.i % 1000000) === 0) && (operands.v === 0)) || (!((operands.e >= 0 && operands.e <= 5)))) return "many";
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
  return formatGeneratedNumber(Number(value), "", "", undefined, undefined, 1, 0, 3, 3, undefined, ",", ".");
}

function formatCurrency(
  value: number | string,
  options: GeneratedCurrencyFormatterOptions = {},
): string {
  const symbol = currencySymbol(options.code ?? "USD");
  if (options.accounting === "true") {
    return formatGeneratedNumber(Number(value), "", " " + symbol + "", undefined, undefined, 1, 2, 2, 3, undefined, ",", ".");
  }
  return formatGeneratedNumber(Number(value), "", " " + symbol + "", undefined, undefined, 1, 2, 2, 3, undefined, ",", ".");
}

function currencySymbol(currency: string): string {
  return new Intl.NumberFormat("it", { style: "currency", currency })
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
      return ["domenica", "lunedì", "martedì", "mercoledì", "giovedì", "venerdì", "sabato"][date.getDay()] + " " + String(date.getDate()) + " " + ["gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno", "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre"][date.getMonth()] + " " + String(date.getFullYear());
    case "long":
      return String(date.getDate()) + " " + ["gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno", "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre"][date.getMonth()] + " " + String(date.getFullYear());
    case "short":
      return padNumber(date.getDate(), 2) + "/" + padNumber(date.getMonth() + 1, 2) + "/" + padNumber(date.getFullYear() % 100, 2);
    default:
      return String(date.getDate()) + " " + ["gen", "feb", "mar", "apr", "mag", "giu", "lug", "ago", "set", "ott", "nov", "dic"][date.getMonth()] + " " + String(date.getFullYear());
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

const __lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C = "Forma dimensione";

const __lgl_form_6D61696E2E4672756974 = {
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralIt(value), { one: "mela", _: "mele" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralIt(value), { one: "pera", _: "pere" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralIt(value), { one: "arancia", _: "arance" }) },
} as const;

function __lgl_name_6D61696E2E53697A65576F7264(p0: string | number): string {
  return selectBranch(String(p0), { small: "piccolo", big: "grande" });
}

function __lgl_name_6D61696E2E53697A6541646A(p0: string | number, p1: string | number, p2: string | number): string {
  return selectBranch(String(p0), { small: selectBranch(pluralIt(p1), { one: selectBranch(String(p2), { male: "piccolo", female: "piccola", neuter: "piccolo", _: "piccolo" }), _: selectBranch(String(p2), { male: "piccoli", female: "piccole", neuter: "piccoli", _: "piccoli" }) }), big: selectBranch(pluralIt(p1), { one: selectBranch(String(p2), { male: "grande", female: "grande", neuter: "grande", _: "grande" }), _: selectBranch(String(p2), { male: "grandi", female: "grandi", neuter: "grandi", _: "grandi" }) }), _: "normali" });
}

function __lgl_name_6D61696E2E44656C697665726564(p0: string | number, p1: string | number): string {
  return selectBranch(pluralIt(p0), { one: selectBranch(String(p1), { _: "Consegnato" }), _: "Consegnati" });
}

const lgl = {
  main,
} as const;

export default lgl;
