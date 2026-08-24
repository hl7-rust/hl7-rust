import { ALL_PAGES } from '$lib/data/navigation';

/**
 * The sitemap.
 *
 * Prerendered like everything else, and generated from the same navigation
 * data the header, the sidebar, and the previous/next links read — so a page
 * added there appears here without a second edit. It is not linked from any
 * page, so svelte.config.js lists it as a prerender entry.
 */
export const prerender = true;

const SITE = 'https://hl7-rust.github.io';

export function GET(): Response {
  const urls = ALL_PAGES.map((page) => {
    // The home page is the entry point; section indexes outrank their children.
    const priority = page.href === '/' ? '1.0' : page.href.split('/').length <= 3 ? '0.8' : '0.6';
    return `  <url>\n    <loc>${SITE}${page.href}</loc>\n    <priority>${priority}</priority>\n  </url>`;
  }).join('\n');

  const body = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urls}
</urlset>
`;

  return new Response(body, {
    headers: { 'content-type': 'application/xml; charset=utf-8' }
  });
}
