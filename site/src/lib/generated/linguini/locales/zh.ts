import { formatCurrency, formatDate, formatNumber, selectBranch } from "../shared";

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

export type Fruit = "apple" | "pear" | "orange";

export type Size = "small" | "big";

export type Gender = "male" | "female" | "neuter" | "other";

export type Money = number;

export type ShortDate = string;

export type Measurement = number;

const FruitForms = {
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralZh(value), { _: "苹果" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralZh(value), { _: "梨" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralZh(value), { _: "橙子" }) },
} as const;

function SizeWord(p0: string | number): string {
  return selectBranch(String(p0), { small: "小", big: "大" });
}

function GenderWord(p0: string | number): string {
  return selectBranch(String(p0), { male: "阳性", female: "阴性", neuter: "中性", _: "中性" });
}

function Delivered(p0: string | number, p1: string | number): string {
  return selectBranch(pluralZh(p0), { _: "已配送" });
}

export const main = {
  nav_why: "为什么",
  nav_language: "语言",
  nav_codegen: "代码生成",
  nav_web: "Web",
  locale_label: "语言",
  hero_eyebrow: "面向产品团队的类型化本地化",
  hero_title: "Linguini",
  hero_copy: "一种编译型本地化语言，把 schema、locale 语法、CLDR 格式化、SvelteKit hooks、cookie 和本地化路由放在同一个生成流程里。",
  primary_cta: "阅读文档",
  secondary_cta: "查看 GitHub",
  schema_chip: "schema 定义契约",
  locale_chip: "locale 定义语言",
  generated_chip: "app 导入生成代码",
  proof_kicker: "Pipeline",
  proof_title: "从 schema 到 SvelteKit 的真实 Linguini",
  feature_schema: "页面文本、enum、TypeKind aliases 和 formatter defaults 都在 .lgs 文件中。",
  feature_locale: "Locale 文件实现 plural、gender、size、forms 和 functions，无需 runtime parsing。",
  feature_cldr: "Number、currency 和 date 示例使用 Linguini 生成的 CLDR formatters。",
  feature_web: "SvelteKit hooks 把 locale 保存到 cookie，并使用 /en/... 和 /ru/... 路由。",
  sample_kicker: "生成输出",
  sample_title: "在 schema 中定义格式化，locale 不需要重复",
  reference_cta: "参考",
  playground_kicker: "实时 Playground",
  playground_title: "修改 count、fruit、size、gender、amount、date 或 locale。",
  count_label: "数量",
  fruit_label: "水果",
  size_label: "大小",
  gender_label: "性别",
  amount_label: "金额",
  date_label: "日期",
  localized_path_label: "本地化路由",
  cookie_label: "Cookie",
  route_label: "路由前缀",
  playground_sentence: (fruit: Fruit, size: Size, gender: Gender, count: number, amount: Money, date: ShortDate) => String(Delivered(count, gender)) + " " + String(count) + " 个 " + String(SizeWord(size)) + " " + String(FruitForms[fruit].nom(count)) + "，性别形式 " + String(GenderWord(gender)) + "。总计 " + String(formatCurrency(amount, { locale: "zh", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "y年M月d日EEEE", long: "y年M月d日", medium: "y年M月d日", short: "y/M/d" }, timeFormats: { full: "zzzz HH:mm:ss", long: "z HH:mm:ss", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1} {0}", short: "{1} {0}" } } }, { code: "CNY" })) + "；日期 " + String(formatDate(date, { locale: "zh", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "y年M月d日EEEE", long: "y年M月d日", medium: "y年M月d日", short: "y/M/d" }, timeFormats: { full: "zzzz HH:mm:ss", long: "z HH:mm:ss", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1} {0}", short: "{1} {0}" } } }, { style: "short" })) + ".",
  cart_summary: (count: number, fruit: Fruit) => "购物车中有 " + String(count) + " 个 " + String(FruitForms[fruit].nom(count)),
  number_format: (value: Measurement) => "数字格式：" + String(formatNumber(value, { locale: "zh", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "y年M月d日EEEE", long: "y年M月d日", medium: "y年M月d日", short: "y/M/d" }, timeFormats: { full: "zzzz HH:mm:ss", long: "z HH:mm:ss", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1} {0}", short: "{1} {0}" } } })),
  currency_format: (amount: Money) => "货币格式：" + String(formatCurrency(amount, { locale: "zh", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "y年M月d日EEEE", long: "y年M月d日", medium: "y年M月d日", short: "y/M/d" }, timeFormats: { full: "zzzz HH:mm:ss", long: "z HH:mm:ss", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1} {0}", short: "{1} {0}" } } }, { code: "CNY" })),
  date_format: (date: ShortDate) => "日期格式：" + String(formatDate(date, { locale: "zh", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "y年M月d日EEEE", long: "y年M月d日", medium: "y年M月d日", short: "y/M/d" }, timeFormats: { full: "zzzz HH:mm:ss", long: "z HH:mm:ss", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1} {0}", short: "{1} {0}" } } }, { style: "short" })),
  override_format: (amount: Money, date: ShortDate) => "Locale override: " + String(formatNumber(amount, { locale: "zh", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "y年M月d日EEEE", long: "y年M月d日", medium: "y年M月d日", short: "y/M/d" }, timeFormats: { full: "zzzz HH:mm:ss", long: "z HH:mm:ss", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1} {0}", short: "{1} {0}" } } })) + " / " + String(formatDate(date, { locale: "zh", numbers: { decimalSymbol: ".", groupSymbol: ",", decimalPattern: "#,##0.###", percentPattern: "#,##0%" }, currency: { standardPattern: "¤#,##0.00", accountingPattern: "¤#,##0.00;(¤#,##0.00)" }, date: { dateFormats: { full: "y年M月d日EEEE", long: "y年M月d日", medium: "y年M月d日", short: "y/M/d" }, timeFormats: { full: "zzzz HH:mm:ss", long: "z HH:mm:ss", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1} {0}", long: "{1} {0}", medium: "{1} {0}", short: "{1} {0}" } } }, { style: "long" })),
  gender_line: (gender: Gender) => "性别形式：" + String(GenderWord(gender)),
  size_line: (size: Size) => "大小形式：" + String(SizeWord(size)),
} as const;

const lgl = {
  main,
} as const;

export default lgl;
