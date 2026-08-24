<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import LinkCards from '$lib/components/LinkCards.svelte';
  import { SECTIONS } from '$lib/data/navigation';

  const guides = SECTIONS.find((section) => section.href === '/guides/')!;

  const contents = [
    { id: 'working-with-v2', label: 'Working with a v2 message' },
    { id: 'moving-messages', label: 'Moving messages around' },
    { id: 'other-standards', label: 'The other standard' },
    { id: 'reading-order', label: 'A suggested reading order' }
  ];

  const pick = (...hrefs: string[]) =>
    hrefs.map((href) => guides.links.find((link) => link.href === href)!);
</script>

<DocPage
  title="Guides"
  lede="One task per page, each with the API calls that do it and the reasoning behind the choices these crates make. Guides assume you have read the quick start; they do not assume you have read each other."
  {contents}
>
  <h2 id="working-with-v2">Working with a v2 message</h2>
  <p>
    The five guides that cover the life of a message inside your process: getting it in, finding
    things in it, changing it, checking it, and teaching the parser a dialect it does not know.
  </p>
  <LinkCards
    label="Message guides"
    links={pick(
      '/guides/parsing/',
      '/guides/navigating/',
      '/guides/modifying/',
      '/guides/validating/',
      '/guides/dictionaries/',
      '/guides/struct-mode/'
    )}
  />

  <h2 id="moving-messages">Moving messages around</h2>
  <p>
    Getting a message to or from another system — over a socket, over HTTP, or into a format
    something downstream can read.
  </p>
  <LinkCards
    label="Transport and conversion guides"
    links={pick('/guides/converting/', '/guides/mllp/', '/guides/soap/')}
  />

  <h2 id="other-standards">The other standard</h2>
  <LinkCards label="HL7 v3" links={pick('/guides/hl7-v3/')} />

  <h2 id="reading-order">A suggested reading order</h2>
  <p>
    If you are integrating a new v2 feed and want to read straight through, this is the order the
    work usually happens in:
  </p>
  <ol>
    <li>
      <a href="/guides/parsing/">Parsing</a> — pick a mode. Almost everyone starts in generic mode
      whether they mean to or not.
    </li>
    <li>
      <a href="/guides/navigating/">Navigating</a> — learn the path grammar. Everything else takes
      paths.
    </li>
    <li>
      <a href="/guides/validating/">Validating</a> — decide what you will reject, before a bad
      message decides for you.
    </li>
    <li>
      <a href="/guides/dictionaries/">Vendor dictionaries</a> — the first time the sender's
      Z-segments matter.
    </li>
    <li>
      <a href="/guides/struct-mode/">Struct mode</a> — once the interface has held still long
      enough to trust.
    </li>
    <li>
      <a href="/guides/modifying/">Modifying and building</a> — because a system that reads HL7
      almost always has to answer in it.
    </li>
    <li>
      <a href="/guides/mllp/">MLLP</a> or <a href="/guides/soap/">SOAP</a> — whichever the far end
      speaks.
    </li>
  </ol>
  <p>
    For longer, start-to-finish walkthroughs of the same ground, see
    <a href="/tutorials/">Tutorials</a>. For short copyable snippets with no prose around them, see
    <a href="/examples/">Examples</a>.
  </p>
</DocPage>
