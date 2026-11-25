/** Messages prepared on every import path to a boundary, including cycles. */
export function withPreparedMessages(initial,boundaries) {
  const all = new Set([...initial.messages,...boundaries.flatMap(boundary => boundary.messages)]);
  const root = {available:new Set(),messages:initial.messages};
  const states = new Map(boundaries.map(boundary => [boundary.id,{
    boundary,messages:boundary.messages,parents:[],available:new Set(all),
  }]));
  for (const id of initial.deferred) states.get(id)?.parents.push(root);
  for (const state of states.values()) {
    for (const id of state.boundary.deferred) states.get(id)?.parents.push(state);
  }
  const reachable = new Set();
  const pending = [...initial.deferred];
  while (pending.length) {
    const id = pending.pop();
    if (reachable.has(id) || !states.has(id)) continue;
    reachable.add(id);
    pending.push(...states.get(id).boundary.deferred);
  }
  if (reachable.size !== states.size) throw new Error('Linguini cannot determine the parent paths of every deferred boundary');
  // Start from the complete set and remove facts until all incoming paths agree.
  // Root paths anchor cycles, so future messages cannot justify their own preparation.
  let changed;
  do {
    changed = false;
    for (const state of states.values()) {
      const [first,...rest] = state.parents;
      let available = new Set([...first.available,...first.messages]);
      for (const parent of rest) {
        const supplied = new Set([...parent.available,...parent.messages]);
        available = new Set([...available].filter(name => supplied.has(name)));
      }
      if (available.size !== state.available.size || [...available].some(name => !state.available.has(name))) {
        state.available = available;
        changed = true;
      }
    }
  } while (changed);
  return boundaries.map(boundary => ({...boundary,available:[...states.get(boundary.id).available].sort()}));
}
