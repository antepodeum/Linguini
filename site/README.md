# Linguini site

Run the site from the repository root so it uses the workspace-local CLI and Vite plugin:

```bash
pnpm dev:site --host 127.0.0.1
```

Open `http://127.0.0.1:5173/`. Changes under `site/linguini/` rebuild generated modules and
flow through Vite HMR. Stop the server with `Ctrl-C`.

Run the complete local site gate from the repository root:

```bash
pnpm test:site
```

That command regenerates Linguini output, runs runtime tests and Svelte checks, builds the
production site, and validates its bundle graph.
