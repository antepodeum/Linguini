import { formatCurrency, formatDate, formatNumber, selectBranch } from "../shared";

function pluralFr(value: number | string): string {
  const operands = pluralOperands(value);
  if (((operands.i === 0 || operands.i === 1))) return "one";
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
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralFr(value), { one: "pomme", _: "pommes" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralFr(value), { one: "poire", _: "poires" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralFr(value), { one: "orange", _: "oranges" }) },
} as const;

function SizeWord(p0: string | number): string {
  return selectBranch(String(p0), { small: "petit", big: "grand" });
}

function GenderWord(p0: string | number): string {
  return selectBranch(String(p0), { male: "masculin", female: "féminin", neuter: "neutre", _: "neutre" });
}

function Delivered(p0: string | number, p1: string | number): string {
  return selectBranch(pluralFr(p0), { one: "Livré", _: "Livrés" });
}

export const main = {
  nav_why: "Pourquoi",
  nav_language: "Langue",
  nav_codegen: "Codegen",
  nav_web: "Web",
  locale_label: "Langue",
  hero_eyebrow: "Localisation typée pour équipes produit",
  hero_title: "Linguini",
  hero_copy: "Un langage de localisation compilé où schémas, grammaire, formatage CLDR, hooks SvelteKit, cookies et routes localisées viennent d'une seule source.",
  primary_cta: "Lire la doc",
  secondary_cta: "Voir GitHub",
  schema_chip: "le schéma définit le contrat",
  locale_chip: "la locale définit la langue",
  generated_chip: "l'app importe du code généré",
  proof_kicker: "Pipeline",
  proof_title: "Linguini réel du schéma à SvelteKit",
  feature_schema: "Le texte, les enum, les TypeKind aliases et les formatter defaults vivent dans les fichiers .lgs.",
  feature_locale: "Les locales implémentent plural, gender, size, forms et functions sans runtime parsing.",
  feature_cldr: "Les exemples number, currency et date utilisent les CLDR formatters générés par Linguini.",
  feature_web: "Les hooks SvelteKit gardent la locale en cookie et routent en /en/... et /ru/....",
  sample_kicker: "Sortie générée",
  sample_title: "Formatage dans le schéma, sans bruit dans la locale",
  reference_cta: "Référence",
  playground_kicker: "Playground live",
  playground_title: "Change count, fruit, size, gender, amount, date ou locale.",
  count_label: "Quantité",
  fruit_label: "Fruit",
  size_label: "Taille",
  gender_label: "Genre",
  amount_label: "Montant",
  date_label: "Date",
  localized_path_label: "Route localisée",
  cookie_label: "Cookie",
  route_label: "Préfixe route",
  playground_sentence: (fruit: Fruit, size: Size, gender: Gender, count: number, amount: Money, date: ShortDate) => String(Delivered(count, gender)) + " " + String(count) + " " + String(SizeWord(size)) + " " + String(FruitForms[fruit].nom(count)) + " pour un élément " + String(GenderWord(gender)) + ". Total " + String(formatCurrency(amount, { locale: "fr", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤;(#,##0.00 ¤)" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1} {0}" } } }, { code: "EUR" })) + "; date " + String(formatDate(date, { locale: "fr", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤;(#,##0.00 ¤)" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1} {0}" } } }, { style: "short" })) + ".",
  cart_summary: (count: number, fruit: Fruit) => "Le panier contient " + String(count) + " " + String(FruitForms[fruit].nom(count)),
  number_format: (value: Measurement) => "Format nombre: " + String(formatNumber(value, { locale: "fr", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤;(#,##0.00 ¤)" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1} {0}" } } })),
  currency_format: (amount: Money) => "Format devise: " + String(formatCurrency(amount, { locale: "fr", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤;(#,##0.00 ¤)" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1} {0}" } } }, { code: "EUR" })),
  date_format: (date: ShortDate) => "Format date: " + String(formatDate(date, { locale: "fr", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤;(#,##0.00 ¤)" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1} {0}" } } }, { style: "short" })),
  override_format: (amount: Money, date: ShortDate) => "Locale override: " + String(formatNumber(amount, { locale: "fr", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤;(#,##0.00 ¤)" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1} {0}" } } })) + " / " + String(formatDate(date, { locale: "fr", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤;(#,##0.00 ¤)" }, date: { dateFormats: { full: "EEEE d MMMM y", long: "d MMMM y", medium: "d MMM y", short: "dd/MM/y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1} {0}" } } }, { style: "long" })),
  gender_line: (gender: Gender) => "Forme genre: " + String(GenderWord(gender)),
  size_line: (size: Size) => "Forme taille: " + String(SizeWord(size)),
} as const;

const lgl = {
  main,
} as const;

export default lgl;
