<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { CATEGORIES, CRATES, REPO, specUrl, type CrateCategory } from '$lib/data/crates';

  const contents = [
    { id: 'what-a-spec-is', label: 'What a spec is here' },
    { id: 'by-crate', label: 'The specifications, by crate' },
    { id: 'no-spec', label: 'The crates with no spec, and why' },
    { id: 'reading-them', label: 'How to read them together' },
    { id: 'workspace', label: 'The workspace’s own specs' }
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
  <h2 id="workspace">The workspace's own specs</h2>
  <p>
    Several specs sit outside any crate, because what they state is true of the whole workspace
    rather than of one member. Ten today. The spec is the normative version in every case; the
    entries below summarise them and link back.
  </p>
  <dl>
    <dt>
      <a href={`${REPO}/blob/main/spec/conformance/index.md`}><code>spec/conformance/index.md</code></a>
    </dt>
    <dd>
      What “supports HL7 v2 releases 2.1 through 2.9” means, exactly — the segments, types, and
      structures by name, and what happens outside them. Summarised at
      <a href="/docs/conformance/">Conformance</a>.
    </dd>
    <dt>
      <a href={`${REPO}/blob/main/spec/phi/index.md`}><code>spec/phi/index.md</code></a>
    </dt>
    <dd>
      What these crates do with protected health information, what they never do, and where a value
      can escape into a log. Summarised at <a href="/docs/patient-data/">Patient data</a>.
    </dd>
    <dt>
      <a href={`${REPO}/blob/main/spec/benchmark/index.md`}><code>spec/benchmark/index.md</code></a>
    </dt>
    <dd>
      How performance is measured, what the published figures mean, and what they are not evidence
      of. Summarised at <a href="/docs/benchmarks/">Benchmarks</a>.
    </dd>
    <dt>
      <a href={`${REPO}/blob/main/spec/rust-msrv-n-minus-2/index.md`}
        ><code>spec/rust-msrv-n-minus-2/index.md</code></a
      >
    </dt>
    <dd>
      The minimum supported Rust version policy every member pins to. See
      <a href="/docs/versions/">Versions and compatibility</a> for what it means in practice.
    </dd>
    <dt>
      <a href={`${REPO}/blob/main/spec/schema-data-provenance/index.md`}
        ><code>spec/schema-data-provenance/index.md</code></a
      >
    </dt>
    <dd>
      Where the bundled dictionary data in <code>hl7-2/schemas/*.json</code> came from, traced as far
      back through git history as the trail goes — no HL7® file is vendored anywhere in this
      workspace.
    </dd>
  </dl>
  <p>
    The five above are about what the crates <em>do</em>. The five below are about how this
    repository runs itself — its own practice, not the code's behavior — but the same rule applies:
    the spec is the source of truth, and a root document that disagrees with one is the bug.
  </p>
  <dl>
    <dt>
      <a href={`${REPO}/blob/main/spec/hl7-trademarks-fair-use/index.md`}
        ><code>spec/hl7-trademarks-fair-use/index.md</code></a
      >
    </dt>
    <dd>
      HL7®'s own fair-use rules for its word marks, quoted verbatim rather than paraphrased, and what
      this project does to follow them — the disclaimer, ® on first use per page, and the “HL7® FHIR®
      standard” naming. Summarised at <a href={`${REPO}/blob/main/TRADEMARKS.md`}
        ><code>TRADEMARKS.md</code></a
      >.
    </dd>
    <dt>
      <a href={`${REPO}/blob/main/spec/professionalization/index.md`}
        ><code>spec/professionalization/index.md</code></a
      >
    </dt>
    <dd>
      What “professional” means for this repository, binding the maintainer as much as any
      contributor — a checked box in <code>tasks.md</code> means verified, a self-declared gap in
      SECURITY.md or MAINTAINERS.md is a promise, and a countable claim is measured before it is
      written.
    </dd>
    <dt>
      <a href={`${REPO}/blob/main/spec/docs-budget-and-links/index.md`}
        ><code>spec/docs-budget-and-links/index.md</code></a
      >
    </dt>
    <dd>
      A size budget and a link-integrity rule binding every markdown document this repository
      tracks, checked by <code>bin/check-docs</code> in CI. Its first run found nine broken links in
      a stray draft the same day it was adopted.
    </dd>
    <dt>
      <a href={`${REPO}/blob/main/spec/serial-comma/index.md`}
        ><code>spec/serial-comma/index.md</code></a
      >
    </dt>
    <dd>English-language prose in this workspace uses the serial comma. That is the whole rule.</dd>
    <dt>
      <a href={`${REPO}/blob/main/spec/special-files-for-public-repos/index.md`}
        ><code>spec/special-files-for-public-repos/index.md</code></a
      >
    </dt>
    <dd>
      The canonical list of root-level files a public repository in this family carries —
      <code>README.md</code> through <code>AI_STATEMENT.md</code> — and, as of this workspace, a
      status section naming which ones exist here and which do not yet.
    </dd>
  </dl>
</DocPage>

<style>
  .outline {
    font-size: 0.875rem;
    opacity: 0.75;
  }
</style>
