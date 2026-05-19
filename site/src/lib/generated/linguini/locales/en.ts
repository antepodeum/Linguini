import { formatCurrency, formatDate, formatNumber, selectBranch } from "../shared";

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

export type Fruit = "apple" | "pear" | "orange";

export type Size = "small" | "big";

export type Gender = "male" | "female" | "neuter" | "other";

export type Money = number;

export type ShortDate = string;

export type Measurement = number;

const FruitForms = {
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralEn(value), { one: "apple", _: "apples" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralEn(value), { one: "pear", _: "pears" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralEn(value), { one: "orange", _: "oranges" }) },
} as const;

function SizeWord(p0: string | number): string {
  return selectBranch(String(p0), { small: "compact", big: "full-size" });
}

function GenderWord(p0: string | number): string {
  return selectBranch(String(p0), { male: "masculine", female: "feminine", neuter: "neuter", _: "neutral" });
}

function Delivered(p0: string | number, p1: string | number): string {
  return selectBranch(pluralEn(p0), { one: selectBranch(String(p1), { male: "Delivered", female: "Delivered", neuter: "Delivered", _: "Delivered" }), _: "Delivered" });
}

export const main = {
  nav_why: "Why",
  nav_language: "Language",
  nav_codegen: "Codegen",
  nav_web: "Web",
  locale_label: "Locale",
  hero_eyebrow: "Typed localization for product teams",
  hero_title: "Linguini",
  hero_copy: "A compiled localization language where schemas, locale grammar, CLDR formatting, SvelteKit hooks, cookies, and localized routes are generated from one source.",
  primary_cta: "Read the docs",
  secondary_cta: "View GitHub",
  schema_chip: "schema owns contract",
  locale_chip: "locale owns language",
  generated_chip: "app imports native generated code",
  proof_kicker: "Pipeline",
  proof_title: "Real Linguini from schema to SvelteKit",
  feature_schema: "The landing page text, enums, TypeKind aliases, and formatter defaults live in .lgs files.",
  feature_locale: "Locale files implement plural, gender, size, forms, and functions without runtime parsing.",
  feature_cldr: "Number, currency, and date examples use generated CLDR formatters from Linguini codegen.",
  feature_web: "SvelteKit hooks persist locale in a cookie and route pages as /en/... and /ru/....",
  sample_kicker: "Generated output",
  sample_title: "Schema formatting without locale boilerplate",
  reference_cta: "Reference",
  playground_kicker: "Live playground",
  playground_title: "Change count, fruit, size, gender, amount, date, or locale.",
  count_label: "Count",
  fruit_label: "Fruit",
  size_label: "Size",
  gender_label: "Gender",
  amount_label: "Amount",
  date_label: "Date",
  localized_path_label: "Localized route",
  cookie_label: "Cookie",
  route_label: "Route prefix",
  playground_sentence: (fruit: Fruit, size: Size, gender: Gender, count: number, amount: Money, date: ShortDate) => String(Delivered(count, gender)) + " " + String(count) + " " + String(SizeWord(size)) + " " + String(FruitForms[fruit].nom(count)) + " for a " + String(GenderWord(gender)) + " item. Total " + String(formatCurrency(amount, { locale: "en", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "EEEE, MMMM d, y", long: "MMMM d, y", medium: "MMM d, y", short: "M/d/yy" }, timeFormats: { full: "h:mm:ss a zzzz", long: "h:mm:ss a z", medium: "h:mm:ss a", short: "h:mm a" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { code: "USD" })) + "; ship date " + String(formatDate(date, { locale: "en", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "EEEE, MMMM d, y", long: "MMMM d, y", medium: "MMM d, y", short: "M/d/yy" }, timeFormats: { full: "h:mm:ss a zzzz", long: "h:mm:ss a z", medium: "h:mm:ss a", short: "h:mm a" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "short" })) + ".",
  cart_summary: (count: number, fruit: Fruit) => "Cart has " + String(count) + " " + String(FruitForms[fruit].nom(count)),
  number_format: (value: Measurement) => "Number formatting: " + String(formatNumber(value, { locale: "en", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "EEEE, MMMM d, y", long: "MMMM d, y", medium: "MMM d, y", short: "M/d/yy" }, timeFormats: { full: "h:mm:ss a zzzz", long: "h:mm:ss a z", medium: "h:mm:ss a", short: "h:mm a" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } })),
  currency_format: (amount: Money) => "Currency formatting: " + String(formatCurrency(amount, { locale: "en", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "EEEE, MMMM d, y", long: "MMMM d, y", medium: "MMM d, y", short: "M/d/yy" }, timeFormats: { full: "h:mm:ss a zzzz", long: "h:mm:ss a z", medium: "h:mm:ss a", short: "h:mm a" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { code: "USD" })),
  date_format: (date: ShortDate) => "Date formatting: " + String(formatDate(date, { locale: "en", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "EEEE, MMMM d, y", long: "MMMM d, y", medium: "MMM d, y", short: "M/d/yy" }, timeFormats: { full: "h:mm:ss a zzzz", long: "h:mm:ss a z", medium: "h:mm:ss a", short: "h:mm a" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "short" })),
  override_format: (amount: Money, date: ShortDate) => "Locale override: " + String(formatNumber(amount, { locale: "en", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "EEEE, MMMM d, y", long: "MMMM d, y", medium: "MMM d, y", short: "M/d/yy" }, timeFormats: { full: "h:mm:ss a zzzz", long: "h:mm:ss a z", medium: "h:mm:ss a", short: "h:mm a" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } })) + " / " + String(formatDate(date, { locale: "en", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "EEEE, MMMM d, y", long: "MMMM d, y", medium: "MMM d, y", short: "M/d/yy" }, timeFormats: { full: "h:mm:ss a zzzz", long: "h:mm:ss a z", medium: "h:mm:ss a", short: "h:mm a" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "long" })),
  gender_line: (gender: Gender) => "Gender form: " + String(GenderWord(gender)),
  size_line: (size: Size) => "Size form: " + String(SizeWord(size)),
} as const;

const lgl = {
  main,
} as const;

export default lgl;
