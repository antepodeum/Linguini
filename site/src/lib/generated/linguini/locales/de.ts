import { formatCurrency, formatDate, formatNumber, selectBranch } from "../shared";

function pluralDe(value: number | string): string {
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
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralDe(value), { one: "Apfel", _: "Äpfel" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralDe(value), { one: "Birne", _: "Birnen" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralDe(value), { one: "Orange", _: "Orangen" }) },
} as const;

function SizeWord(p0: string | number): string {
  return selectBranch(String(p0), { small: "klein", big: "groß" });
}

function GenderWord(p0: string | number): string {
  return selectBranch(String(p0), { male: "maskulin", female: "feminin", neuter: "neutrum", _: "neutral" });
}

function Delivered(p0: string | number, p1: string | number): string {
  return selectBranch(pluralDe(p0), { one: "Geliefert", _: "Geliefert" });
}

export const main = {
  nav_why: "Warum",
  nav_language: "Sprache",
  nav_codegen: "Codegen",
  nav_web: "Web",
  locale_label: "Sprache",
  hero_eyebrow: "Typisierte Lokalisierung für Produktteams",
  hero_title: "Linguini",
  hero_copy: "Eine kompilierte Lokalisierungssprache, in der Schemas, Grammatik, CLDR-Formatierung, SvelteKit-Hooks, Cookies und lokalisierte Routen aus einer Quelle entstehen.",
  primary_cta: "Docs lesen",
  secondary_cta: "GitHub ansehen",
  schema_chip: "Schema definiert Vertrag",
  locale_chip: "Locale definiert Sprache",
  generated_chip: "App importiert generierten Code",
  proof_kicker: "Pipeline",
  proof_title: "Echtes Linguini vom Schema bis SvelteKit",
  feature_schema: "Seitentexte, enums, TypeKind aliases und formatter defaults liegen in .lgs Dateien.",
  feature_locale: "Locales implementieren plural, gender, size, forms und functions ohne runtime parsing.",
  feature_cldr: "Number, currency und date Beispiele nutzen von Linguini generierte CLDR formatters.",
  feature_web: "SvelteKit hooks speichern Locale im Cookie und routen als /en/... und /ru/....",
  sample_kicker: "Generierte Ausgabe",
  sample_title: "Formatierung im Schema ohne Locale-Rauschen",
  reference_cta: "Referenz",
  playground_kicker: "Live Playground",
  playground_title: "Ändere count, fruit, size, gender, amount, date oder locale.",
  count_label: "Anzahl",
  fruit_label: "Frucht",
  size_label: "Größe",
  gender_label: "Genus",
  amount_label: "Betrag",
  date_label: "Datum",
  localized_path_label: "Lokalisierte Route",
  cookie_label: "Cookie",
  route_label: "Route-Präfix",
  playground_sentence: (fruit: Fruit, size: Size, gender: Gender, count: number, amount: Money, date: ShortDate) => String(Delivered(count, gender)) + " " + String(count) + " " + String(SizeWord(size)) + " " + String(FruitForms[fruit].nom(count)) + " für ein " + String(GenderWord(gender)) + " Element. Summe " + String(formatCurrency(amount, { locale: "de", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d. MMMM y", long: "d. MMMM y", medium: "dd.MM.y", short: "dd.MM.yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { code: "EUR" })) + "; Datum " + String(formatDate(date, { locale: "de", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d. MMMM y", long: "d. MMMM y", medium: "dd.MM.y", short: "dd.MM.yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "short" })) + ".",
  cart_summary: (count: number, fruit: Fruit) => "Warenkorb enthält " + String(count) + " " + String(FruitForms[fruit].nom(count)),
  number_format: (value: Measurement) => "Zahlenformat: " + String(formatNumber(value, { locale: "de", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d. MMMM y", long: "d. MMMM y", medium: "dd.MM.y", short: "dd.MM.yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } })),
  currency_format: (amount: Money) => "Währungsformat: " + String(formatCurrency(amount, { locale: "de", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d. MMMM y", long: "d. MMMM y", medium: "dd.MM.y", short: "dd.MM.yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { code: "EUR" })),
  date_format: (date: ShortDate) => "Datumsformat: " + String(formatDate(date, { locale: "de", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d. MMMM y", long: "d. MMMM y", medium: "dd.MM.y", short: "dd.MM.yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "short" })),
  override_format: (amount: Money, date: ShortDate) => "Locale override: " + String(formatNumber(amount, { locale: "de", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d. MMMM y", long: "d. MMMM y", medium: "dd.MM.y", short: "dd.MM.yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } })) + " / " + String(formatDate(date, { locale: "de", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d. MMMM y", long: "d. MMMM y", medium: "dd.MM.y", short: "dd.MM.yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "long" })),
  gender_line: (gender: Gender) => "Genus-Form: " + String(GenderWord(gender)),
  size_line: (size: Size) => "Größenform: " + String(SizeWord(size)),
} as const;

const lgl = {
  main,
} as const;

export default lgl;
