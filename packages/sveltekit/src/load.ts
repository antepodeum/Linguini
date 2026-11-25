import {browser} from '$app/environment';
import {loadPageMessages} from './message-payload.js';
import type {PageMessages} from './message-payload.js';
import type {SerializedLinguiniContext} from './types.js';

type PagePreparation = <Locale extends string>(context:SerializedLinguiniContext<Locale>,route:string|null|undefined) => Promise<SerializedLinguiniContext<Locale>>;
let preparation: PagePreparation | undefined;

/** The build registers this capability only for pages exposing deferred messages. */
export function registerPagePreparation(callback: PagePreparation): void {
  preparation = callback;
}

/** Prepare incoming messages without changing the page currently being rendered. */
export function createClientLoad<Locale extends string = string>() {
  return async <Data extends {linguini?: SerializedLinguiniContext<Locale>}>(event: {data:Data;route?:{id:string|null}}): Promise<Data & {linguiniMessages?: PageMessages}> => {
    const {data} = event;
    const context = data.linguini;
    if (!browser || !context) return data;
    const incoming = preparation?.(context,event.route?.id);
    if (!context.payload) {
      if (incoming) return {...data,linguini:await incoming};
      return data;
    }
    const loading = loadPageMessages(context.payload.url);
    if (incoming) {
      const [messages,prepared] = await Promise.all([loading,incoming]);
      return {...data,linguini:prepared,linguiniMessages:messages};
    }
    return {...data,linguiniMessages:await loading};
  };
}
