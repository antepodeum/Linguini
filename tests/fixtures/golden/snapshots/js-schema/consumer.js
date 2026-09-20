import { normalizeMessageArgs, selectBranch } from "./shared.js";

/** @type {import("./shared.js").Tone} */
const tone = "warm";
/** @type {import("./shared.js").Amount} */
const amount = 42n;
/** @type {import("./messages.js").LinguiniMessages} */
const messages = /** @type {any} */ ({});

const selected = selectBranch(tone, { warm: amount, cool: 0 });
const normalized = normalizeMessageArgs([{ tone }], ["tone"]);
messages.hello(tone);
messages.hello({ tone: "cool" });

/** @type {import("./shared.js").Tone} */
// @ts-expect-error schema enum variants stay closed in checked JavaScript
const invalidTone = "cold";
// @ts-expect-error named message arguments preserve the schema enum
messages.hello({ tone: "cold" });
// @ts-expect-error positional message arguments preserve the schema enum
messages.hello("cold");

void [selected, normalized, invalidTone];
