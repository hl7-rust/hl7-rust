<script lang="ts">
  /** The "read next" strip at the foot of a crate page. */
  import LinkCards from './LinkCards.svelte';
  import { crateBySlug } from '$lib/data/crates';

  let { slugs }: { slugs: string[] } = $props();

  const links = $derived(
    slugs.map((slug) => {
      const crate = crateBySlug(slug);
      return { href: `/crates/${crate.slug}/`, label: crate.name, blurb: crate.tagline };
    })
  );
</script>

<h2 id="related">Related crates</h2>
<LinkCards {links} label="Related crates" />
