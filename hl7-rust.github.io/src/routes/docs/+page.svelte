<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import LinkCards from '$lib/components/LinkCards.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { SECTIONS } from '$lib/data/navigation';

  const docs = SECTIONS.find((section) => section.href === '/docs/')!;
  const guides = SECTIONS.find((section) => section.href === '/guides/')!;
  const tutorials = SECTIONS.find((section) => section.href === '/tutorials/')!;

  const contents = [
    { id: 'in-this-section', label: 'In this section' },
    { id: 'which-crate', label: 'Which crate do I need?' },
    { id: 'elsewhere', label: 'Everywhere else on this site' },
    { id: 'sources', label: 'Where the authority lives' }
  ];
</script>

<DocPage
  lede="Install it, run it, and learn the vocabulary the rest of the site assumes. If you have never touched HL7 before, read Concepts first; if you have, the quick start is five minutes."
  {contents}
>
  <h2 id="in-this-section">In this section</h2>
  <LinkCards links={docs.links} label="Documentation pages" />

  <h2 id="which-crate">Which crate do I need?</h2>
  <p>
    Fourteen crates is a lot to face on day one. In practice the answer is short, and it depends
    on one question: what does the system at the other end send you?
  </p>
  <dl>
    <dt>Pipe-delimited HL7 v2 text, and you want to read or write it</dt>
    <dd>
      <code>cargo add hl7</code> and use <code>hl7::v2</code>. That is the umbrella crate
      re-exporting <a href="/crates/hl7-2/"><code>hl7-2</code></a>, which is where essentially all
      the v2 capability lives. Everything else in the workspace is a layer around it.
    </dd>
    <dt>The same, but over a TCP socket</dt>
    <dd>
      Add <a href="/crates/hl7-2-mllp/"><code>hl7-2-mllp</code></a>. MLLP is how HL7 v2 actually
      crosses a network, and the crate covers framing, streaming, transports, and
      acknowledgements.
    </dd>
    <dt>The same, but over HTTP with a WSDL</dt>
    <dd>
      Add <a href="/crates/hl7-2-soap/"><code>hl7-2-soap</code></a>. It turns bytes into meaning
      and back and leaves the socket to whatever HTTP stack you already use.
    </dd>
    <dt>You need the messages as XML or JSON for something downstream</dt>
    <dd>
      The four <a href="/guides/converting/">conversion crates</a>, or their command-line tools if
      you would rather not write Rust at all.
    </dd>
    <dt>HL7 v3 XML — a CDA-adjacent document, a national registry, an IHE profile</dt>
    <dd>
      <a href="/crates/hl7-3/"><code>hl7-3</code></a> for the model, and
      <a href="/crates/hl7-3-soap/"><code>hl7-3-soap</code></a> for the transport. Read that
      crate's scope section first: it is a foundation, not a complete implementation of v3.
    </dd>
  </dl>
  <p>See <a href="/docs/architecture/">Architecture</a> for the full map and the reasoning.</p>

  <Callout heading="You can take one layer and leave the rest">
    <p>
      Every crate here has a dependency count you can hold in your head — usually one, sometimes
      zero. That is deliberate: in a domain where dependency trees get audited, a small tree is
      worth a few hundred hand-written lines. Nothing forces you to take the whole workspace.
    </p>
  </Callout>

  <h2 id="elsewhere">Everywhere else on this site</h2>
  <h3>{guides.title}</h3>
  <p>{guides.blurb}</p>
  <LinkCards links={guides.links.slice(0, 4)} headingLevel={4} label="Selected guides" />
  <p><a href="/guides/">All {guides.links.length} guides →</a></p>

  <h3>{tutorials.title}</h3>
  <p>{tutorials.blurb}</p>
  <LinkCards links={tutorials.links} headingLevel={4} label="Tutorials" />

  <h2 id="sources">Where the authority lives</h2>
  <p>
    This website is an introduction and a map. It is not normative, and it deliberately does not
    restate conversion rules or parsing rules in its own words. Four places carry actual
    authority, in this order:
  </p>
  <ol>
    <li>
      Each crate's <a href="/spec/"><code>spec/index.md</code></a> — numbered, section by section,
      the single source of truth for that crate's behavior.
    </li>
    <li>The crate's rustdoc on docs.rs, generated from the code that ships.</li>
    <li>The crate's <code>README.md</code>, which is a tour rather than a specification.</li>
    <li>This site, which summarises all three and links back to them.</li>
  </ol>
  <p>
    Where any two disagree, the higher one wins — and the lower one is a bug worth reporting. See
    <a href="/help/support/">Support and contributing</a>.
  </p>
</DocPage>
