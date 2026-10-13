import type { LinguiniWebLocale } from "../web";

export declare function startRuntimeLinkLocalization<Locale extends string>(
  web: LinguiniWebLocale<Locale>,
  getLocale: () => Locale,
): { refresh(): void; destroy(): void } | undefined;
