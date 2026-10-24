import { base } from '$app/paths';
import { createWebLocaleI18n } from '@antepodeum/linguini-web';
import * as locale from '$lib/generated/linguini/locale.js';
import { linguiniPolicy } from './linguini-policy.js';

export const web = createWebLocaleI18n(locale, linguiniPolicy, { base });
