import { createLinguini } from "./plain/javascript/index.js";
import type { Linguini, Locale } from "./plain/javascript/index.js";

const locale: Locale = "fr";
const messages: Linguini = createLinguini(locale);
const greeting: string = messages.hello({ name: "Name" });
const title: string = messages.section.title;
messages.hello("Name");
// @ts-expect-error Required parameter.
messages.hello();
// @ts-expect-error Exact named parameter type.
messages.hello({ name: 42 });
// @ts-expect-error Unknown named properties.
messages.hello({ name: "Name", extra: true });
// @ts-expect-error Locale names remain finite.
createLinguini("de");
void [greeting, title];
