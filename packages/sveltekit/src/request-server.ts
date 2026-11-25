import {getRequestEvent} from '$app/server';

export function getRequestLocale(): unknown {
  return (getRequestEvent().locals as {linguini?:{locale?:unknown}}).linguini?.locale;
}
