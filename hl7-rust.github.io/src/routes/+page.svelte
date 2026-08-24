<script lang="ts">
  import Card from 'lily-design-system-svelte-headless/components/Card/Card.svelte';
  import Badge from 'lily-design-system-svelte-headless/components/Badge/Badge.svelte';

  const er7Example = `PID|1||241900||TEST^FOUAZ`;

  const xmlExample = `<PID>
  <PID.1>1</PID.1>
  <PID.3><CX.1>241900</CX.1></PID.3>
  <PID.5><XPN.1><FN.1>TEST</FN.1></XPN.1><XPN.2>FOUAZ</XPN.2></PID.5>
</PID>`;

  const jsonExample = `{
  "PID": {
    "PID.1": "1",
    "PID.3": { "CX.1": "241900" },
    "PID.5": { "XPN.1": { "FN.1": "TEST" }, "XPN.2": "FOUAZ" }
  }
}`;

  const features = [
    'ER7 parsing at every level: segments, fields, repetitions (~), components (^), and subcomponents (&).',
    'Dynamic delimiters, read from MSH-1/MSH-2 rather than hardcoded.',
    'Typed names: built-in HL7 v2.5 tables map each field of the common segments and composite types to its data type — Z-segments and uncommon types still convert, using positional generic names.',
    'Message-structure groups for ACK, ADT_A01, ORM_O01, and ORU_R01, flattened back out again on the way in.',
    'Graceful fallback rather than failure: unknown structures render flat, unknown fields use positional generic names.',
    'The two reverse crates carry no HL7 v2.5 dictionary of their own — they reconstruct a message purely from the position each forward crate already encodes in every element/key name, and re-escape decoded text back to raw ER7.',
    'None of the four crates is a validator — no schema, cardinality, or table checking is performed.'
  ];
</script>

<svelte:head>
  <title>HL7 Rust</title>
</svelte:head>

<section class="hero">
  <div class="hero-inner">
    <p class="eyebrow"><Badge label="Organization">hl7-rust</Badge></p>
    <h1>HL7 Rust</h1>
    <p class="lede">
      Rust libraries and command-line tools for HL7 version 2.5: a parser, dictionary, and
      validator at the core; MLLP and SOAP transports; and converters that move messages
      losslessly, in both directions, between the traditional pipe-delimited ER7 encoding and
      XML or JSON.
    </p>
    <div class="cta-row">
      <a class="button" href="https://github.com/hl7-rust">GitHub organization</a>
    </div>
  </div>
</section>

<section class="section">
  <div class="section-inner">
    <h2>Core</h2>
    <p>The standards themselves, and the umbrella crate that re-exports them.</p>
    <div class="card-grid">
      <Card
        heading="hl7-2"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2"
        headingLevel={3}
        class="card-featured"
      >
        <p><Badge type="success" label="This is the main crate">Start here</Badge></p>
        <p>
          HL7 v2 (releases 2.1-2.9) for Rust: parse, navigate, validate, modify, and render HL7
          version 2 messages, in three modes — generic, schema-based, and struct-based. Also a
          CLI (<code>hl7-v2</code>). This is what everything else in the ecosystem builds on.
        </p>
      </Card>
      <Card
        heading="hl7-3"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-3"
        headingLevel={3}
      >
        <p>
          HL7 v3 for Rust: the Reference Information Model (RIM) backbone classes, coded values,
          and the three-level message envelope (transport wrapper, control act wrapper, domain
          payload). A foundation, not a full implementation.
        </p>
      </Card>
      <Card
        heading="hl7"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7"
        headingLevel={3}
      >
        <p>
          The umbrella crate: re-exports one module per HL7 standard —
          <code>hl7::v2</code> and <code>hl7::v3</code> today, with room for
          <code>hl7::fhir</code> as that standard gets implemented.
        </p>
      </Card>
    </div>
  </div>
</section>

<section class="section section-alt">
  <div class="section-inner">
    <h2>Transports</h2>
    <div class="card-grid">
      <Card
        heading="hl7-2-mllp"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-mllp"
        headingLevel={3}
      >
        <p>
          HL7 v2 MLLP: the Minimal Lower Layer Protocol that frames HL7 version 2 messages on a
          TCP stream. Framing, streaming, acknowledgements, and a transport trait.
        </p>
      </Card>
      <Card
        heading="hl7-2-soap"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-soap"
        headingLevel={3}
      >
        <p>
          HL7 v2 over SOAP: the envelope, faults, payload carriage, WSDL, and response evaluation
          that carry HL7 version 2 messages over HTTP instead of MLLP.
        </p>
      </Card>
      <Card
        heading="hl7-3-soap"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-3-soap"
        headingLevel={3}
      >
        <p>
          HL7 v3 over SOAP: the envelope, faults, message carriage, WSDL, and the real HL7 v3
          acknowledgement (<code>MCCI_IN000002UV01</code>) — SOAP is v3's own historically
          dominant transport, not an alternative to something else.
        </p>
      </Card>
    </div>
  </div>
</section>

<section class="section">
  <div class="section-inner">
    <h2>Format conversions</h2>
    <div class="card-grid">
      <Card
        heading="hl7-2-from-er7-into-xml"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-xml"
        headingLevel={3}
      >
        <p>
          ER7 to the official HL7 <strong>v2.xml</strong> XML representation
          (<code>urn:hl7-org:v2xml</code>).
        </p>
      </Card>
      <Card
        heading="hl7-2-from-xml-into-er7"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-xml-into-er7"
        headingLevel={3}
      >
        <p>
          v2.xml XML back to ER7 — reads what the crate above writes, with no HL7 v2.5
          dictionary of its own.
        </p>
      </Card>
      <Card
        heading="hl7-2-from-er7-into-json"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-er7-into-json"
        headingLevel={3}
      >
        <p>
          ER7 to a typed <strong>JSON</strong> representation designed to preserve everything
          v2.xml preserves, using idiomatic JSON instead of XML's constructs.
        </p>
      </Card>
      <Card
        heading="hl7-2-from-json-into-er7"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-json-into-er7"
        headingLevel={3}
      >
        <p>
          Typed JSON back to ER7 — reads what the crate above writes, with no HL7 v2.5
          dictionary of its own.
        </p>
      </Card>
    </div>
    <p>
      All four crates share the same ER7 parser
      (<a href="https://crates.io/crates/er7"><code>er7</code></a> on crates.io). The two
      forward crates share the same HL7 v2.5 data-type tables and message-structure grammars —
      only the rendered output format differs. The two reverse crates need none of that: each
      element or key name a forward crate writes already carries its own position, so reversal
      is purely structural — see each reverse crate's <code>spec/index.md</code> §1.1.
    </p>
  </div>
</section>

<section class="section">
  <div class="section-inner">
    <h2>Tooling</h2>
    <div class="card-grid">
      <Card
        heading="hl7-2-derive"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-derive"
        headingLevel={3}
      >
        <p>
          Derive macros for the <code>hl7-2</code> crate: <code>#[derive(FromHl7)]</code> and
          <code>#[derive(ToHl7)]</code> map struct fields to HL7 v2 message paths. Used through
          <code>hl7-2</code>'s <code>derive</code> feature, not directly.
        </p>
      </Card>
      <Card
        heading="hl7-3-derive"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-3-derive"
        headingLevel={3}
      >
        <p>
          Derive macro for the <code>hl7-3</code> crate: <code>#[derive(FromElement)]</code> maps
          struct fields to HL7 v3 XML element attributes and children. Used through
          <code>hl7-3</code>'s <code>derive</code> feature, not directly.
        </p>
      </Card>
      <Card
        heading="hl7-2-from-xsd-into-json-dictionary"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-from-xsd-into-json-dictionary"
        headingLevel={3}
      >
        <p>
          Reads a directory of HL7 v2 XML Schema Definition (XSD) files — the v2.xml encoding, as
          published or as a vendor customised it — and writes the JSON dictionary the
          <code>hl7-2</code> crate reads.
        </p>
      </Card>
      <Card
        heading="hl7-2-xml-lite-helper"
        href="https://github.com/hl7-rust/hl7-rust/tree/main/hl7-2-xml-lite-helper"
        headingLevel={3}
      >
        <p>
          A small, dependency-free XML reader shared by the <code>hl7-2</code> crates: elements,
          attributes, text, and nesting, for documents whose shape you already know, with
          namespace prefixes ignored rather than resolved.
        </p>
      </Card>
    </div>
  </div>
</section>

<section class="section section-alt">
  <div class="section-inner">
    <h2>Example</h2>
    <p>An ER7 fragment such as:</p>
    <pre><code>{er7Example}</code></pre>
    <div class="example-grid">
      <div>
        <h3>converts to XML</h3>
        <pre><code>{xmlExample}</code></pre>
      </div>
      <div>
        <h3>and to JSON</h3>
        <pre><code>{jsonExample}</code></pre>
      </div>
    </div>
    <p>
      Piping either back through its reverse crate reproduces the original
      <code>{er7Example}</code> exactly — that round trip is a runnable example in each forward
      crate's own README.
    </p>
  </div>
</section>

<section class="section">
  <div class="section-inner">
    <h2>What they do</h2>
    <ul class="feature-list">
      {#each features as feature (feature)}
        <li>{feature}</li>
      {/each}
    </ul>
    <p>See <a href="/spec/">Specs</a> for where each project's normative conversion rules live.</p>
  </div>
</section>

<style>
  .hero-inner,
  .section-inner {
    max-width: 64rem;
    margin: 0 auto;
    padding: 2.5rem 1.5rem;
  }

  .eyebrow {
    margin: 0 0 0.75rem;
  }

  .lede {
    max-width: 42rem;
    font-size: 1.125rem;
  }

  .cta-row {
    margin-top: 1.5rem;
  }

  /* The theme's baseline .button uses --color-primary as its background,
     which is also the hero's own background — invert it here so the CTA
     stays visible against the hero. */
  .cta-row :global(.button) {
    background-color: var(--color-base-100, #fff);
    color: var(--color-primary, #4f39f6);
  }

  .cta-row :global(.button:hover) {
    background-color: var(--color-base-200, #f5f5f4);
  }

  .card-grid :global(.card-featured) {
    border: 2px solid var(--color-primary, #4f39f6);
  }

  .section-alt {
    background: var(--color-base-200, #f5f5f4);
  }

  .card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
    gap: 1.5rem;
    margin: 1.5rem 0;
  }

  .example-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
    gap: 1.5rem;
    margin-top: 1rem;
  }

  .example-grid pre {
    overflow-x: auto;
  }

  .feature-list {
    display: grid;
    gap: 0.5rem;
  }
</style>
