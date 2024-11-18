import { sveltekit } from '@sveltejs/kit/vite';
import houdini from 'houdini/vite';
import { defineConfig } from 'vite';
import Icons from 'unplugin-icons/vite';

export default defineConfig({
	plugins: [
		houdini(),
		sveltekit(),
		Icons({
			compiler: 'svelte'
		})
	],
	resolve: {
		alias: {
			$houdini: './$houdini'
		}
	}
});
