import {browser} from '$app/environment';
import {loadPageMessages} from './message-payload.js';
import type {PageMessages} from './message-payload.js';
import type {SerializedLinguiniContext} from './types.js';

/** Prepare incoming messages without changing the page currently being rendered. */
export function createClientLoad<Locale extends string = string>() {
  return async <Data extends {linguini?: SerializedLinguiniContext<Locale>}>({data}: {data: Data}): Promise<Data & {linguiniMessages?: PageMessages}> => {
    const context = data.linguini;
    if (!browser || !context?.payload) return data;
    const messages = await loadPageMessages(context.payload.url);
    return {...data,linguiniMessages:messages};
  };
}
