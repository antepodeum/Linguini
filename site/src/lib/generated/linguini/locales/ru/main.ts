import type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../../shared";
import { selectBranch } from "../../shared";

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

type GeneratedCurrencyFormatterOptions = { code?: string; accounting?: "true" | "false" };
type GeneratedDateFormatterOptions = { style?: "full" | "long" | "medium" | "short" };

function formatNumber(value: number | string): string {
  return formatGeneratedNumber(Number(value), "", "", undefined, undefined, 1, 0, 3, 3, undefined, ",", " ");
}

function formatCurrency(
  value: number | string,
  options: GeneratedCurrencyFormatterOptions = {},
): string {
  const symbol = currencySymbol(options.code ?? "USD");
  if (options.accounting === "true") {
    return formatGeneratedNumber(Number(value), "", " " + symbol + "", undefined, undefined, 1, 2, 2, 3, undefined, ",", " ");
  }
  return formatGeneratedNumber(Number(value), "", " " + symbol + "", undefined, undefined, 1, 2, 2, 3, undefined, ",", " ");
}

function currencySymbol(currency: string): string {
  return new Intl.NumberFormat("ru", { style: "currency", currency })
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
      return ["воскресенье", "понедельник", "вторник", "среда", "четверг", "пятница", "суббота"][date.getDay()] + ", " + String(date.getDate()) + " " + ["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"][date.getMonth()] + " " + String(date.getFullYear()) + " " + "г" + ".";
    case "long":
      return String(date.getDate()) + " " + ["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"][date.getMonth()] + " " + String(date.getFullYear()) + " " + "г" + ".";
    case "short":
      return padNumber(date.getDate(), 2) + "." + padNumber(date.getMonth() + 1, 2) + "." + String(date.getFullYear());
    default:
      return String(date.getDate()) + " " + ["янв.", "февр.", "мар.", "апр.", "мая", "июн.", "июл.", "авг.", "сент.", "окт.", "нояб.", "дек."][date.getMonth()] + " " + String(date.getFullYear()) + " " + "г" + ".";
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

const __lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C = "Форма размера";

const __lgl_form_6D61696E2E4672756974 = {
  apple: { Gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "яблоко", few: "яблока", _: "яблок" }) },
  pear: { Gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "груша", few: "груши", _: "груш" }) },
  orange: { Gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "апельсин", few: "апельсина", _: "апельсинов" }) },
} as const;

function __lgl_name_6D61696E2E53697A65576F7264(p0: string | number): string {
  return selectBranch(String(p0), { small: "маленький", big: "большой", _: "обычный" });
}

function __lgl_name_6D61696E2E53697A6541646A(p0: string | number, p1: string | number, p2: string | number): string {
  return selectBranch(String(p0), { small: selectBranch(pluralRu(p1), { one: selectBranch(String(p2), { male: "маленький", female: "маленькая", neuter: "маленькое", _: "маленький" }), few: selectBranch(String(p2), { female: "маленькие", _: "маленьких" }), _: "маленьких" }), big: selectBranch(pluralRu(p1), { one: selectBranch(String(p2), { male: "большой", female: "большая", neuter: "большое", _: "большой" }), few: selectBranch(String(p2), { female: "большие", _: "больших" }), _: "больших" }), _: selectBranch(pluralRu(p1), { one: selectBranch(String(p2), { male: "обычный", female: "обычная", neuter: "обычное", _: "обычный" }), few: selectBranch(String(p2), { female: "обычные", _: "обычных" }), _: "обычных" }) });
}

function __lgl_name_6D61696E2E44656C697665726564(p0: string | number, p1: string | number): string {
  return selectBranch(pluralRu(p0), { one: selectBranch(String(p1), { male: "Доставлен", female: "Доставлена", neuter: "Доставлено", _: "Доставлено" }), _: "Доставлено" });
}

export const main = {
  codegen: {
    kicker: "Генерация кода",
    title: "Одна схема — типизированный вывод для каждого рантайма",
    intro: "Linguini один раз анализирует контракты .lgs и логику .lgl, затем на этапе сборки генерирует модули под целевой рантайм с CLDR-форматированием, plural-селекторами и адаптерами фреймворков.",
    ts_title: "TypeScript / ESM",
    ts_desc: "Типизированные функции сообщений, общий runtime, файлы деклараций и tree-shaking по локалям.",
    svelte_title: "Svelte 5 + SvelteKit",
    svelte_desc: "Сгенерированный rune l, переключение локали, localizeHref и экспорты handle / reroute / load.",
    planned_title: "Запланированные таргеты",
    planned_intro: "План намеренно широкий: Linguini должен поддерживать все практичные рантаймы, языки и web-фреймворки.",
    rust: "Rust",
    kotlin: "Kotlin",
    swift: "Swift",
    go: "Go",
    python: "Python",
    csharp: "C#",
    status_shipped: "Стабильно",
    status_planned: "В планах",
  },
  hero: {
    eyebrow: "Типизированная локализация для продуктовых команд",
    title: "Linguini",
    tagline: "Интернационализация для всех нас",
    copy: "Компилируемый язык локализации, где схемы, грамматика локали, CLDR-форматирование, хуки SvelteKit, cookie-файлы и локализованные маршруты генерируются из одного источника.",
    term: "/lɪŋˈɡwiːni/",
    term_kind: "СУЩ.",
    term_hint: "длинная тонкая ленточная паста",
    intro: "Linguini превращает типизированные файлы локализации в код, который можно подключить к любому приложению. Сообщения, грамматика, форматтеры, fallback-локали и web-маршруты проверяются на этапе сборки.",
    trait_typed: "Типизированный",
    trait_compiled: "Компилируемый",
    trait_native: "Нативный",
    primary_cta: "Документация",
    secondary_cta: "GitHub",
  },
  nav: {
    why: "Зачем",
    language: "Язык",
    codegen: "Кодогенерация",
    web: "Веб",
    locale_label: "Локаль",
  },
  playground: {
    kicker: "Интерактивная песочница",
    title: "Меняйте количество, фрукт, размер, род, сумму, дату или локаль.",
    count_label: "Количество",
    fruit_label: "Фрукт",
    fruit_apple_label: "яблоко",
    fruit_pear_label: "груша",
    fruit_orange_label: "апельсин",
    size_label: "Размер",
    size_small_label: "маленький",
    size_big_label: "большой",
    amount_label: "Сумма",
    date_label: "Дата",
    localized_path_label: "Локализованный маршрут",
    cookie_label: "Cookie",
    route_label: "Префикс маршрута",
    sentence: (fruit: __lgl_name_6D61696E2E4672756974, size: __lgl_name_6D61696E2E53697A65, count: number, amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => String(__lgl_name_6D61696E2E44656C697665726564(count, __lgl_form_6D61696E2E4672756974[fruit].Gender)) + " " + String(formatNumber(count)) + " " + String(__lgl_name_6D61696E2E53697A6541646A(size, count, __lgl_form_6D61696E2E4672756974[fruit].Gender)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)) + ". Итого " + String(formatCurrency(amount, { code: "RUB" })) + "; дата " + String(formatDate(date, { style: "short" })) + ".",
    cart_summary: (count: number, fruit: __lgl_name_6D61696E2E4672756974) => "В корзине " + String(formatNumber(count)) + " " + String(__lgl_form_6D61696E2E4672756974[fruit].nom(count)),
    number_format: (value: __lgl_name_6D61696E2E4D6561737572656D656E74) => "Формат числа: " + String(formatNumber(value)),
    currency_format: (amount: __lgl_name_6D61696E2E4D6F6E6579) => "Формат валюты: " + String(formatCurrency(amount, { code: "RUB" })),
    date_format: (date: __lgl_name_6D61696E2E53686F727444617465) => "Формат даты: " + String(formatDate(date, { style: "short" })),
    override_format: (amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => "Переопределение локали: " + String(formatNumber(amount)) + " / " + String(formatDate(date, { style: "long" })),
    size_line: (size: __lgl_name_6D61696E2E53697A65) => String(__lgl_name_6D61696E2E73697A655F666F726D5F6C6162656C) + ": " + String(__lgl_name_6D61696E2E53697A65576F7264(size)),
  },
  web: {
    kicker: "SvelteKit",
    title: "Локализованный SvelteKit без ручной i18n-обвязки",
    intro: "Пишите обычные маршруты SvelteKit и вызывайте сгенерированные сообщения. Linguini хранит активную локаль, реактивно обновляет текст и превращает неверные аргументы сообщений в ошибки TypeScript.",
    routing: "Используйте обычные внутренние ссылки; сгенерированный хук reroute обрабатывает локализованные URL.",
    cookie: "Смена локали сохраняется через cookie и хранилище браузера без app-level state.",
    fallback: "Данные из базовой локали встраиваются в сгенерированные locale-модули на этапе сборки.",
    reactivity: "Rune l обновляет текст UI сразу после setLocale.",
    links: "Для отдельных ссылок можно оставить исходный URL нелокализованным.",
  },
} as const;

const lgl = {
  main,
} as const;

export default lgl;
