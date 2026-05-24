import adapter from '@sveltejs/adapter-static';

const basePath = normalizeBasePath(process.env.BASE_PATH ?? '');

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
		// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
		runes: ({ filename }) => (filename.split(/[/\\]/).includes('node_modules') ? undefined : true)
	},
	kit: {
		adapter: adapter(),
		paths: {
			base: basePath
		}
	}
};

function normalizeBasePath(value) {
	const trimmed = value.trim();
	if (!trimmed || trimmed === '/') return '';
	return `/${trimmed.replace(/^\/+|\/+$/g, '')}`;
}

export default config;
