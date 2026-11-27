import { root } from "./locales/en.js";
import type { Date as SourceDate, globalThis as SourceGlobal, Input } from "./shared.js";
import type { LinguiniMessages } from "./messages.js";

const date = new Date("2026-01-02T00:00:00Z");
const input: Input = date;
root(input, date);
root(0, "2026-01-02");
root({ value: date, direct: date });
declare const messages: LinguiniMessages;
messages.root(date, date);
messages.root({ value: date, direct: 0 });
const source: SourceDate = "entry";
const namespace: SourceGlobal = "namespace";
void [source, namespace];
// @ts-expect-error Source Date retains its own finite enum values.
const wrongSource: SourceDate = "wrong";
// @ts-expect-error The source globalThis type remains independent of the native namespace.
const wrongNamespace: SourceGlobal = "wrong";
// @ts-expect-error The aliased input remains a native date, epoch number, or string.
root(true, date);
// @ts-expect-error The direct input retains the same native date domain.
root(date, true);
// @ts-expect-error Native Date inputs do not accept bigint.
root(1n, date);
// @ts-expect-error Date-shaped partial objects cannot substitute for native instances.
root({ getTime: () => 0 }, date);
// @ts-expect-error Both named arguments remain required.
root({ value: date });
// @ts-expect-error Unknown named arguments remain rejected.
root({ value: date, direct: date, extra: date });
