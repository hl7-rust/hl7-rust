<script lang="ts">
  /**
   * Share this page — thin site wiring around the published
   * lily-design-system-svelte-share-picker package. Uses the native share
   * sheet where the platform has one, and this fixed target list otherwise.
   *
   * `title` reads `page.data.title` — the same route-load value DocPage
   * turns into <svelte:head><title>, run through the same siteTitle()
   * suffix rule — so a shared link's title always matches the page
   * actually open, including across client-side navigation (page.data is
   * SvelteKit's own reactive store; there is no DOM-race the way reading
   * document.title directly here would have). `url` is left to the
   * package's own default (the current page's URL).
   *
   * Targets: LinkedIn, Mastodon, Bluesky, Reddit — mirroring the channels
   * help/outreach/index.md names as real for this audience (the Rust
   * community, healthcare-integration practitioners) rather than a
   * generic, unreviewed default list.
   */
  import { page } from '$app/state';
  import { SharePicker, type ShareTarget } from 'lily-design-system-svelte-share-picker';
  import { siteTitle } from '$lib/data/site-title';

  const title = $derived(siteTitle(page.url.pathname, page.data.title));

  const targets: ShareTarget[] = [
    {
      id: 'linkedin',
      label: 'LinkedIn',
      href: (url) => `https://www.linkedin.com/sharing/share-offsite/?url=${encodeURIComponent(url)}`
    },
    {
      id: 'mastodon',
      label: 'Mastodon',
      href: (url, title) =>
        `https://mastodon.social/share?text=${encodeURIComponent(title)}%20${encodeURIComponent(url)}`
    },
    {
      id: 'bluesky',
      label: 'Bluesky',
      href: (url, title) =>
        `https://bsky.app/intent/compose?text=${encodeURIComponent(`${title} ${url}`)}`
    },
    {
      id: 'reddit',
      label: 'Reddit',
      href: (url, title) =>
        `https://www.reddit.com/submit?url=${encodeURIComponent(url)}&title=${encodeURIComponent(title)}`
    }
  ];
</script>

<SharePicker
  label="Share this page"
  {title}
  {targets}
  copyLabel="Copy link"
  copiedLabel="Link copied"
  copyFailedLabel="Could not copy — copy it from the address bar"
/>
