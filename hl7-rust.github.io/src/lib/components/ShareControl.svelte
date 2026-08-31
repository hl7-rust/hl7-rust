<script lang="ts">
  /**
   * Share this page — thin site wiring around the published
   * lily-design-system-svelte-share-picker package. Uses the native share
   * sheet where the platform has one, and this fixed target list otherwise.
   *
   * `url` and (where a destination's href ignores it) `title` are left to
   * the package's own defaults — the current page's URL — rather than
   * threaded through per page. The generic site name is passed as `title`
   * instead of the current page's title, deliberately: reading the live
   * per-page <title> reliably here would mean racing this layout's render
   * against each page's own <svelte:head> during client-side navigation,
   * and the URL alone already identifies which page is being shared.
   *
   * Targets mirror the channels help/outreach/index.md names as real for
   * this audience (the Rust community, healthcare-integration
   * practitioners) rather than a generic, unreviewed default list.
   */
  import { SharePicker, type ShareTarget } from 'lily-design-system-svelte-share-picker';

  const targets: ShareTarget[] = [
    {
      id: 'mastodon',
      label: 'Mastodon',
      href: (url, title) =>
        `https://mastodon.social/share?text=${encodeURIComponent(title)}%20${encodeURIComponent(url)}`
    },
    {
      id: 'linkedin',
      label: 'LinkedIn',
      href: (url) => `https://www.linkedin.com/sharing/share-offsite/?url=${encodeURIComponent(url)}`
    },
    {
      id: 'reddit',
      label: 'Reddit',
      href: (url, title) =>
        `https://www.reddit.com/submit?url=${encodeURIComponent(url)}&title=${encodeURIComponent(title)}`
    },
    {
      id: 'email',
      label: 'Email',
      href: (url, title) => `mailto:?subject=${encodeURIComponent(title)}&body=${encodeURIComponent(url)}`,
      newTab: false
    }
  ];
</script>

<SharePicker
  label="Share this page"
  title="HL7 Rust"
  {targets}
  copyLabel="Copy link"
  copiedLabel="Link copied"
  copyFailedLabel="Could not copy — copy it from the address bar"
/>
