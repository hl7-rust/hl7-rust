<script lang="ts">
  import { Badge } from 'lily-design-system-svelte-headless';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import LinkCards from '$lib/components/LinkCards.svelte';
  import { CATEGORIES, CRATES, cratesIn, type CrateCategory } from '$lib/data/crates';

  const start = [
    {
      href: '/docs/quickstart/',
      label: 'Quick start',
      blurb:
        'Install one crate, parse a real ORU^R01, read a value out of it, and send back an acknowledgement.'
    },
    {
      href: '/guides/',
      label: 'Guides',
      blurb:
        'One task per page: parsing, navigating, validating, converting, MLLP, SOAP, vendor dictionaries.'
    },
    {
      href: '/tutorials/',
      label: 'Tutorials',
      blurb:
        'Longer walkthroughs — taming a vendor dialect, standing up an MLLP listener that answers.'
    },
    {
      href: '/crates/',
      label: 'Crate reference',
      blurb: 'Fourteen workspace members, each with its features, dependencies, and API tour.'
    }
  ];

  const er7 = `MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5
PID|1||444333222^^^ACME&1.2.3.4&ISO^MR||EVERYWOMAN^EVE^E||19620320|F
OBR|1|ORD776655|LAB2233|24331-1^Lipid Panel^LN|||20260813071500
OBX|1|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F`;

  const parse = `use hl7::v2;

let message = v2::parse(text)?;

// A path addresses a value; the tree names it.
assert_eq!(message.get("PID-5.1")?.as_deref(), Some("EVERYWOMAN"));
assert_eq!(message.tree().find("XPN.1").unwrap().text(), "EVERYWOMAN");

// Every node knows the path that reads it back.
let second = message.tree().find_all("OBX").nth(1);
assert_eq!(message.structure_id(), "ORU_R01");`;

  const xml = `<PID>
  <PID.1>1</PID.1>
  <PID.3>
    <CX.1>241900</CX.1>
  </PID.3>
  <PID.5>
    <XPN.1><FN.1>TEST</FN.1></XPN.1>
    <XPN.2>FOUAZ</XPN.2>
  </PID.5>
</PID>`;

  const json = `{
  "PID": {
    "PID.1": "1",
    "PID.3": { "CX.1": "241900" },
    "PID.5": {
      "XPN.1": { "FN.1": "TEST" },
      "XPN.2": "FOUAZ"
    }
  }
}`;

  const map = `er7                                    the ER7 encoding: delimiters,
                                       escapes, paths, byte-for-byte
                                       rendering, batch splitting
  |
hl7-2                                  the HL7 v2 dictionary: releases
  |                                    2.1-2.9, data types, message
  |                                    structures; three parsing modes;
  |                                    mutation; validation
  |
  +-- hl7-2-mllp                       transport: HL7 v2 over TCP (MLLP)
  +-- hl7-2-soap                       transport: HL7 v2 over HTTP (SOAP)
  +-- hl7-2-from-er7-into-json         format conversions
  +-- hl7-2-from-er7-into-xml
  +-- hl7-2-from-json-into-er7
  +-- hl7-2-from-xml-into-er7
  +-- hl7-2-from-xsd-into-json-dictionary   writes the dictionaries
  |                                          hl7-2 reads, from HL7
  |                                          v2.xml XSDs
  +-- hl7-2-xml-lite-helper            shared minimal XML reader, also
        |                              used directly by:
        +-- hl7-3                      HL7 v3: RIM backbone classes,
              |                        coded values, the three-level
              |                        message envelope
              +-- hl7-3-derive         #[derive(FromElement)]
              +-- hl7-3-soap           transport: HL7 v3 over HTTP (SOAP)

hl7                                    the umbrella crate — hl7::v2 and
                                       hl7::v3 today, room for hl7::fhir`;

  const order: CrateCategory[] = ['core', 'transport', 'conversion', 'tooling'];
</script>

<svelte:head>
  <title>HL7 Rust — HL7 v2 and v3 for Rust</title>
  <meta
    name="description"
    content="HL7 Rust: parse, navigate, validate, modify, render, transport, and convert HL7 v2 and v3 messages in Rust. Documentation, guides, tutorials, examples, and a reference page for every crate."
  />
</svelte:head>

<section class="site-hero">
  <div class="site-hero-inner">
    <p class="hero-eyebrow"><Badge label="GitHub organization">hl7-rust</Badge></p>
    <h1>HL7, in Rust, one crate per layer</h1>
    <p class="hero-lede">
      Parse, navigate, validate, modify, and render Health Level Seven messages. HL7 v2 releases
      2.1 through 2.9 with a real dictionary behind them; HL7 v3's RIM backbone and message
      envelope; MLLP and SOAP transports; and lossless conversion between ER7, v2.xml, and JSON.
      Small dependency trees, permissive licensing, and a normative specification for every crate
      whose behavior is normative.
    </p>
    <div class="hero-actions">
      <a class="hero-button hero-button-primary" href="/docs/quickstart/">Quick start</a>
      <a class="hero-button" href="/docs/">Documentation</a>
      <a class="hero-button" href="/crates/">Crate reference</a>
    </div>
    <p class="hero-install"><code>cargo add hl7</code></p>
  </div>
</section>

<section class="band">
  <div class="band-inner">
    <h2>Start here</h2>
    <LinkCards links={start} headingLevel={3} label="Ways in" />
  </div>
</section>

<section class="band band-alt">
  <div class="band-inner">
    <h2>What it looks like</h2>
    <p class="band-lede">
      HL7 v2 is pipes and carets. The hard part is not the syntax — it is knowing what the pipes
      and carets <em>mean</em> in the release the sender speaks. That is what the dictionary is for.
    </p>
    <div class="split">
      <div>
        <CodeSample
          language="er7"
          caption="A lipid panel result, as it arrives on the wire"
          code={er7}
        />
      </div>
      <div>
        <CodeSample language="rust" caption="Read from it by path or by name" code={parse} />
      </div>
    </div>
    <p>
      <a href="/docs/quickstart/">Follow the quick start →</a>
    </p>
  </div>
</section>

<section class="band">
  <div class="band-inner">
    <h2>Convert it, losslessly, both ways</h2>
    <p class="band-lede">
      The same ER7 fragment <code>PID|1||241900||TEST^FOUAZ</code> converts to the official v2.xml
      representation and to a typed JSON mapping designed to preserve everything v2.xml preserves.
      Piping either back through its reverse crate reproduces the original message.
    </p>
    <div class="split">
      <CodeSample language="xml" caption="hl7-2-from-er7-into-xml" code={xml} />
      <CodeSample language="json" caption="hl7-2-from-er7-into-json" code={json} />
    </div>
    <p><a href="/guides/converting/">The conversion guide →</a></p>
  </div>
</section>

<section class="band band-alt">
  <div class="band-inner">
    <h2>How the crates fit together</h2>
    <p class="band-lede">
      One crate per layer, one module per standard. Each seam is a place you can stop: take the
      ER7 encoding alone, take v2 without a transport, take a transport without the dictionary.
    </p>
    <CodeSample language="text" label="The workspace, as a dependency map" code={map} />
    <p><a href="/docs/architecture/">Why the seams fall where they do →</a></p>
  </div>
</section>

<section class="band">
  <div class="band-inner">
    <h2>Every crate</h2>
    {#each order as category (category)}
      <h3>{CATEGORIES[category].label}</h3>
      <p class="band-lede">{CATEGORIES[category].blurb}</p>
      <LinkCards
        headingLevel={4}
        label={CATEGORIES[category].label}
        links={cratesIn(category).map((crate) => ({
          href: `/crates/${crate.slug}/`,
          label: crate.name,
          blurb: crate.tagline
        }))}
      />
    {/each}
    <p>{CRATES.length} crates in one Cargo workspace, plus <code>er7</code>, which is its own.</p>
  </div>
</section>

<style>
  .site-hero {
    background: var(--color-primary);
    color: var(--color-primary-content);
  }

  .site-hero-inner,
  .band-inner {
    max-width: 72rem;
    margin: 0 auto;
    padding: 3.5rem 1.5rem;
  }

  .hero-eyebrow {
    margin: 0 0 1rem;
  }

  /* Lily's baseline badge is a solid neutral pill, which on the primary-filled
     hero reads as a black blot. Invert it to sit on the band. */
  .hero-eyebrow :global(.badge) {
    background: rgb(255 255 255 / 0.16);
    color: inherit;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    padding: 0.1875rem 0.625rem;
  }

  .site-hero h1 {
    margin: 0 0 1rem;
    font-size: clamp(2rem, 5vw, 3rem);
    line-height: 1.15;
  }

  .hero-lede {
    max-width: 46rem;
    font-size: 1.125rem;
    line-height: 1.65;
    margin: 0;
  }

  .hero-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin: 1.75rem 0 0;
  }

  .hero-button {
    display: inline-block;
    padding: 0.625rem 1.25rem;
    border-radius: var(--radius-field, 0.25rem);
    border: 1px solid currentColor;
    text-decoration: none;
    color: inherit;
    font-weight: 600;
  }

  .hero-button:hover {
    background: rgb(255 255 255 / 0.14);
  }

  /* The band is --color-primary in both themes, so the primary call to action
     is painted in the pair that is guaranteed to contrast with it — not in the
     page surface, which is dark in the dark theme and disappears here. */
  .hero-button-primary {
    background: var(--color-primary-content);
    color: var(--color-primary);
    border-color: var(--color-primary-content);
  }

  .hero-button-primary:hover {
    opacity: 0.88;
  }

  .hero-install {
    margin: 1.5rem 0 0;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.9375rem;
    opacity: 0.9;
  }

  .band-alt {
    background: var(--color-base-200);
  }

  .band h2 {
    margin: 0 0 0.5rem;
    font-size: 1.75rem;
  }

  .band h3 {
    margin: 2.5rem 0 0.25rem;
  }

  .band-lede {
    max-width: 46rem;
    margin: 0 0 1rem;
    line-height: 1.7;
  }

  .split {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(20rem, 1fr));
    gap: 1.5rem;
    align-items: start;
  }
</style>
