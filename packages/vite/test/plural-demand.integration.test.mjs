import {before,test} from "node:test";
import {fileURLToPath} from "node:url";
import {build} from "vite";
import {linguini} from "../src/index.js";
import {validatePluralRuntimes} from "../src/plural-payloads.js";
import {verifyPluralDemand} from "../test-support/plural-demand.mjs";
import {buildNativeCompiler} from "../../../tests/support/native-vite.mjs";

const compiler = fileURLToPath(new URL(`../../../target/debug/linguini${process.platform === "win32" ? ".exe" : ""}`,import.meta.url));
before(buildNativeCompiler);
test("native page payloads specialize shared plural demand through sparse fallback and lazy delivery",
  () => verifyPluralDemand({build,linguini,validatePluralRuntimes,compiler}));
