export type Fruit = "apple" | "pear" | "orange";

export type Size = "small" | "big";

export type Money = number;

export type ShortDate = string;

const FruitForms = {
  apple: { gender: "neuter", emoji: "🍎", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "яблоко", few: "яблока", many: "яблок", other: "яблока" }), gen: (value: number | string) => selectBranch(pluralRu(value), { one: "яблока", few: "яблок", many: "яблок", other: "яблока" }), display: { short: "ябл.", long: "спелое яблоко" } },
  pear: { gender: "female", emoji: "🍐", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "груша", few: "груши", many: "груш", other: "груши" }), gen: (value: number | string) => selectBranch(pluralRu(value), { one: "груши", few: "груш", many: "груш", other: "груши" }) },
  orange: { gender: "male", emoji: "🍊", nom: (value: number | string) => selectBranch(pluralRu(value), { one: "апельсин", few: "апельсина", many: "апельсинов", other: "апельсина" }), gen: (value: number | string) => selectBranch(pluralRu(value), { one: "апельсина", few: "апельсинов", many: "апельсинов", other: "апельсина" }) },
} as const;

const SizeForms = {
  small: (gender: string) => selectBranch(gender, { male: "маленький", female: "маленькая", neuter: "маленькое", other: "маленький" }),
  big: (gender: string) => selectBranch(gender, { male: "большой", female: "большая", neuter: "большое", other: "большой" }),
} as const;

function delivered(gender: string): string {
  if (gender === "male") return "Доставлен";
  if (gender === "female") return "Доставлена";
  if (gender === "neuter") return "Доставлено";
  if (gender === "other") return "Доставлено";
  return "";
}

function adjective(size: string, gender: string): string {
  if (size === "small" && gender === "male") return "маленький";
  if (size === "small" && gender === "female") return "маленькая";
  if (size === "small" && gender === "neuter") return "маленькое";
  if (size === "big" && gender === "male") return "большой";
  if (size === "big" && gender === "female") return "большая";
  if (size === "big" && gender === "neuter") return "большое";
  return "обычное";
}

/**  Displayed on the product delivery confirmation card. */
export function delivery(fruit: Fruit, size: Size, count: number): string {
  return String(delivered(FruitForms[fruit].gender)) + " " + String(adjective(size, FruitForms[fruit].gender)) + " " + String(FruitForms[fruit].nom(count));
}

/**  Shown near cart item count. */
export function counted(count: number, fruit: Fruit): string {
  return "В корзине " + String(count) + " " + String(FruitForms[fruit].gen(count));
}

export function price(amount: Money, date: ShortDate): string {
  return "Цена " + String(amount) + " на " + String(date);
}

export function email_input_label(): string {
  return "Email";
}

export function email_input_placeholder(): string {
  return "name@example.com";
}

export function email_input_aria(): string {
  return "Адрес электронной почты";
}

function selectBranch(key: string, branches: Record<string, string>): string {
  return branches[key] ?? branches.other ?? "";
}
