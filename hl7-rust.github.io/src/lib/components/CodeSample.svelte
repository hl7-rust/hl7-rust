<script lang="ts">
  /**
   * A labelled code block with a copy button.
   *
   * The code is a string prop rather than a snippet, for two reasons: the copy
   * button needs the exact text, and Svelte would otherwise reindent it to
   * match the surrounding markup. Nothing is syntax-highlighted — a wrong
   * highlight in an HL7 sample is worse than none, because `|^~\&` and `\X0A\`
   * are values, not punctuation.
   */
  import { ClipboardCopyButton, CodeBlock } from 'lily-design-system-svelte-headless';

  let {
    code,
    language = 'rust',
    caption = undefined,
    label = undefined
  }: {
    /** Exact source text. Rendered verbatim and copied verbatim. */
    code: string;
    /** Shown in the block's header bar. Free text, not a highlighter token. */
    language?: string;
    /** One line above the block saying what it does. */
    caption?: string;
    /** Accessible name, when the caption is not enough. */
    label?: string;
  } = $props();

  let copied = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  function flash() {
    copied = true;
    clearTimeout(timer);
    timer = setTimeout(() => (copied = false), 1600);
  }
</script>

{#if caption}
  <p class="code-caption">{caption}</p>
{/if}

<CodeBlock class="sample" label={label ?? caption ?? `${language} sample`}>
  <div class="sample-bar">
    <span class="sample-language">{language}</span>
    <ClipboardCopyButton class="sample-copy" text={code} label="Copy this code" onsuccess={flash}>
      {copied ? 'Copied' : 'Copy'}
    </ClipboardCopyButton>
  </div>
  <pre><code>{code}</code></pre>
</CodeBlock>

<style>
  .code-caption {
    margin: 1.5rem 0 0.5rem;
    font-size: 0.9375rem;
    opacity: 0.85;
  }

  /* The Lily .code-block treatment is a neutral-filled chrome panel, which is
     the same in both themes — so a sample reads as a sample rather than as
     page background with a border round it. What is added here is the header
     bar, the padding reset, and the horizontal scroll container. */
  :global(.code-block.sample) {
    margin: 0 0 1.5rem;
    padding: 0;
    border-radius: var(--radius-box, 0.5rem);
    overflow: hidden;
    white-space: normal;
  }

  .sample-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.375rem 0.5rem 0.375rem 0.875rem;
    background: rgb(255 255 255 / 0.06);
    border-bottom: 1px solid rgb(255 255 255 / 0.12);
  }

  .sample-language {
    font-size: 0.75rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    opacity: 0.6;
  }

  :global(.clipboard-copy-button.sample-copy) {
    font-size: 0.75rem;
    font-family: inherit;
    padding: 0.25rem 0.625rem;
    border: 1px solid currentColor;
    border-radius: var(--radius-field, 0.25rem);
    background: transparent;
    color: inherit;
    cursor: pointer;
    opacity: 0.6;
  }

  :global(.clipboard-copy-button.sample-copy:hover) {
    opacity: 1;
    background: rgb(255 255 255 / 0.1);
  }

  .sample-bar + :global(pre) {
    margin: 0;
    padding: 1rem 1.125rem;
    overflow-x: auto;
    background: transparent;
    color: inherit;
    font-size: 0.875rem;
    line-height: 1.55;
  }

  .sample-bar + :global(pre) :global(code) {
    background: none;
    padding: 0;
    color: inherit;
  }
</style>
