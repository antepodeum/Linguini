import { sveltekit } from '@sveltejs/kit/vite';
import linguini from '@antepodeum/linguini-sveltekit/vite';
import { defineConfig } from 'vite';
import { compilerCommand, compilerArgs } from './scripts/compiler.mjs';

export default defineConfig(({ command }) => ({
	plugins: [
		linguini({
			command: compilerCommand,
			args: compilerArgs(['build']),
			buildOnStart: command === 'serve'
		}),
		sveltekit()
	]
}));
