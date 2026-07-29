import type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../../shared";
import { selectBranch } from "../../shared";

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
  return new Intl.NumberFormat("es", { style: "currency", currency })
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
      return ["domingo", "lunes", "martes", "miércoles", "jueves", "viernes", "sábado"][date.getDay()] + ", " + String(date.getDate()) + " " + "de" + " " + ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"][date.getMonth()] + " " + "de" + " " + String(date.getFullYear());
    case "long":
      return String(date.getDate()) + " " + "de" + " " + ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"][date.getMonth()] + " " + "de" + " " + String(date.getFullYear());
    case "short":
      return String(date.getDate()) + "/" + String(date.getMonth() + 1) + "/" + padNumber(date.getFullYear() % 100, 2);
    default:
      return String(date.getDate()) + " " + ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sept", "oct", "nov", "dic"][date.getMonth()] + " " + String(date.getFullYear());
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

const __lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C = "Forma de tamaño";

const __lgl_form_6D61696E2E4672756974 = {
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralEs(value), { one: "manzana", _: "manzanas" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralEs(value), { one: "pera", _: "peras" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralEs(value), { one: "naranja", _: "naranjas" }) },
} as const;

function __lgl_name_6D61696E2E53697A65576F7264(p0: string | number): string {
  return selectBranch(String(p0), { small: "pequeño", big: "grande" });
}

function __lgl_name_6D61696E2E53697A6541646A(p0: string | number, p1: string | number, p2: string | number): string {
  return selectBranch(String(p0), { small: selectBranch(pluralEs(p1), { one: selectBranch(String(p2), { male: "pequeño", female: "pequeña", neuter: "pequeño", _: "pequeño" }), _: selectBranch(String(p2), { male: "pequeños", female: "pequeñas", neuter: "pequeños", _: "pequeños" }) }), big: selectBranch(pluralEs(p1), { one: selectBranch(String(p2), { male: "grande", female: "grande", neuter: "grande", _: "grande" }), _: selectBranch(String(p2), { male: "grandes", female: "grandes", neuter: "grandes", _: "grandes" }) }), _: "normales" });
}

function __lgl_name_6D61696E2E44656C697665726564(p0: string | number, p1: string | number): string {
  return selectBranch(pluralEs(p0), { one: selectBranch(String(p1), { _: "Entregado" }), _: "Entregados" });
}

export const main = {
  codegen: {
    kicker: "Generación de código",
    title: "Un esquema, salida tipada para cada runtime",
    intro: "Linguini analiza contratos .lgs y lógica .lgl una vez, luego emite módulos específicos con formateadores CLDR, selectores plurales y adaptadores de framework en tiempo de compilación.",
    ts_title: "TypeScript / ESM",
    ts_desc: "Funciones de mensajes tipadas, runtime compartido, archivos de declaración y tree shaking opcional por locale.",
    svelte_title: "Svelte 5 + SvelteKit",
    svelte_desc: "Rune l generado, cambio de locale, helpers localizeHref y exports handle / reroute / load.",
    planned_title: "Targets planificados",
    planned_intro: "La hoja de ruta es amplia a propósito: Linguini planea cubrir todo runtime, lenguaje y framework web práctico.",
    rust: "Rust",
    kotlin: "Kotlin",
    swift: "Swift",
    go: "Go",
    python: "Python",
    csharp: "C#",
    status_shipped: "Disponible",
    status_planned: "Planificado",
  },
  hero: {
    eyebrow: "Localización tipada para equipos de producto",
    title: "Linguini",
    tagline: "Internacionalización para el resto de nosotros",
    copy: "Un lenguaje de localización compilado donde esquemas, gramática, formato CLDR, hooks de SvelteKit, cookies y rutas localizadas se generan desde una sola fuente.",
    term: "/lɪŋˈɡwiːni/",
    term_kind: "SUST.",
    term_hint: "tiras largas y estrechas de pasta",
    intro: "Linguini convierte archivos de localización tipados en código que puede usar cualquier app. Mensajes, gramática, formateadores, fallback de locales y rutas web se validan durante el build.",
    trait_typed: "Tipado",
    trait_compiled: "Compilado",
    trait_native: "Nativo",
    primary_cta: "Leer docs",
    secondary_cta: "Ver GitHub",
  },
  nav: {
    why: "Por qué",
    language: "Idioma",
    codegen: "Codegen",
    web: "Web",
    locale_label: "Idioma",
  },
  playground: {
    kicker: "Playground en vivo",
    title: "Cambia count, fruit, size, amount, date o locale.",
    count_label: "Cantidad",
    fruit_label: "Fruta",
    fruit_apple_label: "manzana",
    fruit_pear_label: "pera",
    fruit_orange_label: "naranja",
    size_label: "Tamaño",
    size_small_label: "pequeño",
    size_big_label: "grande",
    amount_label: "Importe",
    date_label: "Fecha",
    localized_path_label: "Ruta localizada",
    cookie_label: "Cookie",
    route_label: "Prefijo de ruta",
    sentence: (fruit: __lgl_name_6D61696E2E4672756974, size: __lgl_name_6D61696E2E53697A65, count: number, amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => String(__lgl_name_6D61696E2E44656C697665726564(count, __lgl_form_6D61696E2E4672756974[fruit].Gender)) + " " + String(formatNumber(count)) + " " + String(__lgl_name_6D61696E2E53697A6541646A(size, count, __lgl_form_6D61696E2E4672756974[fruit].Gender)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)) + ". Total " + String(formatCurrency(amount, { code: "EUR" })) + "; fecha " + String(formatDate(date, { style: "short" })) + ".",
    cart_summary: (count: number, fruit: __lgl_name_6D61696E2E4672756974) => "El carrito tiene " + String(formatNumber(count)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)),
    number_format: (value: __lgl_name_6D61696E2E4D6561737572656D656E74) => "Formato numérico: " + String(formatNumber(value)),
    currency_format: (amount: __lgl_name_6D61696E2E4D6F6E6579) => "Formato moneda: " + String(formatCurrency(amount, { code: "EUR" })),
    date_format: (date: __lgl_name_6D61696E2E53686F727444617465) => "Formato fecha: " + String(formatDate(date, { style: "short" })),
    override_format: (amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => "Locale override: " + String(formatNumber(amount)) + " / " + String(formatDate(date, { style: "long" })),
    size_line: (size: __lgl_name_6D61696E2E53697A65) => String(__lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C) + ": " + String(__lgl_name_6D61696E2E53697A65576F7264(size)),
  },
  web: {
    kicker: "SvelteKit",
    title: "SvelteKit localizado sin cableado i18n manual",
    intro: "Write normal SvelteKit routes and generated message calls. Linguini keeps the active locale, updates messages reactively, and turns invalid message arguments into TypeScript errors.",
    routing: "Use ordinary internal links; the generated reroute hook handles localized URLs.",
    cookie: "Locale changes persist through cookies and browser storage without app-level state.",
    fallback: "Los datos de la locale base se integran en los módulos generados durante el build.",
    reactivity: "The l rune updates UI text immediately after setLocale.",
    links: "Algunos enlaces pueden conservar su URL original cuando no deben localizarse.",
  },
} as const;

const lgl = {
  main,
} as const;

export default lgl;
