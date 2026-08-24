<script lang="ts">
  import { page } from '$app/state';
  import SkipLink from 'lily-design-system-svelte-headless/components/SkipLink/SkipLink.svelte';
  import Header from 'lily-design-system-svelte-headless/components/Header/Header.svelte';
  import NavigationMenu from 'lily-design-system-svelte-headless/components/NavigationMenu/NavigationMenu.svelte';
  import Footer from 'lily-design-system-svelte-headless/components/Footer/Footer.svelte';

  let { children } = $props();

  type NavLink = { href: string; label: string };
  const navLinks: NavLink[] = [
    { href: '/', label: 'Home' },
    { href: '/spec/', label: 'Specs' }
  ];

  function isCurrent(href: string): boolean {
    return page.url.pathname === href;
  }
</script>

<SkipLink href="#main" label="Skip to main content" />

<Header label="Site header" class="site-header">
  <div class="site-header-inner">
    <a class="site-brand" href="/" aria-label="HL7 Rust home">
      <span class="site-brand-mark" aria-hidden="true">H7</span>
      <span>HL7 Rust</span>
    </a>
    <NavigationMenu label="Main" class="site-nav">
      {#each navLinks as link (link.href)}
        <a href={link.href} aria-current={isCurrent(link.href) ? 'page' : undefined}>
          {link.label}
        </a>
      {/each}
      <a href="https://github.com/hl7-rust">GitHub</a>
    </NavigationMenu>
  </div>
</Header>

<main id="main" class="site-main">
  {@render children()}
</main>

<Footer label="Site footer" class="site-footer">
  <div class="site-footer-inner">
    <p>Free and open source — MIT, Apache-2.0, BSD-3-Clause, GPL-2.0, or GPL-3.0.</p>
    <div class="site-footer-links">
      <a href="https://github.com/hl7-rust">GitHub</a>
      <a href="https://gitlab.com/hl7-rust">GitLab</a>
      <a href="https://codeberg.org/hl7-rust">Codeberg</a>
      <a href="/spec/">Specs</a>
    </div>
  </div>
</Footer>

<style>
  .site-header-inner,
  .site-footer-inner {
    max-width: 64rem;
    margin: 0 auto;
    padding: 0 1.5rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .site-brand {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 700;
    font-size: 1.125rem;
    text-decoration: none;
    color: inherit;
  }

  .site-brand-mark {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border-radius: 0.375rem;
    background: var(--color-primary, #dea584);
    color: var(--color-primary-content, #1c1917);
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    font-weight: 700;
  }

  :global(.site-nav) {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    flex-wrap: wrap;
  }

  .site-main {
    min-height: 60vh;
  }

  .site-footer-inner {
    padding-top: 1.5rem;
    padding-bottom: 1.5rem;
    font-size: 0.875rem;
  }

  .site-footer-links {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
  }
</style>
