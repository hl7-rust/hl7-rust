import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    sveltekit({
      preprocess: vitePreprocess(),
      adapter: adapter({
        pages: 'build',
        assets: 'build',
        fallback: '404.html',
        strict: true
      }),

      prerender: {
        // Every page is reachable by crawling from "/", except the sitemap,
        // which nothing links to on purpose.
        entries: ['*', '/sitemap.xml']
      }
    })
  ]
});
