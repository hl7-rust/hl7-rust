<script lang="ts">
  /**
   * Light/dark switch — thin site wiring around the published
   * lily-design-system-svelte-theme-picker package (icon button, WAI-ARIA
   * APG listbox, localStorage persistence) instead of a hand-rolled toggle.
   *
   * Both real theme stylesheets (static/lily-light.css and
   * static/lily-dark.css) are loaded unconditionally in app.html and key
   * entirely off the data-theme attribute this component sets. The files
   * under static/themes/ that this component's managed <link> swaps between
   * are deliberately empty placeholders — see the comment in each — so the
   * dynamic-load side effect this package is built around never has to do
   * real work here.
   *
   * `storageKey` matches the key app.html's own inline anti-flash script
   * reads, so an existing stored choice — from this component or its
   * hand-rolled predecessor, which wrote the same key — never flashes wrong
   * on first paint. `detectFromSystem` resolves prefers-color-scheme once on
   * a first visit with no stored choice, matching what the CSS media query
   * already rendered before hydration, so there is nothing to flash either
   * way. (It does not keep following a later OS change with no stored
   * choice, the way the pure-CSS fallback used to — an explicit choice,
   * once resolved, is what every other themed site does too.)
   */
  import { ThemePicker } from 'lily-design-system-svelte-theme-picker';
</script>

<ThemePicker
  label="Theme"
  themesUrl="/themes/"
  themes={['light', 'dark']}
  detectFromSystem
  storageKey="hl7-rust-theme"
/>
