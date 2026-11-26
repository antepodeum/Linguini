// Deliberately not a Linguini executable: explicit launchers are validated by the LSP handshake.
const { spawn } = require('node:child_process');
const child = spawn(process.argv[2], ['lsp'], { stdio: 'inherit' });
child.on('error', (error) => { console.error(error); process.exit(1); });
child.on('exit', (code) => process.exit(code ?? 1));
