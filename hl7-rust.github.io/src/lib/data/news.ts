/**
 * The news index.
 *
 * One entry per post, newest first. Each entry's `slug` is a directory under
 * `src/routes/news/`, holding the post itself — the prose lives in the Svelte
 * file, not here, so that a post can use the same components as every other
 * page. This file carries only what the index, the navigation, and the sitemap
 * need.
 *
 * `date` is ISO 8601 and is the publication date, not a last-edited date. A
 * post that needs a correction gets the correction stated in the post.
 */

export interface Post {
  /** URL slug under /news/. Also the directory name. */
  slug: string;
  title: string;
  /** ISO 8601, publication date. */
  date: string;
  /** One or two sentences, shown on the index and in the page description. */
  summary: string;
}

export const POSTS: Post[] = [
  {
    slug: 'what-this-project-claims',
    title: 'What this project claims, and how to check it',
    date: '2026-08-26',
    summary:
      'A precise conformance statement, published benchmarks with their method, and a plain description of what these crates do with patient data — three documents that answer the questions an evaluation actually asks.'
  }
];

/** Long form, for a post's byline. Deliberately not locale-dependent. */
export function formatDate(iso: string): string {
  const months = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ];
  const [year, month, day] = iso.split('-').map(Number);
  return `${day} ${months[month - 1]} ${year}`;
}

export function postBySlug(slug: string): Post {
  const post = POSTS.find((candidate) => candidate.slug === slug);
  if (!post) throw new Error(`no post named ${slug}`);
  return post;
}
