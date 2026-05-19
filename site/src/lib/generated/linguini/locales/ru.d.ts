export type Fruit = "apple" | "pear" | "orange";

export type Size = "small" | "big";

export type Gender = "male" | "female" | "neuter" | "other";

export type Money = number;

export type ShortDate = string;

export type Measurement = number;

export declare const main: {
  readonly nav_why: string;
  readonly nav_language: string;
  readonly nav_codegen: string;
  readonly nav_web: string;
  readonly locale_label: string;
  readonly hero_eyebrow: string;
  readonly hero_title: string;
  readonly hero_copy: string;
  readonly primary_cta: string;
  readonly secondary_cta: string;
  readonly schema_chip: string;
  readonly locale_chip: string;
  readonly generated_chip: string;
  readonly proof_kicker: string;
  readonly proof_title: string;
  readonly feature_schema: string;
  readonly feature_locale: string;
  readonly feature_cldr: string;
  readonly feature_web: string;
  readonly sample_kicker: string;
  readonly sample_title: string;
  readonly reference_cta: string;
  readonly playground_kicker: string;
  readonly playground_title: string;
  readonly count_label: string;
  readonly fruit_label: string;
  readonly size_label: string;
  readonly gender_label: string;
  readonly amount_label: string;
  readonly date_label: string;
  readonly localized_path_label: string;
  readonly cookie_label: string;
  readonly route_label: string;
  readonly playground_sentence: (fruit: Fruit, size: Size, gender: Gender, count: number, amount: Money, date: ShortDate) => string;
  readonly cart_summary: (count: number, fruit: Fruit) => string;
  readonly number_format: (value: Measurement) => string;
  readonly currency_format: (amount: Money) => string;
  readonly date_format: (date: ShortDate) => string;
  readonly override_format: (amount: Money, date: ShortDate) => string;
  readonly gender_line: (gender: Gender) => string;
  readonly size_line: (size: Size) => string;
};

declare const lgl: {
  readonly main: typeof main;
};

export default lgl;
