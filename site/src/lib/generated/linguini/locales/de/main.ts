import type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../../shared";
import { selectBranch } from "../../shared";

function pluralDe(value: number | bigint | string): string {
  if (value === "zero" || value === "one" || value === "two" || value === "few" || value === "many" || value === "other") return value;
  const operands = pluralOperands(value);
  if ((pluralOperandMatches(operands.i, undefined, false, [[1n, 1n]]) && pluralOperandMatches(operands.v, undefined, false, [[0n, 0n]]))) return "one";
  return "other";
}

type PluralOperand = { integer: bigint; hasFraction: boolean };
const MAX_PLURAL_DECIMAL_DIGITS = 8192;

function pluralOperands(value: number | bigint | string) {
  if (typeof value === "number" && !Number.isFinite(value)) throwInvalidPluralNumber();
  const source = String(value).trim();
  // CLDR c/e notation is a non-negative compact-decimal exponent. It is not
  // JavaScript's signed scientific notation, even though `e` is its legacy
  // spelling. Native numbers are the exception: JavaScript may stringify a
  // finite value with a signed e exponent, which describes only its value and
  // must not set the CLDR c/e operands.
  const match = typeof value === "number"
    ? /^([+-]?)(\d+)(?:\.(\d*))?(?:[eE]([+-]?\d+))?$/.exec(source)
    : /^([+-]?)(\d+)(?:\.(\d*))?(?:[cCeE](\d+))?$/.exec(source);
  if (!match) throwInvalidPluralNumber();

  const whole = match[2];
  const sourceFraction = match[3] ?? "";
  const exponent = Number(match[4] ?? "0");
  if (
    !Number.isSafeInteger(exponent) ||
    Math.abs(exponent) > MAX_PLURAL_DECIMAL_DIGITS ||
    (match[4]?.length ?? 0) > MAX_PLURAL_DECIMAL_DIGITS ||
    whole.length + sourceFraction.length > MAX_PLURAL_DECIMAL_DIGITS
  ) {
    throwInvalidPluralNumber();
  }

  const digits = `${whole}${sourceFraction}` || "0";
  const decimalPosition = whole.length + exponent;
  const expandedLength = decimalPosition <= 0
    ? -decimalPosition + digits.length
    : Math.max(decimalPosition, digits.length);
  if (expandedLength > MAX_PLURAL_DECIMAL_DIGITS) throwInvalidPluralNumber();

  let integer: string;
  let fraction: string;
  if (decimalPosition <= 0) {
    integer = "0";
    fraction = `${"0".repeat(-decimalPosition)}${digits}`;
  } else if (decimalPosition >= digits.length) {
    integer = `${digits}${"0".repeat(decimalPosition - digits.length)}`;
    fraction = "";
  } else {
    integer = digits.slice(0, decimalPosition);
    fraction = digits.slice(decimalPosition);
  }
  integer = integer.replace(/^0+(?=\d)/, "");
  const trimmedFraction = fraction.replace(/0+$/, "");
  const compactExponent = typeof value === "string" ? match[4] ?? "0" : "0";
  const operand = (digits: string, hasFraction = false): PluralOperand => ({
    integer: BigInt(digits || "0"),
    hasFraction,
  });

  return {
    n: operand(integer, /[1-9]/.test(fraction)),
    i: operand(integer),
    v: operand(String(fraction.length)),
    w: operand(String(trimmedFraction.length)),
    f: operand(fraction),
    t: operand(trimmedFraction),
    c: operand(compactExponent),
    e: operand(compactExponent),
  };
}

function pluralOperandMatches(
  value: PluralOperand,
  modulo: bigint | undefined,
  allowFraction: boolean,
  ranges: readonly (readonly [bigint, bigint])[],
): boolean {
  const integer = modulo === undefined || modulo === 0n
    ? value.integer
    : value.integer % modulo;
  return ranges.some(([start, end]) => {
    if (!allowFraction) {
      return !value.hasFraction && integer >= start && integer <= end;
    }
    return integer >= start &&
      (integer < end || (integer === end && !value.hasFraction));
  });
}

function throwInvalidPluralNumber(): never {
  throw new RangeError("Linguini: invalid plural numeric value");
}

type GeneratedNumeric = number | bigint | string;
type GeneratedCurrencyFormatterOptions = { code?: string; accounting?: "true" | "false" };
type GeneratedDateFormatterOptions = { style?: "full" | "long" | "medium" | "short" };

function formatNumber(value: GeneratedNumeric): string {
  return formatGeneratedNumber(value, "", "", undefined, undefined, 1, 0, 3, 3, undefined, ",", ".");
}

function formatCurrency(
  value: GeneratedNumeric,
  fractionDigits: number,
  roundingIncrement: number,
  options: GeneratedCurrencyFormatterOptions = {},
): string {
  const symbol = currencySymbol(options.code ?? "USD");
  if (options.accounting === "true") {
    return formatGeneratedNumber(value, "", " " + symbol + "", undefined, undefined, 1, 2, 2, 3, undefined, ",", ".", fractionDigits, fractionDigits, roundingIncrement);
  }
  return formatGeneratedNumber(value, "", " " + symbol + "", undefined, undefined, 1, 2, 2, 3, undefined, ",", ".", fractionDigits, fractionDigits, roundingIncrement);
}

function currencySymbol(currency: string): string {
  return new Intl.NumberFormat("de", { style: "currency", currency })
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
      return ["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"][date.getUTCDay()] + ", " + String(date.getUTCDate()) + ". " + ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"][date.getUTCMonth()] + " " + String(date.getUTCFullYear());
    case "long":
      return String(date.getUTCDate()) + ". " + ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"][date.getUTCMonth()] + " " + String(date.getUTCFullYear());
    case "short":
      return padNumber(date.getUTCDate(), 2) + "." + padNumber(date.getUTCMonth() + 1, 2) + "." + padNumber(date.getUTCFullYear() % 100, 2);
    default:
      return padNumber(date.getUTCDate(), 2) + "." + padNumber(date.getUTCMonth() + 1, 2) + "." + String(date.getUTCFullYear());
  }
}

type GeneratedDecimal = { negative: boolean; integer: string; fraction: string };
const MAX_GENERATED_DECIMAL_DIGITS = 8192;

function formatGeneratedNumber(
  value: GeneratedNumeric,
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
  minFractionDigitsOverride?: number,
  maxFractionDigitsOverride?: number,
  roundingIncrement = 0,
): string {
  const decimal = parseGeneratedDecimal(value);
  if (!decimal) return String(value);
  const effectiveMinFractionDigits = minFractionDigitsOverride ?? minFractionDigits;
  const effectiveMaxFractionDigits = maxFractionDigitsOverride ?? maxFractionDigits;
  const rounded = roundGeneratedDecimal(decimal, effectiveMaxFractionDigits, roundingIncrement);
  let integer = rounded.integer.padStart(minIntegerDigits, "0");
  const fraction = trimOptionalFractionDigits(
    rounded.fraction,
    effectiveMinFractionDigits,
  );

  integer = groupIntegerDigits(integer, primaryGroupSize, secondaryGroupSize, groupSymbol);
  const formatted = fraction ? `${integer}${decimalSymbol}${fraction}` : integer;
  if (decimal.negative) {
    return `${negativePrefix ?? `-${prefix}`}${formatted}${negativeSuffix ?? suffix}`;
  }
  return `${prefix}${formatted}${suffix}`;
}

function parseGeneratedDecimal(value: GeneratedNumeric): GeneratedDecimal | undefined {
  if (typeof value === "number" && !Number.isFinite(value)) return undefined;
  const negativeZero = typeof value === "number" && Object.is(value, -0);
  const source = String(value);
  const match = /^([+-]?)(\d*)(?:\.(\d*))?(?:[eE]([+-]?\d+))?$/.exec(source);
  if (!match || (match[2] === "" && (match[3] ?? "") === "")) throwInvalidNumber();

  const whole = match[2];
  const fractional = match[3] ?? "";
  const exponent = Number(match[4] ?? "0");
  if (
    !Number.isSafeInteger(exponent) ||
    Math.abs(exponent) > MAX_GENERATED_DECIMAL_DIGITS ||
    (match[4]?.length ?? 0) > MAX_GENERATED_DECIMAL_DIGITS ||
    whole.length + fractional.length > MAX_GENERATED_DECIMAL_DIGITS
  ) {
    throwInvalidNumber();
  }

  const digits = `${whole}${fractional}` || "0";
  const decimalPosition = whole.length + exponent;
  const expandedLength = decimalPosition <= 0
    ? -decimalPosition + digits.length
    : Math.max(decimalPosition, digits.length);
  if (expandedLength > MAX_GENERATED_DECIMAL_DIGITS) throwInvalidNumber();

  let integer: string;
  let fraction: string;
  if (decimalPosition <= 0) {
    integer = "0";
    fraction = `${"0".repeat(-decimalPosition)}${digits}`;
  } else if (decimalPosition >= digits.length) {
    integer = `${digits}${"0".repeat(decimalPosition - digits.length)}`;
    fraction = "";
  } else {
    integer = digits.slice(0, decimalPosition);
    fraction = digits.slice(decimalPosition);
  }
  integer = integer.replace(/^0+(?=\d)/, "");
  fraction = fraction.replace(/0+$/, "");
  return {
    negative: match[1] === "-" || negativeZero,
    integer,
    fraction,
  };
}

function roundGeneratedDecimal(
  decimal: GeneratedDecimal,
  fractionDigits: number,
  roundingIncrement: number,
): { integer: string; fraction: string } {
  if (
    !Number.isSafeInteger(fractionDigits) ||
    fractionDigits < 0 ||
    fractionDigits > MAX_GENERATED_DECIMAL_DIGITS ||
    !Number.isSafeInteger(roundingIncrement) ||
    roundingIncrement < 0
  ) {
    throwInvalidNumber();
  }

  const keptFraction = decimal.fraction.slice(0, fractionDigits).padEnd(fractionDigits, "0");
  const discarded = decimal.fraction.slice(fractionDigits);
  const scaled = BigInt(`${decimal.integer}${keptFraction}` || "0");
  const quantum = BigInt(roundingIncrement || 1);
  const remainder = scaled % quantum;
  let roundUp: boolean;
  if (discarded === "") {
    roundUp = remainder * 2n >= quantum;
  } else {
    const denominator = 10n ** BigInt(discarded.length);
    const exactRemainder = remainder * denominator + BigInt(discarded);
    roundUp = exactRemainder * 2n >= quantum * denominator;
  }
  const rounded = (scaled / quantum + (roundUp ? 1n : 0n)) * quantum;
  const digits = rounded.toString().padStart(fractionDigits + 1, "0");
  return fractionDigits === 0
    ? { integer: digits, fraction: "" }
    : {
        integer: digits.slice(0, -fractionDigits),
        fraction: digits.slice(-fractionDigits),
      };
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

function throwInvalidNumber(): never {
  throw new RangeError("Linguini: invalid numeric value");
}

function padNumber(value: number, length: number): string {
  return String(value).padStart(length, "0");
}

function coerceDate(value: Date | number | string): Date {
  let date: Date;
  if (value instanceof Date) {
    date = value;
  } else if (typeof value === "string") {
    const dateOnly = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
    if (dateOnly) {
      const year = Number(dateOnly[1]);
      const month = Number(dateOnly[2]);
      const day = Number(dateOnly[3]);
      date = createUTCDate(year, month, day);
    } else {
      const dateTime =
        /^(\d{4})-(\d{2})-(\d{2})T\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?(?:Z|[+-]\d{2}:\d{2})?$/.exec(value);
      if (!dateTime) throwInvalidDate();
      createUTCDate(Number(dateTime[1]), Number(dateTime[2]), Number(dateTime[3]));
      const hasTimeZone = /(?:Z|[+-]\d{2}:\d{2})$/.test(value);
      date = new Date(hasTimeZone ? value : `${value}Z`);
    }
  } else {
    date = new Date(value);
  }
  if (!Number.isFinite(date.getTime())) {
    throwInvalidDate();
  }
  return date;
}

function createUTCDate(year: number, month: number, day: number): Date {
  const date = new Date(0);
  date.setUTCHours(0, 0, 0, 0);
  date.setUTCFullYear(year, month - 1, day);
  if (
    date.getUTCFullYear() !== year ||
    date.getUTCMonth() !== month - 1 ||
    date.getUTCDate() !== day
  ) {
    throwInvalidDate();
  }
  return date;
}

function throwInvalidDate(): never {
  throw new RangeError("Linguini: invalid date value");
}

export type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../../shared";

export type Fruit = __lgl_name_6D61696E2E4672756974;

export type Size = __lgl_name_6D61696E2E53697A65;

export type Money = __lgl_name_6D61696E2E4D6F6E6579;

export type ShortDate = __lgl_name_6D61696E2E53686F727444617465;

export type Measurement = __lgl_name_6D61696E2E4D6561737572656D656E74;

type __lgl_name_6D61696E2E47656E646572 = "male" | "female" | "neuter" | "other";

const __lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C = "Größenform";

const __lgl_form_6D61696E2E4672756974 = {
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | bigint | string) => selectBranch(pluralDe(value), { one: "Apfel", _: "Äpfel" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | bigint | string) => selectBranch(pluralDe(value), { one: "Birne", _: "Birnen" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | bigint | string) => selectBranch(pluralDe(value), { one: "Orange", _: "Orangen" }) },
} as const;

function __lgl_name_6D61696E2E53697A65576F7264(__lgl_p0: __lgl_name_6D61696E2E53697A65): string {
  return selectBranch(String(__lgl_p0), { small: (): string => "klein", big: (): string => "groß" })();
}

function __lgl_name_6D61696E2E53697A6541646A(__lgl_p0: __lgl_name_6D61696E2E53697A65, __lgl_p1: number | bigint | string, __lgl_p2: __lgl_name_6D61696E2E47656E646572): string {
  return selectBranch(String(__lgl_p0), { small: (): string => selectBranch(pluralDe(__lgl_p1), { one: (): string => selectBranch(String(__lgl_p2), { male: (): string => "kleiner", female: (): string => "kleine", neuter: (): string => "kleines", _: (): string => "kleiner" })(), _: (): string => "kleine" })(), big: (): string => selectBranch(pluralDe(__lgl_p1), { one: (): string => selectBranch(String(__lgl_p2), { male: (): string => "großer", female: (): string => "große", neuter: (): string => "großes", _: (): string => "großer" })(), _: (): string => "große" })(), _: (): string => "normale" })();
}

export const main = {
  codegen: {
    kicker: "Codegenerierung",
    title: "Ein Schema, typisierte TypeScript-Ausgabe",
    intro: "Linguini analysiert .lgs-Verträge und .lgl-Locale-Logik einmal und erzeugt dann zielspezifische Module mit CLDR-Formatierern, Plural-Selektoren und Framework-Adaptern zur Build-Zeit.",
    ts_title: "TypeScript / ESM",
    ts_desc: "Typisierte Message-Funktionen, gemeinsamer Runtime, Deklarationsdateien und optionales Tree-Shaking pro Locale.",
    svelte_title: "Svelte 5 + SvelteKit",
    svelte_desc: "Generierter l-Rune, Locale-Wechsel, localizeHref-Helfer und handle / reroute / load-Exports.",
    planned_title: "Geplante Targets",
    planned_intro: "Die Roadmap ist bewusst breit: Linguini soll alle praktischen Runtimes, Sprachen und Web-Frameworks erreichen.",
    rust: "Rust",
    kotlin: "Kotlin",
    swift: "Swift",
    go: "Go",
    python: "Python",
    csharp: "C#",
    status_shipped: "Verfügbar",
    status_planned: "Geplant",
  },
  hero: {
    eyebrow: "Typisierte Lokalisierung für Produktteams",
    title: "Linguini",
    tagline: "Internationalisierung für uns alle",
    copy: "Eine kompilierte Lokalisierungssprache, in der Schemas, Grammatik, CLDR-Formatierung, SvelteKit-Hooks, Cookies und lokalisierte Routen aus einer Quelle entstehen.",
    term: "/lɪŋˈɡwiːni/",
    term_kind: "NOMEN",
    term_hint: "lange, schmale Pastabänder",
    intro: "Linguini macht aus schema-gebundenen Locale-Dateien typisierten generierten Code. Vor der Generierung melden check/build fehlende Implementierungen und prüfen unterstützte Formatter-Aufrufe sowie Locale-Aufrufe und Dispatch-Zweige, die der Analyzer auflösen kann.",
    trait_typed: "Typisiert",
    trait_compiled: "Kompiliert",
    trait_native: "Nativ",
    primary_cta: "Docs lesen",
    secondary_cta: "GitHub ansehen",
  },
  nav: {
    why: "Warum",
    language: "Sprache",
    codegen: "Codegen",
    web: "Web",
    locale_label: "Sprache",
  },
  playground: {
    kicker: "Live Playground",
    title: "Ändere count, fruit, size, amount, date oder locale.",
    count_label: "Anzahl",
    fruit_label: "Frucht",
    fruit_apple_label: "Apfel",
    fruit_pear_label: "Birne",
    fruit_orange_label: "Orange",
    size_label: "Größe",
    size_small_label: "klein",
    size_big_label: "groß",
    amount_label: "Betrag",
    date_label: "Datum",
    localized_path_label: "Lokalisierte Route",
    cookie_label: "Cookie",
    route_label: "Route-Präfix",
    sentence: (fruit: __lgl_name_6D61696E2E4672756974, size: __lgl_name_6D61696E2E53697A65, count: number | bigint | string, amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => "Geliefert " + String(formatNumber(count)) + " " + String(__lgl_name_6D61696E2E53697A6541646A(size, count, __lgl_form_6D61696E2E4672756974[fruit].Gender)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)) + ". Summe " + String(formatCurrency(amount, 2, 0, { code: "EUR" })) + "; Datum " + String(formatDate(date, { style: "short" })) + ".",
    cart_summary: (count: number | bigint | string, fruit: __lgl_name_6D61696E2E4672756974) => "Warenkorb enthält " + String(formatNumber(count)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)),
    number_format: (value: __lgl_name_6D61696E2E4D6561737572656D656E74) => "Zahlenformat: " + String(formatNumber(value)),
    currency_format: (amount: __lgl_name_6D61696E2E4D6F6E6579) => "Währungsformat: " + String(formatCurrency(amount, 2, 0, { code: "EUR" })),
    date_format: (date: __lgl_name_6D61696E2E53686F727444617465) => "Datumsformat: " + String(formatDate(date, { style: "short" })),
    override_format: (amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => "Locale override: " + String(formatNumber(amount)) + " / " + String(formatDate(date, { style: "long" })),
    size_line: (size: __lgl_name_6D61696E2E53697A65) => String(__lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C) + ": " + String(__lgl_name_6D61696E2E53697A65576F7264(size)),
  },
  web: {
    kicker: "SvelteKit",
    title: "Lokalisierter SvelteKit ohne handgeschriebene i18n-Verdrahtung",
    intro: "Write normal SvelteKit routes and generated message calls. Linguini keeps the active locale and updates messages reactively. Generated message signatures let TypeScript report invalid arguments when the app runs type checking.",
    routing: "Use ordinary internal links; the generated reroute hook handles localized URLs.",
    cookie: "Locale changes persist through cookies and browser storage without app-level state.",
    fallback: "Daten der Basis-Locale werden beim Build in generierte Locale-Module eingebettet.",
    reactivity: "The l rune updates UI text immediately after setLocale.",
    links: "Einzelne Links können ihre ursprüngliche URL behalten, wenn keine Lokalisierung gewünscht ist.",
  },
} as const;

const lgl = {
  main,
} as const;

export default lgl;
