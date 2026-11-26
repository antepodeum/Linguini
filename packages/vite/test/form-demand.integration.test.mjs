import {before, test} from 'node:test';
import {fileURLToPath} from 'node:url';
import {build} from 'vite';
import {compilePageBundles} from '../src/page-payloads.js';
import {verifyFormDemand} from '../test-support/form-demand.mjs';
import {buildNativeCompiler} from '../../../tests/support/native-vite.mjs';

const compiler = fileURLToPath(new URL(`../../../target/debug/linguini${process.platform === 'win32' ? '.exe' : ''}`, import.meta.url));
before(buildNativeCompiler);

test('native form demand produces exact page/locale payloads', () => verifyFormDemand({build,compilePageBundles,compiler}));
