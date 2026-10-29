import { page } from '$app/state';
import type { PagePayload } from './types.js';

import {loadPageMessages,pagePayloadUrl} from './message-payload.js';
import type {PageMessages} from './message-payload.js';
const prepared = new Map<string, PageMessages>();
const pending = new Map<string, Promise<PageMessages>>();

export async function preparePageMessages(payload: PagePayload, locale: string): Promise<PageMessages> {
  const url = pagePayloadUrl(payload, locale);
  const previous = prepared.get(url);
  if (previous) return previous;
  const loading = pending.get(url);
  if (loading) return loading;
  const task = loadPageMessages(url).then(messages => {
    prepared.set(url, messages);
    return messages;
  }).finally(() => pending.delete(url));
  pending.set(url, task);
  return task;
}

export async function prepareCurrentPageMessages(locale: string): Promise<void> {
  const data = page.data as { linguini?: { payload?: PagePayload } };
  if (data.linguini?.payload) await preparePageMessages(data.linguini.payload, locale);
}

export function getPageMessage(canonical: number, locale: string): unknown {
  const data = page.data as { linguini?: { locale: string; payload?: PagePayload }; linguiniMessages?: PageMessages };
  const payload = data.linguini?.payload;
  const messages = locale === data.linguini?.locale
    ? data.linguiniMessages
    : payload && prepared.get(pagePayloadUrl(payload, locale));
  if (!messages || !Object.hasOwn(messages, canonical)) {
    throw new Error(`Linguini page message ${canonical} for ${locale} was not prepared`);
  }
  return messages[canonical];
}

