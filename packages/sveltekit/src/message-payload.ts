import type {PagePayload} from './types.js';

export type PageMessages = Readonly<Record<number, unknown>>;

export function pagePayloadUrl(payload: PagePayload, locale: string): string {
  return payload.url.slice(0,payload.url.lastIndexOf('/') + 1) + encodeURIComponent(locale) + '.js';
}

export async function loadPageMessages(url: string): Promise<PageMessages> {
  const module = await import(/* @vite-ignore */ url);
  const messages: unknown = module.messages;
  if (!messages || typeof messages !== 'object') throw new TypeError(`Invalid Linguini page payload ${url}`);
  return messages as PageMessages;
}
