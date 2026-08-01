import type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../../shared";

export type { __lgl_name_6D61696E2E4672756974, __lgl_name_6D61696E2E53697A65, __lgl_name_6D61696E2E4D6F6E6579, __lgl_name_6D61696E2E53686F727444617465, __lgl_name_6D61696E2E4D6561737572656D656E74 } from "../../shared";

export type Fruit = __lgl_name_6D61696E2E4672756974;

export type Size = __lgl_name_6D61696E2E53697A65;

export type Money = __lgl_name_6D61696E2E4D6F6E6579;

export type ShortDate = __lgl_name_6D61696E2E53686F727444617465;

export type Measurement = __lgl_name_6D61696E2E4D6561737572656D656E74;

export declare const main: {
  readonly codegen: {
    readonly kicker: string;
    readonly title: string;
    readonly intro: string;
    readonly ts_title: string;
    readonly ts_desc: string;
    readonly svelte_title: string;
    readonly svelte_desc: string;
    readonly planned_title: string;
    readonly planned_intro: string;
    readonly rust: string;
    readonly kotlin: string;
    readonly swift: string;
    readonly go: string;
    readonly python: string;
    readonly csharp: string;
    readonly status_shipped: string;
    readonly status_planned: string;
  };
  readonly hero: {
    readonly eyebrow: string;
    readonly title: string;
    readonly tagline: string;
    readonly copy: string;
    readonly term: string;
    readonly term_kind: string;
    readonly term_hint: string;
    readonly intro: string;
    readonly trait_typed: string;
    readonly trait_compiled: string;
    readonly trait_native: string;
    readonly primary_cta: string;
    readonly secondary_cta: string;
  };
  readonly nav: {
    readonly why: string;
    readonly language: string;
    readonly codegen: string;
    readonly web: string;
    readonly locale_label: string;
  };
  readonly playground: {
    readonly kicker: string;
    readonly title: string;
    readonly count_label: string;
    readonly fruit_label: string;
    readonly fruit_apple_label: string;
    readonly fruit_pear_label: string;
    readonly fruit_orange_label: string;
    readonly size_label: string;
    readonly size_small_label: string;
    readonly size_big_label: string;
    readonly amount_label: string;
    readonly date_label: string;
    readonly localized_path_label: string;
    readonly cookie_label: string;
    readonly route_label: string;
    readonly sentence: (fruit: __lgl_name_6D61696E2E4672756974, size: __lgl_name_6D61696E2E53697A65, count: number | bigint | string, amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => string;
    readonly cart_summary: (count: number | bigint | string, fruit: __lgl_name_6D61696E2E4672756974) => string;
    readonly number_format: (value: __lgl_name_6D61696E2E4D6561737572656D656E74) => string;
    readonly currency_format: (amount: __lgl_name_6D61696E2E4D6F6E6579) => string;
    readonly date_format: (date: __lgl_name_6D61696E2E53686F727444617465) => string;
    readonly override_format: (amount: __lgl_name_6D61696E2E4D6F6E6579, date: __lgl_name_6D61696E2E53686F727444617465) => string;
    readonly size_line: (size: __lgl_name_6D61696E2E53697A65) => string;
  };
  readonly web: {
    readonly kicker: string;
    readonly title: string;
    readonly intro: string;
    readonly routing: string;
    readonly cookie: string;
    readonly fallback: string;
    readonly reactivity: string;
    readonly links: string;
  };
};

declare const lgl: {
  readonly main: typeof main;
};

export default lgl;
