import { sveltekit } from '@sveltejs/kit/vite';
import linguini from '@antepod/linguini-vite';
import { defineConfig } from 'vite';

export default defineConfig(({ command }) => ({
	plugins: [
		linguini({
			command: 'cargo',
			args: ['run', '--locked', '-p', 'linguini-cli', '--', 'build'],
			buildOnStart: command === 'serve'
		}),
		sveltekit()
	]
}));
