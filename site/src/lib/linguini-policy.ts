import type { LinguiniWebOptions } from '@antepodeum/linguini-web';

export const linguiniPolicy = {
  routing: { localePrefix: 'always', canonical: 'redirect' },
  locale: { switch: { writesPath: true, writesCookie: true, writesLocalStorage: true } },
  cookie: {
    name: 'LINGUINI_SITE_LOCALE', path: '/', maxAge: 365 * 24 * 60 * 60,
    sameSite: 'lax', secure: false, httpOnly: false,
  },
  localStorage: { key: 'LINGUINI_SITE_LOCALE' },
  links: { mode: 'runtime' },
  routes: { exclude: ['/_app/**', '/favicon.ico', '/robots.txt'] },
} satisfies LinguiniWebOptions;
