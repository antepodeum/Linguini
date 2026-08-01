import type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../../shared";
import { selectBranch } from "../../shared";

function pluralEn(value: number | bigint | string): string {
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
  return formatGeneratedNumber(value, "", "", undefined, undefined, 1, 0, 3, 3, undefined, ".", ",");
}

function formatCurrency(
  value: GeneratedNumeric,
  fractionDigits: number,
  roundingIncrement: number,
  options: GeneratedCurrencyFormatterOptions = {},
): string {
  const symbol = currencySymbol(options.code ?? "USD");
  if (options.accounting === "true") {
    return formatGeneratedNumber(value, "" + symbol + "", "", "(" + symbol + "", ")", 1, 2, 2, 3, undefined, ".", ",", fractionDigits, fractionDigits, roundingIncrement);
  }
  return formatGeneratedNumber(value, "" + symbol + "", "", undefined, undefined, 1, 2, 2, 3, undefined, ".", ",", fractionDigits, fractionDigits, roundingIncrement);
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
      return ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"][date.getUTCDay()] + ", " + ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"][date.getUTCMonth()] + " " + String(date.getUTCDate()) + ", " + String(date.getUTCFullYear());
    case "long":
      return ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"][date.getUTCMonth()] + " " + String(date.getUTCDate()) + ", " + String(date.getUTCFullYear());
    case "short":
      return String(date.getUTCMonth() + 1) + "/" + String(date.getUTCDate()) + "/" + padNumber(date.getUTCFullYear() % 100, 2);
    default:
      return ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"][date.getUTCMonth()] + " " + String(date.getUTCDate()) + ", " + String(date.getUTCFullYear());
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

const __lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C = "Size form";

const __lgl_form_6D61696E2E4672756974 = {
  apple: { emoji: "🍎", nom: (value: number | bigint | string) => selectBranch(pluralEn(value), { one: "apple", _: "apples" }) },
  pear: { emoji: "🍐", nom: (value: number | bigint | string) => selectBranch(pluralEn(value), { one: "pear", _: "pears" }) },
  orange: { emoji: "🍊", nom: (value: number | bigint | string) => selectBranch(pluralEn(value), { one: "orange", _: "oranges" }) },
} as const;

function __lgl_name_6D61696E2E53697A65576F7264(__lgl_p0: __lgl_name_6D61696E2E53697A65): string {
  return selectBranch(String(__lgl_p0), { small: (): string => "compact", big: (): string => "full-size" })();
}

export const main = {
  codegen: {
    kicker: "Code generation",
    title: "One schema, typed TypeScript output",
    intro: "Linguini analyzes .lgs contracts and .lgl locale logic once, then emits target-specific modules with CLDR formatters, plural selectors, and framework adapters baked in at build time.",
    ts_title: "TypeScript / ESM",
    ts_desc: "Typed message functions, shared runtime, declaration files, and optional per-locale tree shaking.",
    svelte_title: "Svelte 5 + SvelteKit",
    svelte_desc: "Generated l rune, locale switching, localized href helpers, and handle / reroute / load exports.",
    planned_title: "Planned targets",
    planned_intro: "The roadmap is intentionally broad: Linguini is planned to reach every practical runtime, language, and web framework.",
    rust: "Rust",
    kotlin: "Kotlin",
    swift: "Swift",
    go: "Go",
    python: "Python",
    csharp: "C#",
    status_shipped: "Stable",
    status_planned: "Planned",
  },
  hero: {
    eyebrow: "Typed localization for product teams",
    title: "Linguini",
    tagline: "Internationalization for the rest of us",
    copy: "A compiled localization language where schemas, locale grammar, CLDR formatting, SvelteKit hooks, cookies, and localized routes are generated from one source.",
    term: "/lɪŋˈɡwiːni/",
    term_kind: "NOUN",
    term_hint: "long, narrow ribbons of pasta",
    intro: "Linguini turns schema-backed locale files into typed generated code. Before generation, check/build report missing implementations and validate supported formatter uses plus locale calls and dispatch branches the analyzer can resolve.",
    trait_typed: "Typed",
    trait_compiled: "Compiled",
    trait_native: "Native",
    primary_cta: "Read the docs",
    secondary_cta: "View GitHub",
  },
  nav: {
    why: "Why",
    language: "Language",
    codegen: "Codegen",
    web: "Web",
    locale_label: "Locale",
  },
  playground: {
    kicker: "Live playground",
    title: "Change count, fruit, size, amount, date, or locale.",
    count_label: "Count",
    fruit_label: "Fruit",
    fruit_apple_label: "apple",
    fruit_pear_label: "pear",
    fruit_orange_label: "orange",
    size_label: "Size",
    size_small_label: "compact",
    size_big_label: "full-size",
    amount_label: "Amount",
    date_label: "Date",
    localized_path_label: "Localized route",
    cookie_label: "Cookie",
    route_label: "Route prefix",
    sentence: (fruit: __lgl_name_6D61696E2E4672756974, size: __lgl_name_6D61696E2E53697A65, count: number | bigint | string, amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => "Delivered " + String(formatNumber(count)) + " " + String(__lgl_name_6D61696E2E53697A65576F7264(size)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)) + ". Total " + String(formatCurrency(amount, 2, 0, { code: "USD" })) + "; ship date " + String(formatDate(date, { style: "short" })) + ".",
    cart_summary: (count: number | bigint | string, fruit: __lgl_name_6D61696E2E4672756974) => String(((__lgl_inline_selector_0, __lgl_inline_selector_1) => selectBranch(String(__lgl_inline_selector_0), { apple: (): string => selectBranch(String(__lgl_inline_selector_1), { one: (): string => "Cart has " + String(formatNumber(count)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)), other: (): string => "Cart has " + String(formatNumber(count)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)) })(), _: (): string => selectBranch(String(__lgl_inline_selector_1), { one: (): string => "Cart has " + String(formatNumber(count)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)), other: (): string => "Cart has " + String(formatNumber(count)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)) })() })())(fruit, pluralEn(count))),
    number_format: (value: __lgl_name_6D61696E2E4D6561737572656D656E74) => "Number formatting: " + String(formatNumber(value)),
    currency_format: (amount: __lgl_name_6D61696E2E4D6F6E6579) => "Currency formatting: " + String(formatCurrency(amount, 2, 0, { code: "USD" })),
    date_format: (date: __lgl_name_6D61696E2E53686F727444617465) => "Date formatting: " + String(formatDate(date, { style: "short" })),
    override_format: (amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => "Locale override: " + String(formatNumber(amount)) + " / " + String(formatDate(date, { style: "long" })),
    size_line: (size: __lgl_name_6D61696E2E53697A65) => String(((__lgl_inline_selector_0, label) => selectBranch(String(__lgl_inline_selector_0), { small: (): string => String(__lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C) + ": " + String(label), _: (): string => String(__lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C) + ": " + String(label) })())(size, __lgl_name_6D61696E2E53697A65576F7264(size))),
  },
  web: {
    kicker: "SvelteKit",
    title: "Ship localized SvelteKit without handwritten i18n wiring",
    intro: "Write normal SvelteKit routes and generated message calls. Linguini keeps the active locale and updates messages reactively. Generated message signatures let TypeScript report invalid arguments when the app runs type checking.",
    routing: "Use ordinary internal links; the generated reroute hook handles localized URLs.",
    cookie: "Locale changes persist through cookies and browser storage without app-level state.",
    fallback: "Base-locale data is baked into generated locale modules at build time.",
    reactivity: "The l rune updates UI text immediately after setLocale.",
    links: "Individual links can keep their original URL when localization is not wanted.",
  },
} as const;

const lgl = {
  main,
} as const;

export default lgl;
