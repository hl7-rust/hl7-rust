/**
 * The full document title for a page, given its bare `page.data.title` and
 * the current path.
 *
 * The home page's title is already the complete string it wants in
 * `<title>` (see `src/routes/+page.ts`); every other page's is suffixed
 * with the site name. One function, used by both `DocPage` (for
 * `<svelte:head><title>`) and `ShareControl` (for the text handed to a
 * share target), so the two can never say something different about the
 * same page.
 */
export function siteTitle(pathname: string, title: string): string {
  return pathname === '/' ? title : `${title} — HL7 Rust`;
}
