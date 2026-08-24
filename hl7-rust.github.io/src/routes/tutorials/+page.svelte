<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import LinkCards from '$lib/components/LinkCards.svelte';
  import { SECTIONS } from '$lib/data/navigation';

  const tutorials = SECTIONS.find((section) => section.href === '/tutorials/')!;

  const contents = [
    { id: 'the-tutorials', label: 'The tutorials' },
    { id: 'difference', label: 'How these differ from the guides' },
    { id: 'before', label: 'Before you start' }
  ];
</script>

<DocPage
  title="Tutorials"
  lede="Longer, start-to-finish walkthroughs you can follow at a keyboard. Each one builds something that runs, and explains the decisions along the way rather than only the API calls."
  {contents}
>
  <h2 id="the-tutorials">The tutorials</h2>
  <LinkCards links={tutorials.links} label="Tutorials" />

  <h2 id="difference">How these differ from the guides</h2>
  <p>
    A <a href="/guides/">guide</a> answers “how do I do X” and assumes you already know why you
    want to. A tutorial starts from a situation — a vendor sent you a file, a hospital wants a
    listener — and walks the whole way to something that works, including the parts that are
    judgement rather than API.
  </p>
  <p>
    They overlap on purpose. If you have read the guides and want to see them assembled, start
    here; if you have followed a tutorial and want the full surface of one step, the guides are
    linked throughout.
  </p>

  <h2 id="before">Before you start</h2>
  <ul>
    <li>
      A Rust toolchain at or above the <a href="/docs/versions/#msrv">MSRV</a>, and nothing else —
      there is no C dependency anywhere in the workspace.
    </li>
    <li>
      For the tutorials that use the shell, <code>cargo install hl7-2</code>, which gives you a
      binary named <code>hl7-v2</code>. See <a href="/docs/install/#cli">Install</a>.
    </li>
    <li>
      Enough of <a href="/docs/concepts/">Concepts</a> to know what a segment, a field, and a
      component are. The tutorials assume that vocabulary and no more.
    </li>
  </ul>
</DocPage>
