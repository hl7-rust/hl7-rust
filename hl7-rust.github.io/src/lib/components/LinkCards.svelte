<script lang="ts">
  /**
   * A grid of Lily cards, one per link. Used by every index page, so that the
   * documentation hub, a guide index, and the crate list all look and behave
   * the same way.
   */
  import { Card } from 'lily-design-system-svelte-headless';
  import type { NavLink } from '$lib/data/navigation';

  let {
    links,
    headingLevel = 3,
    label = undefined
  }: {
    links: NavLink[];
    headingLevel?: 2 | 3 | 4 | 5 | 6;
    label?: string;
  } = $props();
</script>

<div class="link-cards" aria-label={label}>
  {#each links as link (link.href)}
    <Card class="link-card" heading={link.label} href={link.href} {headingLevel}>
      {#if link.blurb}<p>{link.blurb}</p>{/if}
    </Card>
  {/each}
</div>

<style>
  .link-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
    gap: 1.25rem;
    margin: 1.5rem 0 2rem;
  }

  :global(.card.link-card) {
    padding: 1.25rem 1.375rem;
    border: 1px solid var(--color-base-300);
    border-radius: var(--radius-box, 0.5rem);
    background: var(--color-base-100);
  }

  :global(.card.link-card:hover) {
    border-color: var(--color-primary);
  }

  :global(.card.link-card h2),
  :global(.card.link-card h3),
  :global(.card.link-card h4) {
    margin: 0 0 0.5rem;
    font-size: 1.0625rem;
    overflow-wrap: anywhere;
  }

  :global(.card.link-card p) {
    margin: 0;
    font-size: 0.9375rem;
    opacity: 0.85;
  }
</style>
