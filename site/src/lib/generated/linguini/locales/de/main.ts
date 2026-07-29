import type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../../shared";
import { selectBranch } from "../../shared";

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
      return ["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"][date.getDay()] + ", " + String(date.getDate()) + ". " + ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"][date.getMonth()] + " " + String(date.getFullYear());
    case "long":
      return String(date.getDate()) + ". " + ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"][date.getMonth()] + " " + String(date.getFullYear());
    case "short":
      return padNumber(date.getDate(), 2) + "." + padNumber(date.getMonth() + 1, 2) + "." + padNumber(date.getFullYear() % 100, 2);
    default:
      return padNumber(date.getDate(), 2) + "." + padNumber(date.getMonth() + 1, 2) + "." + String(date.getFullYear());
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

const __lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C = "Größenform";

const __lgl_form_6D61696E2E4672756974 = {
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralDe(value), { one: "Apfel", _: "Äpfel" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralDe(value), { one: "Birne", _: "Birnen" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralDe(value), { one: "Orange", _: "Orangen" }) },
} as const;

function __lgl_name_6D61696E2E53697A65576F7264(p0: string | number): string {
  return selectBranch(String(p0), { small: "klein", big: "groß" });
}

function __lgl_name_6D61696E2E53697A6541646A(p0: string | number, p1: string | number, p2: string | number): string {
  return selectBranch(String(p0), { small: selectBranch(pluralDe(p1), { one: selectBranch(String(p2), { male: "kleiner", female: "kleine", neuter: "kleines", _: "kleiner" }), _: "kleine" }), big: selectBranch(pluralDe(p1), { one: selectBranch(String(p2), { male: "großer", female: "große", neuter: "großes", _: "großer" }), _: "große" }), _: "normale" });
}

export const main = {
  codegen: {
    kicker: "Codegenerierung",
    title: "Ein Schema, typisierte Ausgabe für jeden Runtime",
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
    intro: "Linguini macht aus typisierten Lokalisierungsdateien Code, den jede App nutzen kann. Messages, Grammatik, Formatter, Locale-Fallbacks und Web-Routing werden beim Build geprüft.",
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
    sentence: (fruit: __lgl_name_6D61696E2E4672756974, size: __lgl_name_6D61696E2E53697A65, count: number, amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => "Geliefert " + String(formatNumber(count)) + " " + String(__lgl_name_6D61696E2E53697A6541646A(size, count, __lgl_form_6D61696E2E4672756974[fruit].Gender)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)) + ". Summe " + String(formatCurrency(amount, { code: "EUR" })) + "; Datum " + String(formatDate(date, { style: "short" })) + ".",
    cart_summary: (count: number, fruit: __lgl_name_6D61696E2E4672756974) => "Warenkorb enthält " + String(formatNumber(count)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)),
    number_format: (value: __lgl_name_6D61696E2E4D6561737572656D656E74) => "Zahlenformat: " + String(formatNumber(value)),
    currency_format: (amount: __lgl_name_6D61696E2E4D6F6E6579) => "Währungsformat: " + String(formatCurrency(amount, { code: "EUR" })),
    date_format: (date: __lgl_name_6D61696E2E53686F727444617465) => "Datumsformat: " + String(formatDate(date, { style: "short" })),
    override_format: (amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => "Locale override: " + String(formatNumber(amount)) + " / " + String(formatDate(date, { style: "long" })),
    size_line: (size: __lgl_name_6D61696E2E53697A65) => String(__lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C) + ": " + String(__lgl_name_6D61696E2E53697A65576F7264(size)),
  },
  web: {
    kicker: "SvelteKit",
    title: "Lokalisierter SvelteKit ohne handgeschriebene i18n-Verdrahtung",
    intro: "Write normal SvelteKit routes and generated message calls. Linguini keeps the active locale, updates messages reactively, and turns invalid message arguments into TypeScript errors.",
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
