<svelte:head>
  <title>Specs — HL7 Rust</title>
</svelte:head>

<section class="section">
  <div class="section-inner">
    <h1>Specs</h1>
    <p>
      This site doesn't restate the conversion rules. Each project's own <code>spec/index.md</code>
      is the normative, section-by-section specification and the single source of truth for its
      behavior:
    </p>

    <h2>Core, transports, and tooling</h2>
    <ul>
      <li>
        <a href="https://github.com/hl7-rust/hl7-rust/blob/main/hl7-2/spec/index.md"
          >hl7-2 spec</a
        > — the HL7 v2 dictionary: data types, message structures, parsing modes, mutation,
        validation
      </li>
      <li>
        <a href="https://github.com/hl7-rust/hl7-rust/blob/main/hl7-3/spec/index.md"
          >hl7-3 spec</a
        > — the RIM backbone classes, coded values, and the three-level message envelope
      </li>
      <li>
        <a href="https://github.com/hl7-rust/hl7-rust/blob/main/hl7-2-mllp/spec/index.md"
          >hl7-2-mllp spec</a
        > — HL7 v2 framed on a TCP stream (MLLP)
      </li>
      <li>
        <a href="https://github.com/hl7-rust/hl7-rust/blob/main/hl7-2-soap/spec/index.md"
          >hl7-2-soap spec</a
        > — HL7 v2 carried in a SOAP envelope over HTTP
      </li>
      <li>
        <a href="https://github.com/hl7-rust/hl7-rust/blob/main/hl7-3-soap/spec/index.md"
          >hl7-3-soap spec</a
        > — HL7 v3 carried in a SOAP envelope over HTTP
      </li>
      <li>
        <a
          href="https://github.com/hl7-rust/hl7-rust/blob/main/hl7-2-from-xsd-into-json-dictionary/spec/index.md"
          >hl7-2-from-xsd-into-json-dictionary spec</a
        > — HL7 v2.xml XSDs → the JSON dictionary <code>hl7-2</code> reads
      </li>
      <li>
        <a
          href="https://github.com/hl7-rust/hl7-rust/blob/main/hl7-2-xml-lite-helper/spec/index.md"
          >hl7-2-xml-lite-helper spec</a
        > — the shared minimal XML reader the v2.xml-facing crates use
      </li>
    </ul>
    <p>
      The umbrella <code>hl7</code> crate and the two <code>*-derive</code> crates
      (<code>hl7-2-derive</code>, <code>hl7-3-derive</code>) have no <code>spec/index.md</code> of
      their own — <code>hl7</code> is a thin re-export with nothing normative to state, and each
      derive crate's macro behavior is documented in its own README (and, for
      <code>hl7-3-derive</code>, in <code>hl7-3</code>'s <code>typed</code> module) rather than as
      a spec.
    </p>

    <h2>Format conversions</h2>
    <ul>
      <li>
        <a
          href="https://github.com/hl7-rust/hl7-rust/blob/main/hl7-2-from-er7-into-xml/spec/index.md"
          >hl7-2-from-er7-into-xml spec</a
        > — ER7 → v2.xml XML
      </li>
      <li>
        <a
          href="https://github.com/hl7-rust/hl7-rust/blob/main/hl7-2-from-xml-into-er7/spec/index.md"
          >hl7-2-from-xml-into-er7 spec</a
        > — v2.xml XML → ER7
      </li>
      <li>
        <a
          href="https://github.com/hl7-rust/hl7-rust/blob/main/hl7-2-from-er7-into-json/spec/index.md"
          >hl7-2-from-er7-into-json spec</a
        > — ER7 → typed JSON
      </li>
      <li>
        <a
          href="https://github.com/hl7-rust/hl7-rust/blob/main/hl7-2-from-json-into-er7/spec/index.md"
          >hl7-2-from-json-into-er7 spec</a
        > — typed JSON → ER7
      </li>
    </ul>
    <p>
      The two <em>forward</em> specs (ER7 → XML, ER7 → JSON) are kept consistent with each other
      except where the target format forces a difference (see each one's own §0 for exactly
      where they're meant to diverge: JSON uses real arrays/<code>null</code>, XML follows
      v2.xml's element/attribute constructs). Each <em>reverse</em> spec documents, in its own
      §1.1, why it needs no HL7 v2.5 data-type dictionary at all: every element or key name its
      forward counterpart writes already carries its own position, so reversing the conversion
      is a purely structural, position-based rebuild.
    </p>
  </div>
</section>

<style>
  .section-inner {
    max-width: 64rem;
    margin: 0 auto;
    padding: 2.5rem 1.5rem;
  }
</style>
