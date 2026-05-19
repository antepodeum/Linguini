import { sveltekit } from '@sveltejs/kit/vite';
import linguini from '@antepod/linguini-vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		linguini({
			command: 'cargo',
			args: ['run', '-p', 'linguini-cli', '--', 'build']
		}),
		sveltekit()
	]
});
