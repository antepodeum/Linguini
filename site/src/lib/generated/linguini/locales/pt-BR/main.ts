import type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../../shared";
import { selectBranch } from "../../shared";

function pluralPtBr(value: number | string): string {
  const operands = pluralOperands(value);
  if ((((operands.i >= 0 && operands.i <= 1)))) return "one";
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
    return formatGeneratedNumber(Number(value), "" + symbol + " ", "", undefined, undefined, 1, 2, 2, 3, undefined, ",", ".");
  }
  return formatGeneratedNumber(Number(value), "" + symbol + " ", "", undefined, undefined, 1, 2, 2, 3, undefined, ",", ".");
}

function currencySymbol(currency: string): string {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency })
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
      return ["domingo", "segunda-feira", "terça-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sábado"][date.getDay()] + ", " + String(date.getDate()) + " " + "de" + " " + ["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"][date.getMonth()] + " " + "de" + " " + String(date.getFullYear());
    case "long":
      return String(date.getDate()) + " " + "de" + " " + ["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"][date.getMonth()] + " " + "de" + " " + String(date.getFullYear());
    case "short":
      return padNumber(date.getDate(), 2) + "/" + padNumber(date.getMonth() + 1, 2) + "/" + String(date.getFullYear());
    default:
      return String(date.getDate()) + " " + "de" + " " + ["jan.", "fev.", "mar.", "abr.", "mai.", "jun.", "jul.", "ago.", "set.", "out.", "nov.", "dez."][date.getMonth()] + " " + "de" + " " + String(date.getFullYear());
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

const __lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C = "Forma de tamanho";

const __lgl_form_6D61696E2E4672756974 = {
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralPtBr(value), { one: "maçã", _: "maçãs" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralPtBr(value), { one: "pera", _: "peras" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralPtBr(value), { one: "laranja", _: "laranjas" }) },
} as const;

function __lgl_name_6D61696E2E53697A65576F7264(p0: string | number): string {
  return selectBranch(String(p0), { small: "pequeno", big: "grande" });
}

function __lgl_name_6D61696E2E53697A6541646A(p0: string | number, p1: string | number, p2: string | number): string {
  return selectBranch(String(p0), { small: selectBranch(pluralPtBr(p1), { one: selectBranch(String(p2), { male: "pequeno", female: "pequena", neuter: "pequeno", _: "pequeno" }), _: selectBranch(String(p2), { male: "pequenos", female: "pequenas", neuter: "pequenos", _: "pequenos" }) }), big: selectBranch(pluralPtBr(p1), { one: selectBranch(String(p2), { male: "grande", female: "grande", neuter: "grande", _: "grande" }), _: selectBranch(String(p2), { male: "grandes", female: "grandes", neuter: "grandes", _: "grandes" }) }), _: "normais" });
}

function __lgl_name_6D61696E2E44656C697665726564(p0: string | number, p1: string | number): string {
  return selectBranch(pluralPtBr(p0), { one: selectBranch(String(p1), { _: "Entregue" }), _: "Entregues" });
}

export const main = {
  codegen: {
    kicker: "Geração de código",
    title: "Um schema, saída tipada para cada runtime",
    intro: "O Linguini analisa contratos .lgs e lógica .lgl uma vez e emite módulos com formatadores CLDR e adaptadores de framework no build.",
    ts_title: "TypeScript / ESM",
    ts_desc: "Funções de mensagem tipadas, runtime compartilhado e declarações TypeScript.",
    svelte_title: "Svelte 5 + SvelteKit",
    svelte_desc: "Rune l gerado, troca de locale e exports handle / reroute / load.",
    planned_title: "Alvos planejados",
    planned_intro: "O roadmap é amplo de propósito: Linguini pretende cobrir todos os runtimes, linguagens e frameworks web práticos.",
    rust: "Rust",
    kotlin: "Kotlin",
    swift: "Swift",
    go: "Go",
    python: "Python",
    csharp: "C#",
    status_shipped: "Disponível",
    status_planned: "Planejado",
  },
  hero: {
    eyebrow: "Localização tipada para times de produto",
    title: "Linguini",
    tagline: "Internacionalização para todos nós",
    copy: "Uma linguagem de localização compilada em que schemas, gramática, formatação CLDR e integração web saem de uma única fonte.",
    term: "/lɪŋˈɡwiːni/",
    term_kind: "NOME",
    term_hint: "tiras longas e estreitas de massa",
    intro: "Linguini transforma arquivos de localização tipados em código que qualquer app pode usar. Mensagens, gramática, formatadores, fallback de locales e rotas web são verificados no build.",
    trait_typed: "Tipado",
    trait_compiled: "Compilado",
    trait_native: "Nativo",
    primary_cta: "Ler a documentação",
    secondary_cta: "Ver no GitHub",
  },
  nav: {
    why: "Por quê",
    language: "Idioma",
    codegen: "Codegen",
    web: "Web",
    locale_label: "Idioma",
  },
  playground: {
    kicker: "Playground ao vivo",
    title: "Altere count, fruit, size, amount, date ou locale.",
    count_label: "Quantidade",
    fruit_label: "Fruta",
    fruit_apple_label: "maçã",
    fruit_pear_label: "pera",
    fruit_orange_label: "laranja",
    size_label: "Tamanho",
    size_small_label: "pequeno",
    size_big_label: "grande",
    amount_label: "Valor",
    date_label: "Data",
    localized_path_label: "Rota localizada",
    cookie_label: "Cookie",
    route_label: "Prefixo da rota",
    sentence: (fruit: __lgl_name_6D61696E2E4672756974, size: __lgl_name_6D61696E2E53697A65, count: number, amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => String(__lgl_name_6D61696E2E44656C697665726564(count, __lgl_form_6D61696E2E4672756974[fruit].Gender)) + " " + String(formatNumber(count)) + " " + String(__lgl_name_6D61696E2E53697A6541646A(size, count, __lgl_form_6D61696E2E4672756974[fruit].Gender)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)) + ". Total " + String(formatCurrency(amount, { code: "BRL" })) + "; data " + String(formatDate(date, { style: "short" })) + ".",
    cart_summary: (count: number, fruit: __lgl_name_6D61696E2E4672756974) => "O carrinho tem " + String(formatNumber(count)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)),
    number_format: (value: __lgl_name_6D61696E2E4D6561737572656D656E74) => "Formato numérico: " + String(formatNumber(value)),
    currency_format: (amount: __lgl_name_6D61696E2E4D6F6E6579) => "Formato de moeda: " + String(formatCurrency(amount, { code: "BRL" })),
    date_format: (date: __lgl_name_6D61696E2E53686F727444617465) => "Formato de data: " + String(formatDate(date, { style: "short" })),
    override_format: (amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => "Locale override: " + String(formatNumber(amount)) + " / " + String(formatDate(date, { style: "long" })),
    size_line: (size: __lgl_name_6D61696E2E53697A65) => String(__lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C) + ": " + String(__lgl_name_6D61696E2E53697A65576F7264(size)),
  },
  web: {
    kicker: "SvelteKit",
    title: "SvelteKit localizado sem ligação i18n manual",
    intro: "Write normal SvelteKit routes and generated message calls. Linguini keeps the active locale, updates messages reactively, and turns invalid message arguments into TypeScript errors.",
    routing: "Use ordinary internal links; the generated reroute hook handles localized URLs.",
    cookie: "Locale changes persist through cookies and browser storage without app-level state.",
    fallback: "Os dados da locale base são incorporados nos módulos gerados durante o build.",
    reactivity: "The l rune updates UI text immediately after setLocale.",
    links: "Alguns links podem manter a URL original quando não devem ser localizados.",
  },
} as const;

const lgl = {
  main,
} as const;

export default lgl;
