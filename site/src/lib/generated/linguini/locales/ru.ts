import { formatCurrency, formatDate, formatNumber, selectBranch } from "../shared";

function pluralRu(value: number | string): string {
  const operands = pluralOperands(value);
  if (((operands.v === 0) && ((operands.i % 10) === 1) && !((operands.i % 100) === 11))) return "one";
  if (((operands.v === 0) && (((operands.i % 10) >= 2 && (operands.i % 10) <= 4)) && !(((operands.i % 100) >= 12 && (operands.i % 100) <= 14)))) return "few";
  if (((operands.v === 0) && ((operands.i % 10) === 0)) || ((operands.v === 0) && (((operands.i % 10) >= 5 && (operands.i % 10) <= 9))) || ((operands.v === 0) && (((operands.i % 100) >= 11 && (operands.i % 100) <= 14)))) return "many";
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
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "яблоко", few: "яблока", _: "яблок" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "груша", few: "груши", _: "груш" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "апельсин", few: "апельсина", _: "апельсинов" }) },
} as const;

function SizeWord(p0: string | number): string {
  return selectBranch(String(p0), { small: "маленький", big: "большой" });
}

function GenderWord(p0: string | number): string {
  return selectBranch(String(p0), { male: "мужской", female: "женский", neuter: "средний", _: "нейтральный" });
}

function Delivered(p0: string | number, p1: string | number): string {
  return selectBranch(pluralRu(p0), { one: selectBranch(String(p1), { male: "Доставлен", female: "Доставлена", neuter: "Доставлено", _: "Доставлено" }), _: "Доставлены" });
}

export const main = {
  nav_why: "Зачем",
  nav_language: "Язык",
  nav_codegen: "Кодген",
  nav_web: "Web",
  locale_label: "Локаль",
  hero_eyebrow: "Типизированная локализация для продуктовых команд",
  hero_title: "Linguini",
  hero_copy: "Компилируемый язык локализации, где схемы, грамматика локали, CLDR-форматирование, SvelteKit hooks, cookie и локализованные маршруты генерируются из одного источника.",
  primary_cta: "Документация",
  secondary_cta: "GitHub",
  schema_chip: "схема задает контракт",
  locale_chip: "локаль задает язык",
  generated_chip: "приложение импортирует сгенерированный код",
  proof_kicker: "Pipeline",
  proof_title: "Настоящий Linguini от схемы до SvelteKit",
  feature_schema: "Текст сайта, enum, TypeKind aliases и formatter defaults лежат в .lgs файлах.",
  feature_locale: "Locale files реализуют plural, gender, size, forms и functions без runtime parsing.",
  feature_cldr: "Примеры number, currency и date используют CLDR formatters из Linguini codegen.",
  feature_web: "SvelteKit hooks сохраняют локаль в cookie и ведут страницы как /en/... и /ru/....",
  sample_kicker: "Сгенерированный output",
  sample_title: "Форматирование в схеме, без шума в локали",
  reference_cta: "Reference",
  playground_kicker: "Live playground",
  playground_title: "Меняй count, fruit, size, gender, amount, date или locale.",
  count_label: "Количество",
  fruit_label: "Фрукт",
  size_label: "Размер",
  gender_label: "Род",
  amount_label: "Сумма",
  date_label: "Дата",
  localized_path_label: "Локализованный route",
  cookie_label: "Cookie",
  route_label: "Route prefix",
  playground_sentence: (fruit: Fruit, size: Size, gender: Gender, count: number, amount: Money, date: ShortDate) => String(Delivered(count, gender)) + " " + String(count) + " " + String(SizeWord(size)) + " " + String(FruitForms[fruit].nom(count)) + " для элемента с родом " + String(GenderWord(gender)) + ". Итого " + String(formatCurrency(amount, { locale: "ru", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d MMMM y 'г'.", long: "d MMMM y 'г'.", medium: "d MMM y 'г'.", short: "dd.MM.y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { code: "RUB" })) + "; дата " + String(formatDate(date, { locale: "ru", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d MMMM y 'г'.", long: "d MMMM y 'г'.", medium: "d MMM y 'г'.", short: "dd.MM.y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "short" })) + ".",
  cart_summary: (count: number, fruit: Fruit) => "В корзине " + String(count) + " " + String(FruitForms[fruit].nom(count)),
  number_format: (value: Measurement) => "Формат числа: " + String(formatNumber(value, { locale: "ru", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d MMMM y 'г'.", long: "d MMMM y 'г'.", medium: "d MMM y 'г'.", short: "dd.MM.y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } })),
  currency_format: (amount: Money) => "Формат валюты: " + String(formatCurrency(amount, { locale: "ru", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d MMMM y 'г'.", long: "d MMMM y 'г'.", medium: "d MMM y 'г'.", short: "dd.MM.y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { code: "RUB" })),
  date_format: (date: ShortDate) => "Формат даты: " + String(formatDate(date, { locale: "ru", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d MMMM y 'г'.", long: "d MMMM y 'г'.", medium: "d MMM y 'г'.", short: "dd.MM.y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "short" })),
  override_format: (amount: Money, date: ShortDate) => "Locale override: " + String(formatNumber(amount, { locale: "ru", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d MMMM y 'г'.", long: "d MMMM y 'г'.", medium: "d MMM y 'г'.", short: "dd.MM.y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } })) + " / " + String(formatDate(date, { locale: "ru", numbers: { decimalSymbol: ",", groupSymbol: " ", decimalPattern: "#,##0.###", percentPattern: "#,##0 %" }, currency: { standardPattern: "#,##0.00 ¤", accountingPattern: "#,##0.00 ¤" }, date: { dateFormats: { full: "EEEE, d MMMM y 'г'.", long: "d MMMM y 'г'.", medium: "d MMM y 'г'.", short: "dd.MM.y" }, timeFormats: { full: "HH:mm:ss zzzz", long: "HH:mm:ss z", medium: "HH:mm:ss", short: "HH:mm" }, dateTimeFormats: { full: "{1}, {0}", long: "{1}, {0}", medium: "{1}, {0}", short: "{1}, {0}" } } }, { style: "long" })),
  gender_line: (gender: Gender) => "Форма рода: " + String(GenderWord(gender)),
  size_line: (size: Size) => "Форма размера: " + String(SizeWord(size)),
} as const;

const lgl = {
  main,
} as const;

export default lgl;
