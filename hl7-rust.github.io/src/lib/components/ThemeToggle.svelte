<script lang="ts">
  /**
   * Light/dark switch.
   *
   * Writes `data-theme` on <html>, which is the hook both Lily theme files
   * key off, and remembers the choice in localStorage. The inline script in
   * app.html applies the stored value before first paint; this component only
   * has to keep its own label in step with it after hydration.
   */
  import { onMount } from 'svelte';

  type Theme = 'light' | 'dark';

  let theme = $state<Theme>('light');
  let ready = $state(false);

  onMount(() => {
    const attribute = document.documentElement.getAttribute('data-theme');
    theme =
      attribute === 'dark' || attribute === 'light'
        ? attribute
        : window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light';
    ready = true;
  });

  function toggle() {
    theme = theme === 'dark' ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', theme);
    try {
      localStorage.setItem('hl7-rust-theme', theme);
    } catch {
      // Private browsing, or storage disabled. The choice still applies to
      // this page; it just will not survive a reload.
    }
  }
</script>

<button
  type="button"
  class="theme-toggle"
  onclick={toggle}
  aria-pressed={ready ? theme === 'dark' : undefined}
  aria-label={theme === 'dark' ? 'Switch to the light theme' : 'Switch to the dark theme'}
>
  <span aria-hidden="true">{theme === 'dark' ? '☾' : '☀'}</span>
  <span class="theme-toggle-text">{theme === 'dark' ? 'Dark' : 'Light'}</span>
</button>

<style>
  .theme-toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.3125rem 0.625rem;
    font: inherit;
    font-size: 0.8125rem;
    color: inherit;
    background: transparent;
    border: 1px solid var(--color-base-300);
    border-radius: var(--radius-field, 0.25rem);
    cursor: pointer;
  }

  .theme-toggle:hover {
    background: var(--color-base-200);
  }

  @media (max-width: 40rem) {
    .theme-toggle-text {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip-path: inset(50%);
      white-space: nowrap;
    }
  }
</style>
