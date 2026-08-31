<script lang="ts">
  /**
   * Text-size switch — thin site wiring around the published
   * lily-design-system-svelte-text-size-picker package. Sets
   * data-text-size on <html>; the actual font-size-per-slug rules live in
   * +layout.svelte's global styles, next to the other document-level
   * baseline CSS that file already owns.
   *
   * No pre-hydration anti-flash handling, unlike the theme picker: a
   * returning visitor's stored size only applies after mount, which is a
   * brief, one-time reflow rather than a color flash — not worth the extra
   * inline script for the visual difference at stake.
   */
  import { TextSizePicker } from 'lily-design-system-svelte-text-size-picker';
</script>

<TextSizePicker
  label="Text size"
  sizes={['small', 'medium', 'large', 'x-large']}
  storageKey="hl7-rust-text-size"
/>
