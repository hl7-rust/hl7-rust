/**
 * This site's theme convention: attribute-based, for a multi-stylesheet
 * setup.
 *
 * Every theme's CSS is loaded unconditionally, up front, in app.html
 * (today: lily-light.css and lily-dark.css) — nothing is fetched or
 * swapped when the theme changes. Switching a theme means setting one
 * attribute, `data-theme`, on `<html>`; each stylesheet scopes its own
 * rules to the attribute value it owns (the light file to bare `:root`,
 * the dark file to `:root[data-theme="dark"]` and to
 * `prefers-color-scheme`), so the browser's own cascade — not
 * JavaScript — picks which one is active. No flash, no extra network
 * request on switch, and adding a theme later means adding a stylesheet
 * and a slug here, not rewiring how switching works.
 *
 * lily-design-system-svelte-theme-picker's own model doesn't assume
 * this: its API is built around fetching one theme's stylesheet on
 * demand and swapping it in. ThemeToggle.svelte uses THEMES below to
 * drive the picker, but points its `themesUrl` at placeholder files
 * (static/themes/*.css, deliberately empty) so that per-theme fetch is
 * a harmless no-op — the picker's `data-theme` side effect is the only
 * part this convention actually needs.
 */
export const THEMES = ['light', 'dark'] as const;

export type Theme = (typeof THEMES)[number];
