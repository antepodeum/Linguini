import { sveltekit } from '@sveltejs/kit/vite';
import linguini from '@antepodeum/linguini-sveltekit/vite';
import { defineConfig } from 'vite';

export default defineConfig(({ command }) => ({
	plugins: [
		linguini({
			command: 'node',
			args: ['scripts/restore-linguini-output.mjs'],
			buildOnStart: command === 'serve'
		}),
		sveltekit()
	]
}));
