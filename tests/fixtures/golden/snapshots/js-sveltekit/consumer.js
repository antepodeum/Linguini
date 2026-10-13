import { handle, reroute, load } from "./all/sveltekit.js";
import { linguiniHandle, linguiniReroute, linguiniLoad } from "./path/sveltekit-control.js";
/** @type {import("@sveltejs/kit").Handle} */
const checkedHandle = handle;
/** @type {import("@sveltejs/kit").Reroute} */
const checkedReroute = reroute;
/** @type {import("@sveltejs/kit").ServerLoad} */
const checkedLoad = load;
// @ts-expect-error Handle requires a resolve function.
handle({ event: {} });
// @ts-expect-error Reroute receives a URL object.
reroute({ url: "https://example.test" });
// @ts-expect-error Load requires a complete request event.
load({ locals: {} });
void [checkedHandle, checkedReroute, checkedLoad, linguiniHandle, linguiniReroute, linguiniLoad];
