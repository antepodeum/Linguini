import { formatCurrency, formatDate, formatNumber, selectBranch } from "../shared";

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

export type Fruit = "apple" | "pear" | "orange";

export type Size = "small" | "big";

export type Gender = "male" | "female" | "neuter" | "other";

export type Money = number;

export type ShortDate = string;

export type Measurement = number;

const FruitForms = {
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralIt(value), { one: "mela", _: "mele" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralIt(value), { one: "pera", _: "pere" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralIt(value), { one: "arancia", _: "arance" }) },
} as const;

function SizeWord(p0: string | number): string {
  return selectBranch(String(p0), { small: "piccolo", big: "grande" });
}

function GenderWord(p0: string | number): string {
  return selectBranch(String(p0), { male: "maschile", female: "femminile", neuter: "neutro", _: "neutro" });
}

function Delivered(p0: string | number, p1: string | number): string {
  return selectBranch(pluralIt(p0), { one: "Consegnato", _: "Consegnati" });
}

export const main = {
  nav_why: "Perché",
  nav_language: "Lingua",
  nav_codegen: "Codegen",
  nav_web: "Web",
  locale_label: "Lingua",
  hero_eyebrow: "Localizzazione tipizzata per team di prodotto",
  hero_title: "Linguini",
  hero_copy: "Un linguaggio di localizzazione compilato dove schemi, grammatica, formattazione CLDR, hook SvelteKit, cookie e route localizzate vengono da una sola fonte.",
  primary_cta: "Leggi docs",
  secondary_cta: "Vedi GitHub",
  schema_chip: "lo schema definisce il contratto",
  locale_chip: "la locale definisce la lingua",
  generated_chip: "l'app importa codice generato",
  proof_kicker: "Pipeline",
  proof_title: "Linguini reale dallo schema a SvelteKit",
  feature_schema: "Testo, enum, alias TypeKind e default formatter vivono nei file .lgs.",
  feature_locale: "Le locale implementano plural, gender, size, forms e functions senza runtime parsing.",
  feature_cldr: "Esempi number, currency e date usano formatter CLDR generati da Linguini.",
  feature_web: "Gli hook SvelteKit salvano la locale nel cookie e usano route come /en/... e /ru/....",
  sample_kicker: "Output generato",
  sample_title: "Formattazione nello schema senza rumore nella locale",
  reference_cta: "Riferimento",
  playground_kicker: "Playground live",
  playground_title: "Cambia count, fruit, size, gender, amount, date o locale.",
  count_label: "Quantità",
  fruit_label: "Frutta",
  size_label: "Dimensione",
  gender_label: "Genere",
  amount_label: "Importo",
  date_label: "Data",
  localized_path_label: "Route localizzata",
  cookie_label: "Cookie",
  route_label: "Prefisso route",
  playground_sentence: (fruit: Fruit, size: Size, gender: Gender, count: number, amount: Money, date: ShortDate) => String(Delivered(count, gender)) + " " + String(count) + " " + String(SizeWord(size)) + " " + String(FruitForms[fruit].nom(count)) + " per un elemento " + String(GenderWord(gender)) + ". Totale " + String(formatCurrency(amount, { locale: "it", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { code: "EUR" })) + "; data " + String(formatDate(date, { locale: "it", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "short" })) + ".",
  cart_summary: (count: number, fruit: Fruit) => "Il carrello ha " + String(count) + " " + String(FruitForms[fruit].nom(count)),
  number_format: (value: Measurement) => "Formato numero: " + String(formatNumber(value, { locale: "it", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1}, {0}", short: "{1}, {0}" } } })),
  currency_format: (amount: Money) => "Formato valuta: " + String(formatCurrency(amount, { locale: "it", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { code: "EUR" })),
  date_format: (date: ShortDate) => "Formato data: " + String(formatDate(date, { locale: "it", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "short" })),
  override_format: (amount: Money, date: ShortDate) => "Locale override: " + String(formatNumber(amount, { locale: "it", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1}, {0}", short: "{1}, {0}" } } })) + " / " + String(formatDate(date, { locale: "it", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/yy" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "long" })),
  gender_line: (gender: Gender) => "Forma genere: " + String(GenderWord(gender)),
  size_line: (size: Size) => "Forma dimensione: " + String(SizeWord(size)),
} as const;

const lgl = {
  main,
} as const;

export default lgl;
