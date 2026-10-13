import { createLinguini as js } from "./plain/javascript/index.js";
import { createLinguini as ts } from "./plain/typescript/index";
import { createLinguini as sourceJs } from "./source-only/javascript/index.js";
/** @type {string} */
const greeting = js("fr").hello("Name");
sourceJs("fr").hello({ name: "Name" });
ts("en").section.title;
// @ts-expect-error Required parameter.
js("fr").hello();
// @ts-expect-error Named calls reject unknown properties.
sourceJs("fr").hello({ name: "Name", extra: true });
// @ts-expect-error Source-only JavaScript retains required positional types.
sourceJs("fr").hello(3);
// @ts-expect-error Locale names remain finite.
js("de");
void greeting;

/** @param {import("./unsafe/messages.js").LinguiniMessages} messages */
function checkHostileKeys(messages) {
  messages.top["bad*/key"].leaf({ ["name*/key"]: "value" });
  // @ts-expect-error Exact hostile property identity and value type are retained.
  messages.top["bad*/key"].leaf({ ["name*/key"]: 3 });
  // @ts-expect-error Escaping must not insert spaces into source keys.
  messages.top["bad* /key"].leaf("value");
}
void checkHostileKeys;
