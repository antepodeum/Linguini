import type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../../shared";
import { selectBranch } from "../../shared";

function pluralZh(value: number | string): string {
  const operands = pluralOperands(value);
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
  return formatGeneratedNumber(Number(value), "", "", undefined, undefined, 1, 0, 3, 3, undefined, ".", ",");
}

function formatCurrency(
  value: number | string,
  options: GeneratedCurrencyFormatterOptions = {},
): string {
  const symbol = currencySymbol(options.code ?? "USD");
  if (options.accounting === "true") {
    return formatGeneratedNumber(Number(value), "" + symbol + "", "", "(" + symbol + "", ")", 1, 2, 2, 3, undefined, ".", ",");
  }
  return formatGeneratedNumber(Number(value), "" + symbol + "", "", undefined, undefined, 1, 2, 2, 3, undefined, ".", ",");
}

function currencySymbol(currency: string): string {
  return new Intl.NumberFormat("zh", { style: "currency", currency })
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
      return String(date.getFullYear()) + "年" + String(date.getMonth() + 1) + "月" + String(date.getDate()) + "日" + ["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"][date.getDay()];
    case "long":
      return String(date.getFullYear()) + "年" + String(date.getMonth() + 1) + "月" + String(date.getDate()) + "日";
    case "short":
      return String(date.getFullYear()) + "/" + String(date.getMonth() + 1) + "/" + String(date.getDate());
    default:
      return String(date.getFullYear()) + "年" + String(date.getMonth() + 1) + "月" + String(date.getDate()) + "日";
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

const __lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C = "大小形式";

const __lgl_form_6D61696E2E4672756974 = {
  apple: { emoji: "🍎", nom: (value: number | string) => selectBranch(pluralZh(value), { _: "苹果" }) },
  pear: { emoji: "🍐", nom: (value: number | string) => selectBranch(pluralZh(value), { _: "梨" }) },
  orange: { emoji: "🍊", nom: (value: number | string) => selectBranch(pluralZh(value), { _: "橙子" }) },
} as const;

function __lgl_name_6D61696E2E53697A65576F7264(p0: string | number): string {
  return selectBranch(String(p0), { small: "小", big: "大" });
}

export const main = {
  codegen: {
    kicker: "代码生成",
    title: "一份 schema，为每个运行时生成类型化输出",
    intro: "Linguini 一次性分析 .lgs 契约与 .lgl 语言逻辑，然后在构建期生成目标模块，内置 CLDR 格式化、复数选择器与框架适配器。",
    ts_title: "TypeScript / ESM",
    ts_desc: "类型化消息函数、共享 runtime、声明文件，以及按 locale 的可选 tree shaking。",
    svelte_title: "Svelte 5 + SvelteKit",
    svelte_desc: "生成 l rune、切换 locale、localizeHref 辅助函数，以及 handle / reroute / load 导出。",
    planned_title: "计划中的目标",
    planned_intro: "路线图刻意保持宽广：Linguini 计划支持所有实用的 runtime、语言和 Web 框架。",
    rust: "Rust",
    kotlin: "Kotlin",
    swift: "Swift",
    go: "Go",
    python: "Python",
    csharp: "C#",
    status_shipped: "已发布",
    status_planned: "计划中",
  },
  hero: {
    eyebrow: "面向产品团队的类型化本地化",
    title: "Linguini",
    tagline: "为每个人而做的国际化",
    copy: "一种编译型本地化语言，把 schema、locale 语法、CLDR 格式化、SvelteKit hooks、cookie 和本地化路由放在同一个生成流程里。",
    term: "/lɪŋˈɡwiːni/",
    term_kind: "名词",
    term_hint: "细长扁平的意式面",
    intro: "Linguini 将类型化本地化文件转换成任何应用都能使用的代码。消息、语法、格式化器、locale fallback 和 Web 路由都会在构建时检查。",
    trait_typed: "类型化",
    trait_compiled: "编译型",
    trait_native: "原生",
    primary_cta: "阅读文档",
    secondary_cta: "查看 GitHub",
  },
  nav: {
    why: "为什么",
    language: "语言",
    codegen: "代码生成",
    web: "Web",
    locale_label: "语言",
  },
  playground: {
    kicker: "实时 Playground",
    title: "修改 count、fruit、size、amount、date 或 locale。",
    count_label: "数量",
    fruit_label: "水果",
    fruit_apple_label: "苹果",
    fruit_pear_label: "梨",
    fruit_orange_label: "橙子",
    size_label: "大小",
    size_small_label: "小",
    size_big_label: "大",
    amount_label: "金额",
    date_label: "日期",
    localized_path_label: "本地化路由",
    cookie_label: "Cookie",
    route_label: "路由前缀",
    sentence: (fruit: __lgl_name_6D61696E2E4672756974, size: __lgl_name_6D61696E2E53697A65, count: number, amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => "已配送 " + String(formatNumber(count)) + " 个 " + String(__lgl_name_6D61696E2E53697A65576F7264(size)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)) + "。总计 " + String(formatCurrency(amount, { code: "CNY" })) + "；日期 " + String(formatDate(date, { style: "short" })) + ".",
    cart_summary: (count: number, fruit: __lgl_name_6D61696E2E4672756974) => "购物车中有 " + String(formatNumber(count)) + " 个 " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)),
    number_format: (value: __lgl_name_6D61696E2E4D6561737572656D656E74) => "数字格式： " + String(formatNumber(value)),
    currency_format: (amount: __lgl_name_6D61696E2E4D6F6E6579) => "货币格式： " + String(formatCurrency(amount, { code: "CNY" })),
    date_format: (date: __lgl_name_6D61696E2E53686F727444617465) => "日期格式： " + String(formatDate(date, { style: "short" })),
    override_format: (amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => "Locale override: " + String(formatNumber(amount)) + " / " + String(formatDate(date, { style: "long" })),
    size_line: (size: __lgl_name_6D61696E2E53697A65) => String(__lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C) + ": " + String(__lgl_name_6D61696E2E53697A65576F7264(size)),
  },
  web: {
    kicker: "SvelteKit",
    title: "无需手写 i18n 粘合代码的本地化 SvelteKit",
    intro: "Write normal SvelteKit routes and generated message calls. Linguini keeps the active locale, updates messages reactively, and turns invalid message arguments into TypeScript errors.",
    routing: "Use ordinary internal links; the generated reroute hook handles localized URLs.",
    cookie: "Locale changes persist through cookies and browser storage without app-level state.",
    fallback: "基础 locale 数据会在构建时写入生成的 locale 模块。",
    reactivity: "The l rune updates UI text immediately after setLocale.",
    links: "不需要本地化的链接可以保留原始 URL。",
  },
} as const;

const lgl = {
  main,
} as const;

export default lgl;
