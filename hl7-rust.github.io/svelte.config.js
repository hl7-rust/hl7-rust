import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
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
  }
};

export default config;
