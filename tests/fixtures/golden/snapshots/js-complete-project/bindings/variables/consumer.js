// @ts-check
import { message } from "./standalone.js";

message(NaN);
message({ value: 2n });
// @ts-expect-error Source named argument keys remain exact.
message({ wrong: 2 });
// @ts-expect-error Numeric parameters reject booleans.
message(true);
