import { base } from '$app/paths';
import { createWebLocaleI18n, pathLocale, cookieLocale, requestLanguage } from '@antepodeum/linguini-web';
import * as locale from '$lib/generated/linguini/locale.js';
import { linguiniPolicy } from '$lib/linguini-policy.js';

export const web = createWebLocaleI18n(locale, {
  ...linguiniPolicy,
  locale: {
    ...linguiniPolicy.locale,
    sources: [pathLocale(), cookieLocale(), requestLanguage()],
  },
}, { base });
