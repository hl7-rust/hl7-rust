<script lang="ts">
  /**
   * A note or a warning beside the main prose.
   *
   * `note` is supplementary — read it or don't. `warning` is the kind of thing
   * that costs a clinical message if you skip it, so it gets role="alert" via
   * Lily's WarningCallout.
   */
  import type { Snippet } from 'svelte';
  import { InformationCallout, WarningCallout } from 'lily-design-system-svelte-headless';

  let {
    type = 'note',
    heading,
    children
  }: {
    type?: 'note' | 'warning';
    heading: string;
    children: Snippet;
  } = $props();
</script>

{#if type === 'warning'}
  <WarningCallout class="doc-callout" label={heading}>
    <p class="doc-callout-heading">{heading}</p>
    {@render children()}
  </WarningCallout>
{:else}
  <InformationCallout class="doc-callout" label={heading}>
    <p class="doc-callout-heading">{heading}</p>
    {@render children()}
  </InformationCallout>
{/if}

<style>
  :global(.doc-callout) {
    margin: 1.75rem 0;
    padding: 1rem 1.25rem;
    border-radius: var(--radius-box, 0.5rem);
    border-left: 4px solid var(--color-info);
    background: var(--color-base-200);
  }

  :global(.warning-callout.doc-callout) {
    border-left-color: var(--color-warning);
  }

  .doc-callout-heading {
    margin: 0 0 0.375rem;
    font-weight: 700;
  }

  :global(.doc-callout p:last-child) {
    margin-bottom: 0;
  }
</style>
