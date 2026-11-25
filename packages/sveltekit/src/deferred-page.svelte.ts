import {page} from '$app/state';
import {getPageMessage,preparePageMessages} from './page.svelte.js';
import type {PageMessages} from './message-payload.js';

interface RequestContext {locale:string}
interface DeferredDirectory {ready:boolean}
interface DeferredRequest {
  active:Map<string,DeferredDirectory>;
  messages:Map<string,Record<string,unknown>>;
}
const deferred = new WeakMap<RequestContext,DeferredRequest>();

function emptyRequest(): DeferredRequest {
  return {active:new Map(),messages:new Map()};
}

function requestContext(): RequestContext | undefined {
  return (page.data as {linguini?:RequestContext}).linguini;
}

/** Only the build-linked capability module registers this loader. */
export async function prepareDeferredCurrentPageMessages(locale: string): Promise<void> {
  const request = requestContext();
  const state = request && deferred.get(request);
  if (!request || !state) return;
  await Promise.all([...state.active].map(([directory,entry]) => prepareDirectory(request,state,directory,entry,locale)));
}

export async function prepareDeferredPageMessages(routes: Readonly<Record<string,string>>,getLocale: () => string): Promise<void> {
  const request = requestContext();
  const route = page.route.id;
  const directory = route && Object.hasOwn(routes,route) ? routes[route] : undefined;
  if (!request || !directory) throw abortedDeferred();
  let state = deferred.get(request);
  if (!state) {
    state = emptyRequest();
    deferred.set(request,state);
  }
  const entry = state.active.get(directory) ?? {ready:false};
  state.active.set(directory,entry);
  try {
    for (;;) {
      const locale = getLocale();
      await prepareDirectory(request,state,directory,entry,locale);
      if (requestContext() !== request || page.route.id !== route) throw abortedDeferred();
      if (getLocale() === locale) return;
    }
  } catch (error) {
    if (!entry.ready) state.active.delete(directory);
    throw error;
  }
}

async function prepareDirectory(request: RequestContext,state: DeferredRequest,directory: string,entry: DeferredDirectory,locale: string): Promise<void> {
  const messages = await directoryMessages(directory,locale);
  if (requestContext() !== request) return;
  storeDirectory(state,directory,entry,locale,messages);
}

function directoryMessages(directory:string,locale:string): Promise<PageMessages> {
  return preparePageMessages({url:directory + encodeURIComponent(locale) + '.js'},locale);
}

function storeDirectory(state:DeferredRequest,directory:string,entry:DeferredDirectory,locale:string,messages:PageMessages): void {
  let values = state.messages.get(locale);
  if (!values) {
    values = Object.create(null) as Record<string,unknown>;
    state.messages.set(locale,values);
  }
  const source = messages as Readonly<Record<string,unknown>>;
  for (const key of Object.keys(messages)) {
    if (!Object.hasOwn(values,key)) values[key] = source[key];
  }
  entry.ready = true;
  state.active.set(directory,entry);
}

/** Prepare a fresh metadata identity before Kit publishes a reused page component. */
export async function prepareDeferredNavigation<Context extends RequestContext>(incoming:Context,route:string|null|undefined): Promise<Context> {
  const next = {...incoming};
  const current = requestContext();
  const previous = current && route === page.route.id ? deferred.get(current) : undefined;
  if (!previous?.active.size) return next;
  const state = emptyRequest();
  deferred.set(next,state);
  await Promise.all([...previous.active.keys()].map(async directory => {
    const messages = await directoryMessages(directory,incoming.locale);
    storeDirectory(state,directory,{ready:true},incoming.locale,messages);
  }));
  return next;
}

function abortedDeferred(): Error {
  const error = new Error('Linguini deferred import no longer belongs to the active page');
  error.name = 'AbortError';
  return error;
}

export function getDeferredPageMessage(canonical: number,locale: string): unknown {
  const request = requestContext();
  const additional = request && deferred.get(request)?.messages.get(locale);
  if (additional && Object.hasOwn(additional,canonical)) return additional[canonical];
  return getPageMessage(canonical,locale);
}
