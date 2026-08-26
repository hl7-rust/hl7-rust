<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { REPO } from '$lib/data/crates';

  const contents = [
    { id: 'answer', label: 'The one-paragraph answer' },
    { id: 'principle', label: 'Degrade, never reject' },
    { id: 'dictionary', label: 'The v2.5 dictionary, exactly' },
    { id: 'outside', label: 'What happens outside that set' },
    { id: 'encoding', label: 'Encoding' },
    { id: 'out-of-scope', label: 'What is out of scope' },
    { id: 'other-layers', label: 'Transports, conversions, v3, the HL7® FHIR® standard' },
    { id: 'evaluate', label: 'How to evaluate this yourself' }
  ];

  const spec = `${REPO}/blob/main/spec/conformance/index.md`;

  const segments =
    'AL1 BLG CTI DG1 DSC ERR EVN IN1 MRG MSA MSH NK1 NTE OBR OBX ORC PD1 PID PR1 PV1 PV2 ROL SFT SPM';
  const types =
    'AD AUI CCD CE CNE CNN CP CQ CWE CX DLD DLN DR ED EI EIP ELD ERL FC FN HD JCC MO MOC MSG NDL PL PRL PT RI RP SAD SN SPS TQ TS VID XAD XCN XON XPN XTN';

  const outside = [
    { input: 'A segment not in the 24', result: 'Parses; fields read positionally; SegmentUnknown warning' },
    { input: 'A Z-segment', result: 'Parses; no warning at all — a local extension is the site’s business' },
    { input: 'A field past the end of a known segment', result: 'Parses; reads positionally; FieldUnknown warning' },
    { input: 'A component past the end of a known type', result: 'Parses; reads positionally; ComponentUnknown warning' },
    { input: 'A message structure not in the 4', result: 'Parses; reads as a flat segment list; StructureUnknown warning' },
    { input: 'A data type not in the 42', result: 'Treated as primitive: the value is a scalar' }
  ];

  const evaluate = `# 1. What does it name, and what does it miss?
hl7-v2 --tree --paths redacted-sample.hl7

# 2. Does the round trip come back byte for byte?
hl7-v2 --er7 redacted-sample.hl7 | diff - redacted-sample.hl7

# 3. How big is the gap between this dictionary and your feed?
hl7-v2 --check redacted-sample.hl7`;
</script>

<DocPage
  title="Conformance"
  lede="What “supports HL7 v2 releases 2.1 through 2.9” means here, stated precisely enough to evaluate against — including everything it does not mean."
  {contents}
>
  <Callout type="warning" heading="This is a self-assessment, not a certification">
    <p>
      No certifying body has assessed this project. HL7 International certifies people, not
      libraries, and where product conformance testing exists it is against a specific
      implementation guide and test suite. Every line below is checkable against the code and the
      tests instead — see <a href={spec}>the normative version</a>, which this page summarises.
    </p>
  </Callout>

  <h2 id="answer">The one-paragraph answer</h2>
  <p>
    <a href="/crates/hl7-2/"><code>hl7-2</code></a> reads and writes <strong>any</strong>
    syntactically well-formed ER7 message from <strong>any</strong> release, losing nothing and
    rejecting nothing. What varies by release, and what the dictionary bounds, is how much of the
    message it can <em>name</em>: which fields it knows the data type of, which segments it can
    check against a structure, and which values it can validate. It ships a complete v2.5
    dictionary of 24 segments, 42 composite data types, and 4 message structures, and models the
    other thirteen releases as deltas of it.
  </p>
  <p>
    So: if your criterion is “does it handle our feed without losing data”, the answer is yes for
    any ER7 input. If it is “does it know what every field in our feed means”, the answer is
    bounded by the dictionary below — and extending it is one JSON file.
  </p>

  <h2 id="principle">Degrade, never reject</h2>
  <p>
    Every conformance gap in this project resolves the same way, and it is the design decision the
    rest of this page elaborates:
  </p>
  <p>
    <strong>
      An unmodelled segment, field, data type, structure, or release difference costs you a
      <em>name</em>, never a <em>value</em>.
    </strong>
    The field reads with a positional generic name instead of its data type; the message still
    parses, every value is still reachable by path, and rendering still returns the original bytes.
  </p>
  <p>
    Only four things fail a call: a message with no usable MSH header, a path that is not a path, a
    dictionary that will not load, and — in struct mode — a value that does not fit the Rust type
    you asked for. Everything below the header degrades, and is reported by
    <code>validate()</code> if you want to know.
  </p>
  <p>
    That is why the incompleteness here is bounded rather than dangerous. A library that rejected
    what it did not recognise would turn every gap into a dropped clinical message.
  </p>

  <h2 id="dictionary">The v2.5 dictionary, exactly</h2>
  <p>
    v2.5 is the complete base; every other release is a delta of it. See
    <a href="/docs/versions/">Versions and compatibility</a> for what each release restates. The
    base covers:
  </p>
  <p><strong>24 segments</strong></p>
  <CodeSample language="segments" code={segments} />
  <p><strong>42 composite data types</strong></p>
  <CodeSample language="data types" code={types} />
  <p>
    <strong>4 message structures</strong> — <code>ACK</code>, <code>ADT_A01</code>,
    <code>ORM_O01</code>, <code>ORU_R01</code> — with <code>ADT_A04</code>, <code>ADT_A08</code>,
    and <code>ADT_A13</code> aliased onto <code>ADT_A01</code>.
  </p>
  <p>
    That is the honest number. HL7 v2.5 defines well over a hundred segments and around eighty
    message structures. This dictionary covers the ones carrying the overwhelming majority of real
    interface traffic — admissions, orders, results, and their acknowledgements — and nothing else.
  </p>
  <Callout heading="Extending it is deliberately cheap">
    <p>
      A dictionary is JSON. Add segments, types, structures, or a whole vendor dialect at run time
      with <a href="/guides/dictionaries/">schema mode</a>, or generate one from your own HL7 v2.xml
      XSD files with
      <a href="/crates/hl7-2-from-xsd-into-json-dictionary/"><code>hl7-2-from-xsd-into-json-dictionary</code></a>.
      A dialect can inherit a bundled release and state only its differences.
    </p>
  </Callout>

  <h2 id="outside">What happens outside that set</h2>
  <p>This is the part that decides whether the number above matters to you.</p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Input</th><th>Result</th></tr>
      </thead>
      <tbody>
        {#each outside as row (row.input)}
          <tr><td>{row.input}</td><td>{row.result}</td></tr>
        {/each}
      </tbody>
    </table>
  </div>
  <p>
    In every row the message parses, every value is reachable by path, and the round trip is byte
    for byte.
  </p>

  <h2 id="encoding">Encoding</h2>
  <p>
    Encoding conformance belongs to <a href="https://crates.io/crates/er7"><code>er7</code></a>, and
    it is where this project makes its strongest claims:
  </p>
  <ul>
    <li>
      <strong>Delimiters come from the message.</strong> MSH-1 and MSH-2 declare them; the usual set
      is a default, not an assumption.
    </li>
    <li><strong>Escape sequences</strong> are decoded on demand and re-encoded on the way out.</li>
    <li>
      <strong>The explicit null is distinct from an absent value</strong>, at every level, and stays
      distinct through parsing, mutation, and rendering. This is the single most commonly botched
      detail in HL7 tooling, and it is the difference between “the patient has no middle name” and
      “delete the middle name we have on file”.
    </li>
    <li>
      <strong>Byte-for-byte round trip</strong>: a message parsed and not modified renders back
      identically, after documented input normalization.
    </li>
    <li><strong>Batches</strong> split into their constituent messages.</li>
    <li>
      <strong>Paths</strong> — <code>PID-5.1</code>, <code>OBX[2]-5[1].1.2</code> — with repetition,
      component, and subcomponent addressing.
    </li>
  </ul>

  <h2 id="out-of-scope">What is out of scope</h2>
  <p>Named explicitly, because an evaluation needs the gaps more than the features.</p>
  <ul>
    <li>
      <strong>Vocabulary and code tables.</strong> HL7 tables, LOINC, SNOMED CT, ICD. A coded value
      is read as the string it is; nothing checks that it exists or is allowed there.
    </li>
    <li>
      <strong>Conformance profiles.</strong> HL7 v2 message profiles are not read, applied, or
      generated. Validation is against the dictionary, not against a profile.
    </li>
    <li><strong>Field length limits.</strong> Not modelled, so never enforced.</li>
    <li>
      <strong>Usage codes</strong> beyond the required/repeats pair the dictionary carries.
      Conditional usage rules are not modelled.
    </li>
    <li>
      <strong>Exact repetition upper bounds.</strong> “At most 10” and “unbounded” both read as
      “repeats”.
    </li>
    <li>
      <strong>Implementation guides.</strong> No US Core, no national extension, no IHE profile, no
      jurisdiction-specific rule set.
    </li>
    <li>
      <strong>Clinical semantics.</strong> Nothing here knows that an <code>A08</code> should update
      a patient rather than create one.
    </li>
  </ul>

  <h2 id="other-layers">Transports, conversions, v3, the HL7® FHIR® standard</h2>
  <dl>
    <dt>MLLP</dt>
    <dd>
      Framing, streaming reassembly, and acknowledgement generation. No TLS, no async runtime, no
      pooling, no retry policy, no persistence — and it opens no sockets itself; you supply the
      stream.
    </dd>
    <dt>SOAP</dt>
    <dd>
      SOAP 1.1 envelopes, faults with their HTTP statuses, both ways v2 is carried in a body, the
      response, and a WSDL. No HTTP client or server, and deliberately no SOAP 1.2, WS-Security,
      WS-Addressing, MTOM, or attachments — none appeared in the interfaces this was built from.
    </dd>
    <dt>Format conversions</dt>
    <dd>
      ER7 to and from v2.xml and typed JSON, in both directions, with the tree names, the JSON keys,
      and the XML elements deliberately identical. The round trip is a test, not an aspiration.
    </dd>
    <dt>HL7 v3</dt>
    <dd>
      A foundation, not an implementation: six RIM backbone classes, six data types, and the
      three-level envelope read generically. No vocabulary validation, no per-interaction schemas,
      and <strong>no CDA</strong> — it reuses the RIM but its document model is its own thing.
    </dd>
    <dt>HL7® FHIR® standard</dt>
    <dd>
      <strong>Not implemented.</strong> The umbrella crate reserves the module path and nothing
      more. If you need the HL7® FHIR® standard today, this project is not it.
    </dd>
  </dl>

  <h2 id="evaluate">How to evaluate this yourself</h2>
  <p>Do not take the above on trust; it takes about an hour to check.</p>
  <CodeSample language="sh" caption="On a redacted sample from your own feed" code={evaluate} />
  <ol>
    <li>
      <strong>What does it name?</strong> A field that reads as <code>PID.34</code> rather than by
      its data type is a dictionary gap, and you will see exactly which ones you have.
    </li>
    <li><strong>Does the round trip hold?</strong> The diff should be empty.</li>
    <li>
      <strong>How big is the gap?</strong> Count the warnings by kind over a day of redacted
      traffic. <code>SegmentUnknown</code> and <code>StructureUnknown</code> size it for you.
    </li>
    <li>
      <strong>Then decide whether the gap is work.</strong> A vendor dialect is one JSON file, or a
      dictionary generated from your own XSDs. Usually an afternoon, not a project.
    </li>
  </ol>
  <Callout heading="Do not paste patient data anywhere public">
    <p>
      Redact before a sample leaves your network, and before it reaches an issue tracker. See
      <a href="/docs/patient-data/">Patient data</a> for how, and for what these crates do with what
      you hand them.
    </p>
  </Callout>
</DocPage>
