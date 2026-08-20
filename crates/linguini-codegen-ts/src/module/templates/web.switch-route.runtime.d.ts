import type { LinguiniWebLocale } from "../web";

export declare function handleLocaleSwitchRoute<Locale extends string>(
  web: LinguiniWebLocale<Locale>,
  event: { url: URL; request: Request },
): Response | undefined;
