<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { CATEGORIES, CRATES, REPO, specUrl, type CrateCategory } from '$lib/data/crates';

  const contents = [
    { id: 'what-a-spec-is', label: 'What a spec is here' },
    { id: 'by-crate', label: 'The specifications, by crate' },
    { id: 'no-spec', label: 'The crates with no spec, and why' },
    { id: 'reading-them', label: 'How to read them together' }
  ];

  const order: CrateCategory[] = ['core', 'transport', 'conversion', 'tooling'];
  const specced = CRATES.filter((crate) => crate.spec);
  const unspecced = CRATES.filter((crate) => !crate.spec);

  /** The section headings each spec is organized into, for orientation. */
  const outlines: Record<string, string[]> = {
    'hl7-2': [
      '§2 the ER7 layer',
      '§3 the dictionary',
      '§4 generic mode',
      '§5 schema mode',
      '§6 struct mode',
      '§7 mutation and building',
      '§8 validation',
      '§12 command line'
    ],
    'hl7-3': [
      '§1 scope — read before filing a bug',
      '§2 the XML layer',
      '§3 data types',
      '§4 the RIM backbone',
      '§5 the three-level envelope',
      '§6 vocabulary validation as future work'
    ],
    'hl7-2-mllp': [
      '§2 the protocol',
      '§3 framing',
      '§4 streaming',
      '§5 acknowledgement',
      '§6 transport',
      '§7 features'
    ],
    'hl7-2-soap': [
      '§3 the envelope',
      '§4 what the body carries',
      '§5 faults',
      '§6 responses',
      '§7 WSDL'
    ],
    'hl7-3-soap': [
      '§3 the envelope',
      '§4 what the body carries',
      '§5 faults',
      '§6 responses',
      '§7 WSDL'
    ],
    'hl7-2-from-er7-into-xml': [
      '§2 ER7 parsing',
      '§3 message-structure grouping',
      '§4 XML naming and rendering',
      '§4a schema mode',
      '§5 batch input',
      '§8 command line'
    ],
    'hl7-2-from-er7-into-json': [
      '§0 relationship to the XML sibling',
      '§2 ER7 parsing',
      '§3 grouping',
      '§4 JSON key naming',
      '§5 batch input',
      '§8 command line'
    ],
    'hl7-2-from-xml-into-er7': [
      '§2 XML parsing',
      '§3 reconstructing the value tree',
      '§4 segment- and field-level special cases',
      '§5 fallbacks and limitations'
    ],
    'hl7-2-from-json-into-er7': [
      '§2 JSON parsing',
      '§3 reconstructing the value tree',
      '§4 segment- and field-level special cases',
      '§5 fallbacks and limitations'
    ],
    'hl7-2-from-xsd-into-json-dictionary': [
      '§2 input',
      '§3 what is read, and what it becomes',
      '§4 what the schemas cannot say',
      '§5 output',
      '§8 command line'
    ],
    'hl7-2-xml-lite-helper': [
      '§2 why it exists, and why the name',
      '§3 reading',
      '§4 writing',
      '§6 what this is not'
    ]
  };
</script>

<DocPage
  title="Specifications"
  lede="Ten of the fourteen crates carry a normative spec/index.md — the single source of truth for that crate's behavior. This site does not restate them; it points at them and says what each one covers."
  {contents}
>
  <h2 id="what-a-spec-is">What a spec is here</h2>
  <p>
    Each crate's <code>spec/index.md</code> is a numbered, section-by-section statement of every
    rule the crate implements: how a delimiter is resolved, what a name falls back to, which
    inputs are errors and which are warnings, what the command line does with each flag. It is
    written to be cited — “<code>spec/index.md</code> §4.2” is a real address you can quote in a
    bug report.
  </p>
  <Callout heading="The spec wins">
    <p>
      Where a crate's README, its rustdoc, or this website disagrees with its
      <code>spec/index.md</code>, the spec is right and the other three are the bug. That ordering
      is stated in the crates' own READMEs, and it is why nothing on this site is written as
      normative text.
    </p>
  </Callout>

  <h2 id="by-crate">The specifications, by crate</h2>
  {#each order as category (category)}
    {@const inCategory = specced.filter((crate) => crate.category === category)}
    {#if inCategory.length > 0}
      <h3>{CATEGORIES[category].label}</h3>
      <dl>
        {#each inCategory as crate (crate.name)}
          <dt>
            <a href={specUrl(crate)}>{crate.name}</a> —
            <a href={`/crates/${crate.slug}/`}>crate page</a>
          </dt>
          <dd>
            {crate.tagline}.
            {#if outlines[crate.name]}
              <br /><span class="outline">{outlines[crate.name].join(' · ')}</span>
            {/if}
          </dd>
        {/each}
      </dl>
    {/if}
  {/each}

  <h2 id="no-spec">The crates with no spec, and why</h2>
  <ul>
    {#each unspecced as crate (crate.name)}
      <li>
        <a href={`/crates/${crate.slug}/`}><code>{crate.name}</code></a> —
        {#if crate.name === 'hl7'}
          a thin re-export (<code>pub use hl7_2 as v2;</code>) with nothing normative of its own to
          state. Read <a href={specUrl(CRATES.find((c) => c.name === 'hl7-2')!)}>hl7-2's spec</a>
          and <a href={specUrl(CRATES.find((c) => c.name === 'hl7-3')!)}>hl7-3's</a> instead.
        {:else if crate.name === 'hl7-2-derive'}
          macro behavior is documented in the crate's own README and, normatively, in
          <a href={specUrl(CRATES.find((c) => c.name === 'hl7-2')!)}>hl7-2's spec §6 (struct mode)</a>.
        {:else}
          macro behavior is documented in the crate's own README and in <code>hl7-3</code>'s
          <code>typed</code> module, rather than as a spec of its own.
        {/if}
      </li>
    {/each}
  </ul>

  <h2 id="reading-them">How to read them together</h2>
  <p>
    The two <em>forward</em> conversion specs (ER7 → XML and ER7 → JSON) are deliberately kept
    consistent with each other, and each one's §0 says exactly where they are meant to diverge:
    JSON uses real arrays and real <code>null</code>; XML follows v2.xml's element and attribute
    constructs. Reading one tells you most of the other.
  </p>
  <p>
    Each <em>reverse</em> spec documents in its own §1.1 why it needs no HL7 v2.5 data-type
    dictionary at all: every element or key name its forward counterpart writes already carries
    its own position, so reversing the conversion is a purely structural, position-based rebuild.
    That is also why the reverse crates are so much smaller than the forward ones.
  </p>
  <p>
    The workspace itself has one spec outside any crate:
    <a href={`${REPO}/blob/main/spec/rust-msrv-n-minus-3/index.md`}
      ><code>spec/rust-msrv-n-minus-3/index.md</code></a
    >, which states the minimum supported Rust version policy every member pins to. See
    <a href="/docs/versions/">Versions and compatibility</a> for what that means in practice.
  </p>
</DocPage>

<style>
  .outline {
    font-size: 0.875rem;
    opacity: 0.75;
  }
</style>
