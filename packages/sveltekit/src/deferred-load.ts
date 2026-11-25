import {browser} from '$app/environment';
import {loadPageMessages} from './message-payload.js';
import type {PageMessages} from './message-payload.js';
import type {SerializedLinguiniContext} from './types.js';

type PagePreparation = <Locale extends string>(context:SerializedLinguiniContext<Locale>,route:string|null|undefined) => Promise<SerializedLinguiniContext<Locale>>;
let preparation: PagePreparation | undefined;

/** Only owning page nodes register deferred navigation preparation. */
export function registerPagePreparation(callback: PagePreparation): void {
  preparation = callback;
}

export function createClientLoad<Locale extends string = string>() {
  return async <Data extends {linguini?: SerializedLinguiniContext<Locale>}>(event:{data:Data;route?:{id:string|null}}): Promise<Data & {linguiniMessages?:PageMessages}> => {
    const {data} = event;
    const context = data.linguini;
    if (!browser || !context) return data;
    const incoming = preparation?.(context,event.route?.id);
    if (!context.payload) return incoming ? {...data,linguini:await incoming} : data;
    const loading = loadPageMessages(context.payload.url);
    if (!incoming) return {...data,linguiniMessages:await loading};
    const [messages,prepared] = await Promise.all([loading,incoming]);
    return {...data,linguini:prepared,linguiniMessages:messages};
  };
}
