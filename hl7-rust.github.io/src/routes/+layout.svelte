<script lang="ts">
  /**
   * Site chrome, and the only place that owns prose styling.
   *
   * The sidebar is not a per-section layout file: every section's links live in
   * $lib/data/navigation.ts, so one layout can render the right sidebar for
   * whichever section the current URL falls in, and a section with no child
   * pages simply gets none.
   */
  import { page } from '$app/state';
  import { Footer, Header, NavigationMenu, Sidebar, SkipLink } from 'lily-design-system-svelte-headless';
  import ThemeToggle from '$lib/components/ThemeToggle.svelte';
  import { PRIMARY_NAV, SECTIONS } from '$lib/data/navigation';
  import { ORG, REPO } from '$lib/data/crates';

  let { children } = $props();

  const pathname = $derived(page.url.pathname);
  const section = $derived(
    SECTIONS.find((candidate) => pathname.startsWith(candidate.href) && candidate.links.length > 0)
  );
</script>

<SkipLink href="#main" label="Skip to main content" />

<Header label="Site header" class="site-header">
  <div class="site-header-inner">
    <a class="site-brand" href="/" aria-label="HL7® Rust home">
      <span class="site-brand-mark" aria-hidden="true">H7</span>
      <span>HL7® Rust</span>
    </a>
    <NavigationMenu label="Main" class="site-nav">
      {#each PRIMARY_NAV as link (link.href)}
        <a
          href={link.href}
          aria-current={pathname === link.href ||
          (link.href !== '/' && pathname.startsWith(link.href))
            ? 'page'
            : undefined}
        >
          {link.label}
        </a>
      {/each}
      <a href={ORG}>GitHub</a>
    </NavigationMenu>
    <ThemeToggle />
  </div>
</Header>

<div class="site-shell" class:site-shell-wide={!section}>
  {#if section}
    <Sidebar label={`${section.title} navigation`} class="site-sidebar">
      <p class="site-sidebar-heading">
        <a href={section.href}>{section.title}</a>
      </p>
      <ul class="site-sidebar-list">
        {#each section.links as link (link.href)}
          <li>
            <a href={link.href} aria-current={pathname === link.href ? 'page' : undefined}>
              {link.label}
            </a>
          </li>
        {/each}
      </ul>
    </Sidebar>
  {/if}

  <main id="main" class="site-main">
    {@render children()}
  </main>
</div>

<Footer label="Site footer" class="site-footer">
  <div class="site-footer-inner">
    <div>
      <p class="site-footer-title">HL7 Rust</p>
      <p>
        Free and open source, licensed under any of MIT, Apache-2.0, BSD-3-Clause, GPL-2.0-only,
        or GPL-3.0-only, at your option.
      </p>
    </div>
    <nav class="site-footer-columns" aria-label="Footer">
      <div>
        <p class="site-footer-column-title">Learn</p>
        <a href="/docs/">Documentation</a>
        <a href="/guides/">Guides</a>
        <a href="/tutorials/">Tutorials</a>
        <a href="/examples/">Examples</a>
      </div>
      <div>
        <p class="site-footer-column-title">Reference</p>
        <a href="/crates/">Crates</a>
        <a href="/spec/">Specs</a>
        <a href="/docs/cli/">Command line</a>
        <a href="/docs/versions/">Versions</a>
      </div>
      <div>
        <p class="site-footer-column-title">Project</p>
        <a href={REPO}>GitHub</a>
        <a href="https://gitlab.com/hl7-rust">GitLab</a>
        <a href="https://codeberg.org/hl7-rust">Codeberg</a>
        <a href="/help/">Help</a>
      </div>
    </nav>
  </div>
  <p class="site-footer-trademark">
    HL7®, and FHIR® are the registered trademarks of Health Level Seven International and their
    use of these trademarks does not constitute an endorsement by HL7.
  </p>
</Footer>

<style>
  .site-footer-trademark {
    max-width: 84rem;
    margin: 1.5rem auto 0;
    padding: 1rem 1.5rem 0;
    border-top: 1px solid var(--color-base-300);
    font-size: 0.8125rem;
    line-height: 1.5;
    opacity: 0.75;
  }

  .site-header-inner {
    max-width: 84rem;
    margin: 0 auto;
    padding: 0.75rem 1.5rem;
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .site-brand {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 700;
    font-size: 1.125rem;
    text-decoration: none;
    color: inherit;
    margin-right: auto;
  }

  .site-brand-mark {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border-radius: 0.375rem;
    background: var(--color-primary);
    color: var(--color-primary-content);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.75rem;
    font-weight: 700;
  }

  :global(.site-nav) {
    display: flex;
    align-items: center;
    gap: 1.125rem;
    flex-wrap: wrap;
    font-size: 0.9375rem;
  }

  :global(.site-nav a) {
    color: inherit;
    text-decoration: none;
  }

  :global(.site-nav a:hover) {
    text-decoration: underline;
    text-underline-offset: 4px;
  }

  :global(.site-nav a[aria-current='page']) {
    font-weight: 700;
    text-decoration-thickness: 2px;
    text-underline-offset: 4px;
  }

  .site-shell {
    max-width: 84rem;
    margin: 0 auto;
    padding: 0 1.5rem;
    display: grid;
    grid-template-columns: 15rem minmax(0, 1fr);
    gap: 3rem;
    align-items: start;
  }

  .site-shell-wide {
    display: block;
    max-width: none;
    padding: 0;
  }

  :global(.site-sidebar) {
    background: transparent;
    border-radius: 0;
    position: sticky;
    top: 1.5rem;
    padding: 2rem 0 2rem;
    font-size: 0.9375rem;
  }

  .site-sidebar-heading {
    margin: 0 0 0.75rem;
    font-weight: 700;
    font-size: 0.8125rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .site-sidebar-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 0.125rem;
    border-left: 1px solid var(--color-base-300);
  }

  .site-sidebar-list a {
    display: block;
    padding: 0.3125rem 0.75rem;
    margin-left: -1px;
    border-left: 2px solid transparent;
    text-decoration: none;
    overflow-wrap: anywhere;
  }

  .site-sidebar-list a:hover {
    background: var(--color-base-200);
  }

  .site-sidebar-list a[aria-current='page'] {
    border-left-color: var(--color-primary);
    font-weight: 700;
    color: var(--color-primary);
  }

  .site-main {
    min-height: 60vh;
    padding: 2rem 0 4rem;
  }

  .site-shell-wide .site-main {
    padding: 0 0 4rem;
  }

  /* Narrow viewports: the section nav becomes one horizontally scrollable row
     of chips rather than a tall column the reader has to scroll past to reach
     the page they already chose. */
  @media (max-width: 62rem) {
    .site-shell {
      grid-template-columns: minmax(0, 1fr);
      gap: 0;
    }

    :global(.site-sidebar) {
      position: static;
      padding: 1rem 0 0;
      margin: 0 -1.5rem;
    }

    .site-sidebar-heading {
      padding: 0 1.5rem;
    }

    .site-sidebar-list {
      display: flex;
      gap: 0.5rem;
      overflow-x: auto;
      border-left: none;
      padding: 0 1.5rem 0.5rem;
      scrollbar-width: thin;
    }

    .site-sidebar-list a {
      white-space: nowrap;
      overflow-wrap: normal;
      margin-left: 0;
      padding: 0.3125rem 0.75rem;
      border: 1px solid var(--color-base-300);
      border-radius: 999px;
    }

    .site-sidebar-list a[aria-current='page'] {
      border-color: var(--color-primary);
    }

    /* Brand and theme toggle share the first row; the nav wraps below both. */
    :global(.site-nav) {
      order: 3;
      flex-basis: 100%;
    }
  }

  .site-footer-inner {
    max-width: 84rem;
    margin: 0 auto;
    padding: 2.5rem 1.5rem;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 2.5rem;
    font-size: 0.875rem;
  }

  .site-footer-title {
    font-weight: 700;
    margin: 0 0 0.375rem;
  }

  .site-footer-columns {
    display: flex;
    gap: 3rem;
    flex-wrap: wrap;
  }

  .site-footer-columns div {
    display: grid;
    gap: 0.25rem;
    align-content: start;
  }

  .site-footer-column-title {
    margin: 0 0 0.25rem;
    font-weight: 700;
  }

  @media (max-width: 48rem) {
    .site-footer-inner {
      grid-template-columns: minmax(0, 1fr);
    }
  }

  /* ---------------------------------------------------------------------
     Page baseline.

     The Lily theme declares --lily-font-body but does not apply it to the
     document, on the grounds that the page owns its own typography. So this
     is where the site claims it — otherwise the browser default (serif)
     wins and nothing else in the theme looks like it belongs to it.
     --------------------------------------------------------------------- */

  :global(body) {
    margin: 0;
    font-family: var(--lily-font-body);
    background: var(--color-base-100);
    color: var(--color-base-content);
    -webkit-font-smoothing: antialiased;
  }

  /* Lily gives every *-list-item a hairline rule between rows, which is right
     for a summary list and wrong for a breadcrumb trail, an on-page contents
     list, and a sidebar. Those three are opted out here rather than page by
     page. */
  :global(.doc .breadcrumb-list-item),
  :global(.doc .contents-list-item) {
    padding: 0;
    border-bottom: none;
  }

  :global(.doc .breadcrumb-list) {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    font-size: 0.875rem;
    margin: 0 0 1.5rem;
  }

  :global(.doc .breadcrumb-list-item + .breadcrumb-list-item::before) {
    content: '/';
    margin-right: 0.5rem;
    opacity: 0.5;
  }

  /* ---------------------------------------------------------------------
     Prose. Every page's body is `.doc-body`, so the reading measure, the
     heading rhythm, and the table and code treatment are defined once here
     rather than repeated per page.
     --------------------------------------------------------------------- */

  :global(.doc-body) {
    line-height: 1.7;
  }

  :global(.doc-body h2) {
    margin: 3rem 0 0.75rem;
    padding-top: 0.5rem;
    font-size: 1.5rem;
    scroll-margin-top: 1.5rem;
  }

  :global(.doc-body h3) {
    margin: 2rem 0 0.5rem;
    font-size: 1.1875rem;
    scroll-margin-top: 1.5rem;
  }

  :global(.doc-body h4) {
    margin: 1.5rem 0 0.5rem;
    font-size: 1.0625rem;
  }

  :global(.doc-body p),
  :global(.doc-body li) {
    max-width: 44rem;
  }

  :global(.doc-body ul),
  :global(.doc-body ol) {
    display: grid;
    gap: 0.5rem;
    padding-left: 1.25rem;
  }

  :global(.doc-body code) {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.9em;
    overflow-wrap: anywhere;
  }

  :global(.doc-body :not(pre) > code) {
    padding: 0.1em 0.35em;
    border-radius: 0.25rem;
    background: var(--color-base-200);
  }

  :global(.doc-body table) {
    width: 100%;
    margin: 1.5rem 0;
    border-collapse: collapse;
  }

  :global(.doc-body table th),
  :global(.doc-body table td) {
    text-align: left;
    vertical-align: top;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--color-base-300);
  }

  :global(.doc-body .table-wrap) {
    overflow-x: auto;
  }

  :global(.doc-body dl) {
    display: grid;
    gap: 0.25rem;
    margin: 1.25rem 0;
  }

  :global(.doc-body dt) {
    font-weight: 700;
  }

  :global(.doc-body dd) {
    margin: 0 0 0.75rem;
    max-width: 44rem;
  }
</style>
