<script lang="ts">
  /**
   * The shell every documentation, guide, tutorial, crate, and help page uses.
   *
   * It owns the parts that must not drift page to page: the breadcrumb trail,
   * the h1 and its lede, the optional "on this page" list, and the
   * previous/next pair. Page files supply only the prose.
   */
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';
  import { BreadcrumbList, BreadcrumbListItem, BreadcrumbNav, ContentsList, ContentsListItem, ContentsNav } from 'lily-design-system-svelte-headless';
  import { SECTIONS, neighborsFor } from '$lib/data/navigation';

  export interface TocEntry {
    /** The id of the h2 this links to. */
    id: string;
    label: string;
  }

  let {
    title,
    lede = undefined,
    eyebrow = undefined,
    contents = [],
    children,
    aside = undefined
  }: {
    title: string;
    /** One or two sentences under the h1. Plain text. */
    lede?: string;
    /** Overrides the section name shown above the h1. */
    eyebrow?: string;
    /** One entry per h2 in the page, in document order. */
    contents?: TocEntry[];
    children: Snippet;
    /** Extra material shown beside the contents list. */
    aside?: Snippet;
  } = $props();

  const pathname = $derived(page.url.pathname);
  const section = $derived(SECTIONS.find((candidate) => pathname.startsWith(candidate.href)));
  const isSectionIndex = $derived(section?.href === pathname);
  const neighbors = $derived(neighborsFor(pathname));
  const documentTitle = $derived(pathname === '/' ? title : `${title} — HL7 Rust`);
</script>

<svelte:head>
  <title>{documentTitle}</title>
  {#if lede}<meta name="description" content={lede} />{/if}
</svelte:head>

<article class="doc">
  <BreadcrumbNav label="Breadcrumb">
    <BreadcrumbList>
      <BreadcrumbListItem><a href="/">Home</a></BreadcrumbListItem>
      {#if section}
        <BreadcrumbListItem current={isSectionIndex}>
          {#if isSectionIndex}
            {section.title}
          {:else}
            <a href={section.href}>{section.title}</a>
          {/if}
        </BreadcrumbListItem>
      {/if}
      {#if section && !isSectionIndex}
        <BreadcrumbListItem current>{title}</BreadcrumbListItem>
      {/if}
    </BreadcrumbList>
  </BreadcrumbNav>

  <header class="doc-header">
    {#if eyebrow ?? section?.title}
      <p class="doc-eyebrow">{eyebrow ?? section?.title}</p>
    {/if}
    <h1>{title}</h1>
    {#if lede}<p class="doc-lede">{lede}</p>{/if}
  </header>

  {#if contents.length > 0 || aside}
    <div class="doc-contents">
      {#if contents.length > 0}
        <ContentsNav label="On this page">
          <p class="doc-contents-heading">On this page</p>
          <ContentsList>
            {#each contents as entry (entry.id)}
              <ContentsListItem><a href={`#${entry.id}`}>{entry.label}</a></ContentsListItem>
            {/each}
          </ContentsList>
        </ContentsNav>
      {/if}
      {#if aside}{@render aside()}{/if}
    </div>
  {/if}

  <div class="doc-body">
    {@render children()}
  </div>

  {#if neighbors.previous || neighbors.next}
    <nav class="doc-neighbors" aria-label="Previous and next page">
      {#if neighbors.previous}
        <a class="doc-neighbor" href={neighbors.previous.href} rel="prev">
          <span class="doc-neighbor-role">Previous</span>
          <span class="doc-neighbor-label">{neighbors.previous.label}</span>
        </a>
      {:else}
        <span></span>
      {/if}
      {#if neighbors.next}
        <a class="doc-neighbor doc-neighbor-next" href={neighbors.next.href} rel="next">
          <span class="doc-neighbor-role">Next</span>
          <span class="doc-neighbor-label">{neighbors.next.label}</span>
        </a>
      {/if}
    </nav>
  {/if}
</article>

<style>
  .doc {
    max-width: 52rem;
  }

  .doc-header {
    margin-bottom: 0.5rem;
  }

  .doc-eyebrow {
    margin: 0 0 0.25rem;
    font-size: 0.8125rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    opacity: 0.7;
  }

  .doc-header h1 {
    margin: 0 0 0.75rem;
  }

  .doc-lede {
    margin: 0;
    font-size: 1.125rem;
    line-height: 1.6;
    opacity: 0.9;
  }

  .doc-contents {
    display: grid;
    gap: 1rem;
    margin: 2rem 0 0;
    max-width: 32rem;
  }

  .doc-contents-heading {
    margin: 0 0 0.5rem;
    font-weight: 700;
    font-size: 0.875rem;
  }

  :global(.doc .contents-nav) {
    padding: 0.875rem 1.125rem;
    border-left: 3px solid var(--color-primary);
    background: var(--color-base-200);
    border-radius: var(--radius-box, 0.5rem);
  }

  :global(.doc .contents-list) {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    gap: 0.125rem;
    font-size: 0.9375rem;
  }

  :global(.doc .contents-list a) {
    display: block;
    padding: 0.125rem 0;
  }

  .doc-neighbors {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    margin-top: 3.5rem;
    padding-top: 1.5rem;
    border-top: 1px solid var(--color-base-300);
  }

  .doc-neighbor {
    display: grid;
    gap: 0.125rem;
    text-decoration: none;
    padding: 0.75rem 1rem;
    border: 1px solid var(--color-base-300);
    border-radius: var(--radius-box, 0.5rem);
    max-width: 20rem;
  }

  .doc-neighbor:hover {
    background: var(--color-base-200);
  }

  .doc-neighbor-next {
    text-align: right;
  }

  .doc-neighbor-role {
    font-size: 0.75rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    opacity: 0.65;
  }

  .doc-neighbor-label {
    font-weight: 600;
  }
</style>
