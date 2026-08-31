<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-2-from-er7-into-xml');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'example', label: 'What it produces' },
    { id: 'cli', label: 'Command line' },
    { id: 'library', label: 'Library' },
    { id: 'does', label: 'What it does' },
    { id: 'fallbacks', label: 'Fallback behavior' },
    { id: 'limitations', label: 'Limitations' },
    { id: 'round-trip', label: 'Round trip with the reverse crate' },
    { id: 'related', label: 'Related crates' }
  ];

  const er7 = `PID|1||241900||TEST^FOUAZ`;

  const xml = `<PID>
  <PID.1>1</PID.1>
  <PID.3>
    <CX.1>241900</CX.1>
  </PID.3>
  <PID.5>
    <XPN.1>
      <FN.1>TEST</FN.1>
    </XPN.1>
    <XPN.2>FOUAZ</XPN.2>
  </PID.5>
</PID>`;

  const cli = `# From a file to stdout
hl7-2-from-er7-into-xml samples/orm_o01.hl7

# From stdin, to a file
cat samples/oru_r01.hl7 | hl7-2-from-er7-into-xml -o out.xml

# Disable message-structure grouping
hl7-2-from-er7-into-xml --flat samples/orm_o01.hl7

# Convert against a dictionary built from XSDs instead of bundled v2.5
hl7-2-from-er7-into-xml --dictionary my-dialect.json samples/orm_o01.hl7

# Let that dictionary decide the document's exact shape
hl7-2-from-er7-into-xml --dictionary my-dialect.json --schema-shape samples/orm_o01.hl7`;

  const library = `let er7 = "MSH|^~\\\\&|hphis||EPIC||20131011093851||ORM^O01|14AAACVDD|P|2.5\\r\\
           PID|1||241900||MEDIANO^FOUAZ\\r\\
           ORC|NW|ORD1";

let xml = hl7_2_from_er7_into_xml::convert(er7)?;`;

  const batch = `use hl7_2_from_er7_into_xml::{convert, split_messages};

let batch = "MSH|^~\\\\&|A||||1||ACK|1|P|2.5\\rMSA|AA|1\\r\\
             MSH|^~\\\\&|B||||2||ACK|2|P|2.5\\rMSA|AA|2";

for message in split_messages(batch) {
    match convert(&message) {
        Ok(xml) => println!("{xml}"),
        Err(e) => eprintln!("skipping malformed message: {e}"),
    }
}`;

  const roundTrip = `hl7-2-from-er7-into-xml samples/orm_o01.hl7 \\
  | hl7-2-from-xml-into-er7`;
</script>

<DocPage lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>
    Convert HL7 v2.5 messages from the traditional pipe-delimited ER7 encoding to the official HL7
    <strong>v2.xml</strong> XML representation (<code>urn:hl7-org:v2xml</code>), as a Rust library
    and command-line tool.
  </p>
  <p>
    The ER7 encoding itself comes from the
    <a href="https://crates.io/crates/er7"><code>er7</code></a> crate. This crate is the layer above
    it: the HL7 v2.5 data-type tables that name XML elements, the message-structure grammars that
    group segments, and the XML renderer.
  </p>
  <p>
    Since 0.5.0 those tables and grammars come from the
    <a href="/crates/hl7-2/"><code>hl7-2</code></a> dictionary rather than being hand-written here —
    which is also what lets a caller pass <code>--dictionary</code> to convert against a vendor's own
    XML Schema instead of the bundled v2.5 release.
  </p>

  <h2 id="example">What it produces</h2>
  <p>An ER7 fragment such as:</p>
  <CodeSample language="er7" code={er7} />
  <p>converts to the v2.xml structure, with components named after their HL7 v2.5 data types:</p>
  <CodeSample language="xml" code={xml} />

  <h2 id="cli">Command line</h2>
  <CodeSample language="sh" code={cli} />
  <p>
    <code>--dictionary FILE</code> reads a JSON dictionary — such as one built by
    <a href="/crates/hl7-2-from-xsd-into-json-dictionary/"
      ><code>hl7-2-from-xsd-into-json-dictionary</code></a
    > from a directory of XSDs — in place of the bundled HL7 v2.5 tables.
  </p>
  <p>
    <code>--schema-shape</code> changes how that dictionary is read: instead of only naming what a
    field <em>is</em>, it decides what the document <em>contains</em> — required fields are written
    even when empty, fields that cannot repeat keep their repetition separator as text, and no field
    the dictionary does not declare is written. That is what makes the output validate against the
    schemas the dictionary came from.
  </p>
  <p>
    Input may hold one message, several messages, or an HL7 batch file (FHS/BHS envelopes are
    dropped); each message becomes one XML document.
  </p>

  <h2 id="library">Library</h2>
  <CodeSample language="rust" code={library} />
  <p>
    See also <code>convert_with_options</code> (for example <code>Options &#123; flat: true &#125;</code>)
    and <code>split_messages</code> for batch input:
  </p>
  <CodeSample language="rust" code={batch} />
  <p>
    <code>convert</code> and <code>convert_with_options</code> return
    <code>Result&lt;String, Hl7Error&gt;</code>; an <code>Err</code> only ever means the message has
    no usable MSH header — empty input, missing MSH, or a malformed MSH header. Everything below
    that always converts, falling back to generic names or a flat layout rather than failing.
  </p>

  <h2 id="does">What it does</h2>
  <ul>
    <li>
      <strong>ER7 parsing</strong> at every level: segments, fields, repetitions (<code>~</code>),
      components (<code>^</code>), and subcomponents (<code>&amp;</code>).
    </li>
    <li>
      <strong>Dynamic delimiters</strong>: the separator set is read from <code>MSH-1</code> and
      <code>MSH-2</code> rather than hardcoded, and both are emitted literally per the standard.
    </li>
    <li>
      <strong>Escape sequences</strong>: <code>\F\ \S\ \T\ \R\ \E\</code> decode to the delimiter
      characters and <code>\Xhh..\</code> decodes hex bytes, before XML escaping. Unrecognized
      sequences — formatting commands such as <code>\.br\</code> — are kept literally.
    </li>
    <li>
      <strong>Typed element names</strong>: the dictionary maps each field of the common segments
      (MSH, SFT, EVN, PID, PD1, NK1, PV1, PV2, ROL, DG1, PR1, ORC, OBR, OBX, NTE, AL1, IN1, MRG, MSA,
      ERR, DSC, BLG, CTI, SPM) to its data type, and each composite type (CX, XPN, XCN, XAD, CE, CWE,
      EI, HD, TS, …) to its component types — producing
      <code>&lt;PID.5&gt;&lt;XPN.1&gt;&lt;FN.1&gt;</code> style nesting.
    </li>
    <li>
      <strong>OBX-5 variable typing</strong>: the value type declared in <code>OBX-2</code> (CE, CX,
      SN, …) names the <code>OBX-5</code> components.
    </li>
    <li>
      <strong>HL7 null</strong>: the explicit null <code>""</code> keeps its literal text
      (<code>&lt;PID.2&gt;""&lt;/PID.2&gt;</code>). An <em>empty</em> element means “not sent”
      instead — the XML Encoding Rules give the two opposite meanings, so they do not share an
      encoding.
    </li>
    <li>
      <strong>Message-structure groups</strong>: for known structures the segments are nested into
      their official groups, e.g. <code>&lt;ORM_O01.PATIENT&gt;</code>. Grammars are included for
      ACK, ADT_A01 (also used by ADT^A04/A08/A13), ORM_O01, and ORU_R01. The root element name comes
      from <code>MSH-9.3</code> when present, otherwise from <code>MSH-9.1</code> and
      <code>MSH-9.2</code>.
    </li>
  </ul>

  <h2 id="fallbacks">Fallback behavior</h2>
  <p>Fidelity degrades gracefully instead of failing:</p>
  <ul>
    <li>
      A message whose segment sequence does not fit its declared structure — it contains Z-segments,
      or uses a structure with no built-in grammar — renders with all segments
      <strong>flat</strong> under the root element.
    </li>
    <li>
      Fields of unknown segments, and segment fields beyond the built-in tables, use positional
      generic names: <code>&lt;ZDS.1&gt;</code>, <code>&lt;ZDS.1.1&gt;</code>, and so on.
    </li>
  </ul>

  <h2 id="limitations">Limitations</h2>
  <Callout type="warning" heading="Not a validator">
    <p>
      No XSD validation, cardinality, or table checking is performed; the input is assumed to be
      sensible HL7 v2.5. If you need conformance checking, run
      <a href="/guides/validating/"><code>hl7-2</code></a> first and convert what passes.
    </p>
  </Callout>
  <ul>
    <li>Only the four message structures listed above are grouped; everything else renders flat.</li>
    <li>
      ORM_O01 order detail supports the common OBR choice; RQD/RQ1/RXO/ODS/ODT detail segments cause
      a flat rendering.
    </li>
    <li>
      Formatting escape sequences are preserved as literal text rather than mapped to
      <code>&lt;escape/&gt;</code> elements.
    </li>
  </ul>

  <h2 id="round-trip">Round trip with the reverse crate</h2>
  <p>
    Because <a href="/crates/hl7-2-from-xml-into-er7/"><code>hl7-2-from-xml-into-er7</code></a> reads
    back exactly what this crate writes, the two compose into a lossless round trip you can run from
    the shell:
  </p>
  <CodeSample language="sh" code={roundTrip} />
  <p>
    The output is the original ER7 message, canonicalized — a good smoke test after changing either
    crate's naming rules, since a drift in one breaks the other's assumptions. See
    <a href="/tutorials/round-trip/">A lossless round trip</a>.
  </p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
