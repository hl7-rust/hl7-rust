// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    interface PageData {
      /**
       * The page's own bare title (no site-name suffix) — every route's
       * `+page.ts` sets this via `load`, so it is the single source of
       * truth `DocPage` reads for `<title>`, the h1, and the breadcrumb,
       * and that `ShareControl` in the root layout reads to share the
       * page actually open, not a generic site title. See
       * `$lib/data/site-title.ts` for how it becomes the full `<title>`.
       */
      title: string;
    }
    // interface PageState {}
    // interface Platform {}
  }
}

export {};
