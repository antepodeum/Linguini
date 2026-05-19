import { formatCurrency, formatDate, formatNumber, selectBranch } from "../shared";

function pluralEs(value: number | string): string {
  const operands = pluralOperands(value);
  if (((operands.n === 1))) return "one";
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
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralEs(value), { one: "manzana", _: "manzanas" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralEs(value), { one: "pera", _: "peras" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralEs(value), { one: "naranja", _: "naranjas" }) },
} as const;

function SizeWord(p0: string | number): string {
  return selectBranch(String(p0), { small: "pequeño", big: "grande" });
}

function GenderWord(p0: string | number): string {
  return selectBranch(String(p0), { male: "masculino", female: "femenino", neuter: "neutro", _: "neutral" });
}

function Delivered(p0: string | number, p1: string | number): string {
  return selectBranch(pluralEs(p0), { one: "Entregado", _: "Entregados" });
}

export const main = {
  nav_why: "Por qué",
  nav_language: "Idioma",
  nav_codegen: "Codegen",
  nav_web: "Web",
  locale_label: "Idioma",
  hero_eyebrow: "Localización tipada para equipos de producto",
  hero_title: "Linguini",
  hero_copy: "Un lenguaje de localización compilado donde esquemas, gramática, formato CLDR, hooks de SvelteKit, cookies y rutas localizadas se generan desde una sola fuente.",
  primary_cta: "Leer docs",
  secondary_cta: "Ver GitHub",
  schema_chip: "el esquema define el contrato",
  locale_chip: "la locale define el idioma",
  generated_chip: "la app importa código generado",
  proof_kicker: "Pipeline",
  proof_title: "Linguini real desde esquema hasta SvelteKit",
  feature_schema: "El texto, enum, TypeKind aliases y formatter defaults viven en archivos .lgs.",
  feature_locale: "Las locales implementan plural, género, tamaño, forms y functions sin runtime parsing.",
  feature_cldr: "Los ejemplos de number, currency y date usan CLDR formatters generados por Linguini.",
  feature_web: "Los hooks de SvelteKit guardan la locale en cookie y enrutan como /en/... y /ru/....",
  sample_kicker: "Salida generada",
  sample_title: "Formato en esquema sin ruido en locale",
  reference_cta: "Referencia",
  playground_kicker: "Playground en vivo",
  playground_title: "Cambia count, fruit, size, gender, amount, date o locale.",
  count_label: "Cantidad",
  fruit_label: "Fruta",
  size_label: "Tamaño",
  gender_label: "Género",
  amount_label: "Importe",
  date_label: "Fecha",
  localized_path_label: "Ruta localizada",
  cookie_label: "Cookie",
  route_label: "Prefijo de ruta",
  playground_sentence: (fruit: Fruit, size: Size, gender: Gender, count: number, amount: Money, date: ShortDate) => String(Delivered(count, gender)) + " " + String(count) + " " + String(SizeWord(size)) + " " + String(FruitForms[fruit].nom(count)) + " para un elemento " + String(GenderWord(gender)) + ". Total " + String(formatCurrency(amount, { locale: "es", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d 'de' MMMM 'de' y", long: "d 'de' MMMM 'de' y", medium: "d MMM y", short: "d/M/yy" }, timeFormats: { full: "H:mm:ss (zzzz)", long: "H:mm:ss z", medium: "H:mm:ss", short: "H:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { code: "EUR" })) + "; fecha " + String(formatDate(date, { locale: "es", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d 'de' MMMM 'de' y", long: "d 'de' MMMM 'de' y", medium: "d MMM y", short: "d/M/yy" }, timeFormats: { full: "H:mm:ss (zzzz)", long: "H:mm:ss z", medium: "H:mm:ss", short: "H:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "short" })) + ".",
  cart_summary: (count: number, fruit: Fruit) => "El carrito tiene " + String(count) + " " + String(FruitForms[fruit].nom(count)),
  number_format: (value: Measurement) => "Formato numérico: " + String(formatNumber(value, { locale: "es", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d 'de' MMMM 'de' y", long: "d 'de' MMMM 'de' y", medium: "d MMM y", short: "d/M/yy" }, timeFormats: { full: "H:mm:ss (zzzz)", long: "H:mm:ss z", medium: "H:mm:ss", short: "H:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } })),
  currency_format: (amount: Money) => "Formato moneda: " + String(formatCurrency(amount, { locale: "es", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d 'de' MMMM 'de' y", long: "d 'de' MMMM 'de' y", medium: "d MMM y", short: "d/M/yy" }, timeFormats: { full: "H:mm:ss (zzzz)", long: "H:mm:ss z", medium: "H:mm:ss", short: "H:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { code: "EUR" })),
  date_format: (date: ShortDate) => "Formato fecha: " + String(formatDate(date, { locale: "es", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d 'de' MMMM 'de' y", long: "d 'de' MMMM 'de' y", medium: "d MMM y", short: "d/M/yy" }, timeFormats: { full: "H:mm:ss (zzzz)", long: "H:mm:ss z", medium: "H:mm:ss", short: "H:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "short" })),
  override_format: (amount: Money, date: ShortDate) => "Locale override: " + String(formatNumber(amount, { locale: "es", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d 'de' MMMM 'de' y", long: "d 'de' MMMM 'de' y", medium: "d MMM y", short: "d/M/yy" }, timeFormats: { full: "H:mm:ss (zzzz)", long: "H:mm:ss z", medium: "H:mm:ss", short: "H:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } })) + " / " + String(formatDate(date, { locale: "es", numbers: { decimalSymbol: ",", groupSymbol: ".", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d 'de' MMMM 'de' y", long: "d 'de' MMMM 'de' y", medium: "d MMM y", short: "d/M/yy" }, timeFormats: { full: "H:mm:ss (zzzz)", long: "H:mm:ss z", medium: "H:mm:ss", short: "H:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "long" })),
  gender_line: (gender: Gender) => "Forma de género: " + String(GenderWord(gender)),
  size_line: (size: Size) => "Forma de tamaño: " + String(SizeWord(size)),
} as const;

const lgl = {
  main,
} as const;

export default lgl;
