import type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../../shared";
import { selectBranch } from "../../shared";

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

export type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../../shared";

export type Fruit = __lgl_name_6D61696E2E4672756974;

export type Size = __lgl_name_6D61696E2E53697A65;

export type Money = __lgl_name_6D61696E2E4D6F6E6579;

export type ShortDate = __lgl_name_6D61696E2E53686F727444617465;

export type Measurement = __lgl_name_6D61696E2E4D6561737572656D656E74;

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

export const main = {
  codegen: {
    kicker: "Generazione codice",
    title: "Uno schema, output tipizzato per ogni runtime",
    intro: "Linguini analizza contratti .lgs e logica .lgl una volta, poi emette moduli specifici con formattatori CLDR, selettori plurali e adapter framework a build time.",
    ts_title: "TypeScript / ESM",
    ts_desc: "Funzioni messaggio tipizzate, runtime condiviso, declaration file e tree shaking opzionale per locale.",
    svelte_title: "Svelte 5 + SvelteKit",
    svelte_desc: "Rune l generato, cambio locale, helper localizeHref ed export handle / reroute / load.",
    planned_title: "Target pianificati",
    planned_intro: "La roadmap è volutamente ampia: Linguini punta a supportare ogni runtime, linguaggio e framework web pratico.",
    rust: "Rust",
    kotlin: "Kotlin",
    swift: "Swift",
    go: "Go",
    python: "Python",
    csharp: "C#",
    status_shipped: "Disponibile",
    status_planned: "Pianificato",
  },
  hero: {
    eyebrow: "Localizzazione tipizzata per team di prodotto",
    title: "Linguini",
    tagline: "Internazionalizzazione per tutti noi",
    copy: "Un linguaggio di localizzazione compilato dove schemi, grammatica, formattazione CLDR, hook SvelteKit, cookie e route localizzate vengono da una sola fonte.",
    term: "/lɪŋˈɡwiːni/",
    term_kind: "NOME",
    term_hint: "pasta lunga e stretta a nastri",
    intro: "Linguini trasforma file di localizzazione tipizzati in codice usabile da qualsiasi app. Messaggi, grammatica, formattatori, fallback delle locale e routing web vengono controllati in build.",
    trait_typed: "Tipizzato",
    trait_compiled: "Compilato",
    trait_native: "Nativo",
    primary_cta: "Leggi docs",
    secondary_cta: "Vedi GitHub",
  },
  nav: {
    why: "Perché",
    language: "Lingua",
    codegen: "Codegen",
    web: "Web",
    locale_label: "Lingua",
  },
  playground: {
    kicker: "Playground live",
    title: "Cambia count, fruit, size, amount, date o locale.",
    count_label: "Quantità",
    fruit_label: "Frutta",
    fruit_apple_label: "mela",
    fruit_pear_label: "pera",
    fruit_orange_label: "arancia",
    size_label: "Dimensione",
    size_small_label: "piccolo",
    size_big_label: "grande",
    amount_label: "Importo",
    date_label: "Data",
    localized_path_label: "Route localizzata",
    cookie_label: "Cookie",
    route_label: "Prefisso route",
    sentence: (fruit: __lgl_name_6D61696E2E4672756974, size: __lgl_name_6D61696E2E53697A65, count: number, amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => String(__lgl_name_6D61696E2E44656C697665726564(count, __lgl_form_6D61696E2E4672756974[fruit].Gender)) + " " + String(formatNumber(count)) + " " + String(__lgl_name_6D61696E2E53697A6541646A(size, count, __lgl_form_6D61696E2E4672756974[fruit].Gender)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)) + ". Totale " + String(formatCurrency(amount, { code: "EUR" })) + "; data " + String(formatDate(date, { style: "short" })) + ".",
    cart_summary: (count: number, fruit: __lgl_name_6D61696E2E4672756974) => "Il carrello ha " + String(formatNumber(count)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)),
    number_format: (value: __lgl_name_6D61696E2E4D6561737572656D656E74) => "Formato numero: " + String(formatNumber(value)),
    currency_format: (amount: __lgl_name_6D61696E2E4D6F6E6579) => "Formato valuta: " + String(formatCurrency(amount, { code: "EUR" })),
    date_format: (date: __lgl_name_6D61696E2E53686F727444617465) => "Formato data: " + String(formatDate(date, { style: "short" })),
    override_format: (amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => "Locale override: " + String(formatNumber(amount)) + " / " + String(formatDate(date, { style: "long" })),
    size_line: (size: __lgl_name_6D61696E2E53697A65) => String(__lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C) + ": " + String(__lgl_name_6D61696E2E53697A65576F7264(size)),
  },
  web: {
    kicker: "SvelteKit",
    title: "SvelteKit localizzato senza cablaggio i18n manuale",
    intro: "Write normal SvelteKit routes and generated message calls. Linguini keeps the active locale, updates messages reactively, and turns invalid message arguments into TypeScript errors.",
    routing: "Use ordinary internal links; the generated reroute hook handles localized URLs.",
    cookie: "Locale changes persist through cookies and browser storage without app-level state.",
    fallback: "I dati della locale base vengono integrati nei moduli generati durante la build.",
    reactivity: "The l rune updates UI text immediately after setLocale.",
    links: "Alcuni link possono mantenere l'URL originale quando non vanno localizzati.",
  },
} as const;

const lgl = {
  main,
} as const;

export default lgl;
